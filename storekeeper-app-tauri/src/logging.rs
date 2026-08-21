//! Tracing setup and log-file reads.
//!
//! Log lines go to stdout in the human-readable format and to a daily rolling
//! file as one JSON object per line, which the in-app viewer parses.

use anyhow::Context;
use anyhow::Result;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use storekeeper_core::AppConfig;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload;
use tracing_subscriber::util::SubscriberInitExt;

/// Config subdirectory that holds rolling log files.
const LOG_DIR_NAME: &str = "logs";

const LOG_FILE_PREFIX: &str = "storekeeper";

const LOG_FILE_SUFFIX: &str = "log";

/// Number of daily log files kept before the oldest is deleted.
const MAX_LOG_FILES: usize = 7;

/// Log level applied when neither `RUST_LOG` nor the config supplies one.
const DEFAULT_LEVEL: &str = "info";

/// Upper bound on how much of a log file's tail is read for one request.
const TAIL_READ_BYTES: u64 = 1 << 20;

/// Swap the global log filter while the application runs.
///
/// A `RUST_LOG` value set at startup keeps precedence for the process lifetime.
pub struct LogFilter {
    handle: reload::Handle<EnvFilter, Registry>,
    env_override: bool,
}

impl LogFilter {
    /// Apply a configured log level to the subscriber.
    ///
    /// Keep the current filter when `RUST_LOG` holds precedence or `level`
    /// contains no valid directive.
    pub fn set_level(&self, level: &str) {
        if self.env_override {
            tracing::debug!(
                level = level,
                "RUST_LOG is set, keeping it over the configured log level"
            );
            return;
        }

        let filter = match EnvFilter::try_new(level) {
            Ok(filter) => filter,
            Err(e) => {
                tracing::warn!(level = level, error = %e, "Ignoring unparseable log level");
                return;
            }
        };

        match self.handle.reload(filter) {
            Ok(()) => tracing::info!(level = level, "Log level applied"),
            Err(e) => tracing::warn!(error = %e, "Failed to apply the configured log level"),
        }
    }
}

/// Return the directory holding rolling log files.
///
/// # Errors
///
/// Returns an error if the OS config directory cannot be resolved or is not
/// valid UTF-8.
pub fn log_dir() -> Result<Utf8PathBuf> {
    Ok(AppConfig::config_dir()?.join(LOG_DIR_NAME))
}

/// Install the tracing subscriber and return its reload handle.
///
/// `RUST_LOG` selects and pins the startup filter. Without it, the filter
/// starts at [`DEFAULT_LEVEL`] and the caller applies the configured level
/// after the config loads. If the log directory cannot be created, stdout
/// remains the only destination and startup continues.
///
/// # Panics
///
/// Panics if a global tracing subscriber is already installed.
#[must_use]
pub fn init() -> LogFilter {
    let env_directives = std::env::var(EnvFilter::DEFAULT_ENV)
        .ok()
        .filter(|directives| !directives.trim().is_empty());
    let env_override = env_directives.is_some();
    let filter = env_directives
        .and_then(|directives| EnvFilter::try_new(directives).ok())
        .unwrap_or_else(|| EnvFilter::new(DEFAULT_LEVEL));
    let (filter_layer, handle) = reload::Layer::new(filter);

    let (file_layer, file_error) = match log_dir().and_then(|dir| build_appender(&dir)) {
        Ok(appender) => (
            Some(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(appender),
            ),
            None,
        ),
        Err(e) => (None, Some(e)),
    };

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer)
        .init();

    if let Some(e) = file_error {
        tracing::error!(error = %format!("{e:#}"), "Logging to stdout only");
    }

    LogFilter {
        handle,
        env_override,
    }
}

/// Build a daily rolling appender in `dir` with the filename parts that
/// [`newest_log_file`] matches.
fn build_appender(dir: &Utf8Path) -> Result<tracing_appender::rolling::RollingFileAppender> {
    fs_err::create_dir_all(dir).context("failed to create the log directory")?;

    tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix(LOG_FILE_PREFIX)
        .filename_suffix(LOG_FILE_SUFFIX)
        .max_log_files(MAX_LOG_FILES)
        .build(dir)
        .context("failed to build the rolling log appender")
}

/// Select the lexicographically latest date-stamped log file, or return `None`
/// when the directory or a matching file is absent.
fn newest_log_file(dir: &Utf8Path) -> Result<Option<Utf8PathBuf>> {
    if !dir.exists() {
        return Ok(None);
    }

    let mut newest: Option<Utf8PathBuf> = None;
    for entry in fs_err::read_dir(dir).context("failed to read the log directory")? {
        let entry = entry.context("failed to read a log directory entry")?;
        let Ok(path) = Utf8PathBuf::from_path_buf(entry.path()) else {
            continue;
        };
        let Some(name) = path.file_name() else {
            continue;
        };
        if !name.starts_with(LOG_FILE_PREFIX) || !name.ends_with(LOG_FILE_SUFFIX) {
            continue;
        }
        if newest.as_ref().is_none_or(|current| path > *current) {
            newest = Some(path);
        }
    }

    Ok(newest)
}

/// Reads the last `lines` entries of the current day's log file.
///
/// Each returned entry is one raw JSON line. The read covers at most
/// [`TAIL_READ_BYTES`] from the end of the file, and when that window opens
/// mid-line its first entry is dropped. Returns an empty vector when no log
/// file exists yet.
///
/// # Errors
///
/// Returns an error if the log directory or the file cannot be read.
pub fn read_tail(lines: usize) -> Result<Vec<String>> {
    read_tail_from(&log_dir()?, TAIL_READ_BYTES, lines)
}

