//! Known HoYoLab API retcodes and how a caller should treat each one.
//!
//! Codes and meanings follow the `thesadru/genshin.py` error table:
//! <https://github.com/thesadru/genshin.py/blob/master/genshin/errors.py>

/// How a caller should treat a HoYoLab retcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetcodeKind {
    /// Temporary throttle; retry with backoff.
    Cooldown,
    /// Throttle bounded by a quota rather than by time; retrying cannot clear
    /// it.
    RateLimited,
    /// The cookie is missing, expired, or bound to no HoYoLab account.
    Cookie,
    /// The account or its bound game account cannot be used.
    Account,
    /// The daily reward is already signed for today.
    AlreadyClaimed,
    /// Geetest challenge flow, including required, rejected, and waived
    /// challenges.
    Geetest,
    /// A code-redemption endpoint refused the request. A redemption cooldown
    /// uses this kind because retrying a redemption spends an attempt against
    /// the code.
    Redemption,
    /// The request was malformed, unsupported, or failed inside the server.
    Request,
}

/// A recognized retcode with its handling class and canonical meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retcode {
    /// The value the API returns in the `retcode` field.
    pub code: i32,
    /// How a caller should treat the code.
    pub kind: RetcodeKind,
    /// English meaning, which the API's own message does not always carry.
    pub meaning: &'static str,
}

const fn entry(code: i32, kind: RetcodeKind, meaning: &'static str) -> Retcode {
    Retcode {
        code,
        kind,
        meaning,
    }
}

/// Every retcode this client recognizes, grouped by the endpoint family that
/// returns it. Each code appears once, and [`lookup`] returns the first match.
const RETCODES: &[Retcode] = &[
    // HoYoLab community
    entry(-100, RetcodeKind::Cookie, "Cookies are not valid."),
    entry(-108, RetcodeKind::Request, "Invalid language."),
    entry(-110, RetcodeKind::Cooldown, "Visits too frequently."),
    entry(1028, RetcodeKind::Cooldown, "Visits too frequently."),
    entry(2010, RetcodeKind::Account, "Account is muted."),
    // Game record
    entry(10001, RetcodeKind::Cookie, "Cookies are not valid."),
    entry(-10001, RetcodeKind::Request, "Malformed request."),
    entry(
        -10002,
        RetcodeKind::Account,
        "No game account associated with cookies.",
    ),
    entry(
        10101,
        RetcodeKind::RateLimited,
        "Cannot get data for more than 30 accounts per cookie per day.",
    ),
    entry(10102, RetcodeKind::Account, "User's data is not public."),
    entry(
        10103,
        RetcodeKind::Cookie,
        "Cookies are valid but do not have a HoYoLab account bound to them.",
    ),
    entry(
        10104,
        RetcodeKind::Account,
        "Cannot view real-time notes of other users.",
    ),
    // Any endpoint
    entry(-1, RetcodeKind::Request, "Internal database error."),
    entry(10307, RetcodeKind::Request, "Internal database error."),
    entry(
        1009,
        RetcodeKind::Account,
        "Could not find user; uid may be invalid.",
    ),
    // Miyoushe
    entry(
        1008,
        RetcodeKind::Account,
        "Could not find user; uid may be invalid.",
    ),
    entry(
        -1104,
        RetcodeKind::Request,
        "This action must be done in the app.",
    ),
    // Calculator
    entry(
        -500_001,
        RetcodeKind::Request,
        "Invalid fields in calculation.",
    ),
    entry(-500_004, RetcodeKind::Cooldown, "Visits too frequently."),
    entry(
        -502_001,
        RetcodeKind::Request,
        "User does not have this character.",
    ),
    entry(
        -502_002,
        RetcodeKind::Account,
        "Calculator sync is not enabled.",
    ),
    // Code redemption
    entry(-1065, RetcodeKind::Redemption, "Invalid redemption code."),
    entry(-1071, RetcodeKind::Cookie, "Cookies are not valid."),
    entry(
        -1073,
        RetcodeKind::Account,
        "Account has no game account bound to it.",
    ),
    entry(
        -2001,
        RetcodeKind::Redemption,
        "Redemption code has expired.",
    ),
    entry(
        -2003,
        RetcodeKind::Redemption,
        "Redemption code is incorrectly formatted.",
    ),
    entry(-2004, RetcodeKind::Redemption, "Invalid redemption code."),
    entry(
        -2006,
        RetcodeKind::Redemption,
        "Redemption code has reached max usage limit.",
    ),
    entry(
        -2008,
        RetcodeKind::Redemption,
        "Redemption code is not available in the account's region.",
    ),
    entry(
        -2011,
        RetcodeKind::Redemption,
        "Redemption code cannot be claimed because the game level is too low.",
    ),
    entry(
        -2014,
        RetcodeKind::Redemption,
        "Redemption code not activated.",
    ),
    entry(-2016, RetcodeKind::Redemption, "Redemption is on cooldown."),
    entry(
        -2017,
        RetcodeKind::Redemption,
        "Redemption code has been claimed already.",
    ),
    entry(
        -2018,
        RetcodeKind::Redemption,
        "Redemption code has been claimed already.",
    ),
    entry(
        -2021,
        RetcodeKind::Redemption,
        "Redemption code cannot be claimed because the game level is too low.",
    ),
    // Daily reward
    entry(
        -5003,
        RetcodeKind::AlreadyClaimed,
        "Already claimed the daily reward today.",
    ),
    // Account and login
    entry(-3004, RetcodeKind::Account, "Account login failed."),
    entry(-3208, RetcodeKind::Account, "Account login failed."),
    entry(
        -3202,
        RetcodeKind::Account,
        "Account is locked after too many password attempts; retry in 20 minutes.",
    ),
    entry(-3203, RetcodeKind::Account, "Account does not exist."),
    entry(
        -3205,
        RetcodeKind::Account,
        "The provided OTP code is wrong.",
    ),
    entry(
        -3206,
        RetcodeKind::RateLimited,
        "Too many verification code requests for the account.",
    ),
    entry(-216, RetcodeKind::Account, "Game account is incorrect."),
    entry(-202, RetcodeKind::Account, "Game password is incorrect."),
    // Miyoushe OTP
    entry(
        -119,
        RetcodeKind::RateLimited,
        "Too many OTP messages sent for the number; the cap is 40 per day.",
    ),
    entry(
        -3006,
        RetcodeKind::RateLimited,
        "Request too frequent (OTP endpoint).",
    ),
    // Lineup
    entry(-1004, RetcodeKind::Cooldown, "Action is in cooldown."),
    entry(-3101, RetcodeKind::Cooldown, "Action is in cooldown."),
    // Geetest
    entry(
        -3102,
        RetcodeKind::Geetest,
        "The provided aigis payload is invalid.",
    ),
    entry(30001, RetcodeKind::Geetest, "No need to do geetest."),
];

