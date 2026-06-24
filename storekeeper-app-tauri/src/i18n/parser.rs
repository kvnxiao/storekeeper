use super::store::Value;
use icu_plurals::PluralCategory;
use icu_plurals::PluralRules;
use std::collections::HashMap;

/// Formats a message template with arguments.
///
/// Handles two patterns:
/// 1. `{name}` - simple variable substitution
/// 2. `{name, plural, one {...} other {...}}` - plural dispatch
pub(super) fn format_message(
    template: &str,
    args: &[(&str, Value)],
    plural_rules: Option<&PluralRules>,
) -> String {
    let arg_map: HashMap<&str, &Value> = args.iter().map(|(k, v)| (*k, v)).collect();
    let mut result = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(open) = rest.find('{') {
        // Offsets come from `find` / `find_matching_brace`, so they always land
        // on char boundaries; the `unwrap_or_default` fallbacks are unreachable.
        // Emit everything before the opening brace verbatim.
        result.push_str(rest.get(..open).unwrap_or_default());
        let from_open = rest.get(open..).unwrap_or_default();

        if let Some(close) = find_matching_brace(from_open, 0) {
            let inner = from_open.get(1..close).unwrap_or_default();
            let formatted = format_placeholder(inner, &arg_map, plural_rules);
            result.push_str(&formatted);
            rest = from_open.get(close + 1..).unwrap_or_default();
        } else {
            // Unbalanced brace: emit it literally and continue scanning.
            result.push('{');
            rest = from_open.get('{'.len_utf8()..).unwrap_or_default();
        }
    }

    result.push_str(rest);
    result
}

/// Finds the byte offset of the matching closing brace, respecting nesting.
///
/// `start` must be the byte offset of an opening `{` within `s`. Returns the
/// byte offset (relative to `s`) of the `}` that closes it.
fn find_matching_brace(s: &str, start: usize) -> Option<usize> {
    let mut depth: i32 = 0;
    for (offset, ch) in s.get(start..).unwrap_or_default().char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + offset);
                }
            }
            _ => {}
        }
    }
    None
}

/// Formats a single placeholder (content between outermost `{` and `}`).
fn format_placeholder(
    inner: &str,
    args: &HashMap<&str, &Value>,
    plural_rules: Option<&PluralRules>,
) -> String {
    let parts: Vec<&str> = inner.splitn(3, ',').collect();

    match parts.as_slice() {
        // Simple: {name}
        [name] => {
            let name = name.trim();
            match args.get(name) {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                None => format!("{{{name}}}"),
            }
        }
        // Plural: {name, plural, one {..} other {..}}
        [name, kind, branches_str] if kind.trim() == "plural" => {
            let name = name.trim();
            let branches_str = branches_str.trim();

            let count = match args.get(name) {
                Some(Value::Number(n)) => *n,
                _ => return format!("{{{inner}}}"),
            };

            let category_str = match plural_rules {
                Some(rules) => {
                    let category = rules.category_for(count);
                    match category {
                        PluralCategory::Zero => "zero",
                        PluralCategory::One => "one",
                        PluralCategory::Two => "two",
                        PluralCategory::Few => "few",
                        PluralCategory::Many => "many",
                        PluralCategory::Other => "other",
                    }
                }
                // Fallback to "other" when plural rules are unavailable
                None => "other",
            };

            // Try the specific category first, fall back to "other"
            let branch = select_plural_branch(branches_str, category_str)
                .or_else(|| select_plural_branch(branches_str, "other"))
                .unwrap_or_default();

            branch.replace('#', &count.to_string())
        }
        _ => format!("{{{inner}}}"),
    }
}

/// Selects the content for a plural branch like `one {# minute}` from the
/// branches string.
fn select_plural_branch(branches: &str, category: &str) -> Option<String> {
    // Walk "keyword {content}" pairs, handling nested braces in each block.
    let mut rest = branches;

    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            return None;
        }

        // Read the keyword: up to the next whitespace or opening brace.
        let kw_end = rest
            .find(|c: char| c.is_whitespace() || c == '{')
            .unwrap_or(rest.len());
        // `kw_end` / `close` are byte offsets on char boundaries, so the
        // `unwrap_or_default` fallbacks below are unreachable.
        let keyword = rest.get(..kw_end).unwrap_or_default();
        rest = rest.get(kw_end..).unwrap_or_default().trim_start();

        // The keyword must be followed by a braced block.
        if !rest.starts_with('{') {
            return None;
        }
        let close = find_matching_brace(rest, 0)?;
        let content = rest.get(1..close).unwrap_or_default();

        if keyword == category {
            return Some(content.to_string());
        }
        rest = rest.get(close + 1..).unwrap_or_default();
    }
}