/// Read at most `window_bytes` from the end of the newest log file in `dir`.
fn read_tail_from(dir: &Utf8Path, window_bytes: u64, lines: usize) -> Result<Vec<String>> {
    let Some(path) = newest_log_file(dir)? else {
        return Ok(Vec::new());
    };

    let mut file = fs_err::File::open(&path).context("failed to open the log file")?;
    let length = file
        .metadata()
        .context("failed to read the log file size")?
        .len();
    // Read one byte before the window to distinguish a line boundary from a
    // truncated first line.
    let start = length.saturating_sub(window_bytes);
    let probed = start > 0;

    file.seek(SeekFrom::Start(if probed { start - 1 } else { start }))
        .context("failed to seek the log file")?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .context("failed to read the log file")?;
    let window = String::from_utf8_lossy(&bytes);

    let (window, partial_first) = match window.strip_prefix('\n') {
        Some(whole) if probed => (whole, false),
        _ => (window.as_ref(), probed),
    };

    Ok(tail_lines(window, partial_first, lines))
}

/// Drop a partial first entry, then return the last `lines` entries in order.
fn tail_lines(window: &str, partial_first: bool, lines: usize) -> Vec<String> {
    let mut entries = window.lines();
    if partial_first {
        entries.next();
    }

    let entries: Vec<&str> = entries.collect();
    let skip = entries.len().saturating_sub(lines);
    entries
        .into_iter()
        .skip(skip)
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Clear the fixture directory on entry so a failed test cannot leave a
    /// dated log file for the next run.
    fn fixture_dir(name: &str) -> Utf8PathBuf {
        let dir = Utf8PathBuf::from_path_buf(std::env::temp_dir())
            .expect("the temp directory path is valid UTF-8")
            .join(format!("storekeeper-{name}"));
        if dir.exists() {
            fs_err::remove_dir_all(&dir).expect("clear the fixture directory");
        }
        dir
    }

    #[test]
    fn newest_log_file_is_absent_for_a_directory_that_was_never_written() {
        let dir = fixture_dir("logs-that-do-not-exist");
        assert!(
            newest_log_file(&dir)
                .expect("a missing directory is not an error")
                .is_none()
        );
    }

    #[test]
    fn newest_log_file_picks_the_latest_date_stamp() {
        let dir = fixture_dir("logs-latest-date");
        fs_err::create_dir_all(&dir).expect("create the fixture directory");
        for name in [
            "storekeeper.2026-08-19.log",
            "storekeeper.2026-08-21.log",
            "storekeeper.2026-08-20.log",
            "notes.txt",
        ] {
            fs_err::write(dir.join(name), "{}\n").expect("write the fixture file");
        }

        let newest = newest_log_file(&dir)
            .expect("the fixture directory is readable")
            .expect("a log file exists");

        assert_eq!(newest.file_name(), Some("storekeeper.2026-08-21.log"));
    }

    #[test]
    fn read_tail_returns_the_last_lines_the_appender_wrote() {
        let dir = fixture_dir("read-tail");
        let mut appender = build_appender(&dir).expect("build the appender");
        for index in 0..5 {
            writeln!(
                appender,
                r#"{{"level":"INFO","fields":{{"message":"{index}"}}}}"#
            )
            .expect("write a line");
        }
        appender.flush().expect("flush the appender");

        let tail = read_tail_from(&dir, TAIL_READ_BYTES, 2).expect("read the tail");

        assert_eq!(tail.len(), 2, "the request bounds how many entries return");
        assert!(
            tail.last()
                .is_some_and(|line| line.contains(r#""message":"4""#)),
            "the newest entry is last, got {tail:?}"
        );
    }

    #[test]
    fn read_tail_is_empty_before_any_log_is_written() {
        let dir = fixture_dir("read-tail-empty");

        assert!(
            read_tail_from(&dir, TAIL_READ_BYTES, 10)
                .expect("a missing directory is not an error")
                .is_empty()
        );
    }

    #[test]
    fn read_tail_keeps_an_entry_the_window_opens_exactly_on() {
        let dir = fixture_dir("read-tail-boundary");
        let mut appender = build_appender(&dir).expect("build the appender");
        writeln!(appender, "first").expect("write the first line");
        writeln!(appender, "second").expect("write the second line");
        appender.flush().expect("flush the appender");

        // The file is thirteen bytes and "second" starts at byte six, so a
        // seven-byte window opens exactly on that line boundary.
        let tail = read_tail_from(&dir, 7, 10).expect("read the tail");

        assert_eq!(tail, vec!["second"]);
    }

    #[test]
    fn tail_lines_returns_the_last_entries_in_order() {
        let window = "one\ntwo\nthree\nfour\n";

        assert_eq!(tail_lines(window, false, 2), vec!["three", "four"]);
    }

    #[test]
    fn tail_lines_drops_the_entry_the_window_starts_inside() {
        let window = "ncated\ntwo\nthree\n";

        assert_eq!(tail_lines(window, true, 10), vec!["two", "three"]);
    }

    #[test]
    fn tail_lines_caps_a_request_larger_than_the_file() {
        let window = "one\ntwo\n";

        assert_eq!(tail_lines(window, false, 100), vec!["one", "two"]);
    }
}