/// Looks up a retcode, returning `None` when the catalogue does not list it.
#[must_use]
pub fn lookup(code: i32) -> Option<&'static Retcode> {
    RETCODES.iter().find(|retcode| retcode.code == code)
}

/// Returns whether the catalogue classifies `code` as `kind`.
#[must_use]
pub fn has_kind(code: i32, kind: RetcodeKind) -> bool {
    lookup(code).is_some_and(|retcode| retcode.kind == kind)
}

/// Returns whether a retcode is classified as a temporary throttle.
#[must_use]
pub fn is_transient_throttle(code: i32) -> bool {
    has_kind(code, RetcodeKind::Cooldown)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn the_catalogue_lists_each_code_once() {
        let mut seen = HashSet::new();

        for retcode in RETCODES {
            assert!(
                seen.insert(retcode.code),
                "retcode {} appears more than once",
                retcode.code
            );
        }
    }

    #[test]
    fn lookup_maps_a_code_to_each_kind() {
        for (code, kind) in [
            (-1004, RetcodeKind::Cooldown),
            (-110, RetcodeKind::Cooldown),
            (1028, RetcodeKind::Cooldown),
            (10101, RetcodeKind::RateLimited),
            (-100, RetcodeKind::Cookie),
            (-10002, RetcodeKind::Account),
            (-5003, RetcodeKind::AlreadyClaimed),
            (-3102, RetcodeKind::Geetest),
            (-2016, RetcodeKind::Redemption),
            (-10001, RetcodeKind::Request),
        ] {
            assert_eq!(
                lookup(code).map(|retcode| retcode.kind),
                Some(kind),
                "retcode {code}"
            );
        }
    }

    #[test]
    fn lookup_is_none_for_a_code_outside_the_catalogue() {
        assert_eq!(lookup(-1002), None);
    }

    #[test]
    fn only_a_time_bounded_throttle_is_transient() {
        for code in [-1004, -3101, -110, 1028, -500_004] {
            assert!(
                is_transient_throttle(code),
                "retcode {code} is a temporary throttle"
            );
        }

        for code in [10101, -119, -3206, -3006, -2016, -5003, -100, -1002] {
            assert!(
                !is_transient_throttle(code),
                "retcode {code} is not a temporary throttle"
            );
        }
    }
}
