use std::collections::HashMap;

use icu_plurals::{PluralCategory, PluralRules};

use super::store::Value;

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
        // Emit everything before the opening brace verbatim.
        result.push_str(&rest[..open]);
        let from_open = &rest[open..];

        if let Some(close) = find_matching_brace(from_open, 0) {
            let inner = &from_open[1..close];
            let formatted = format_placeholder(inner, &arg_map, plural_rules);
            result.push_str(&formatted);
            rest = &from_open[close + 1..];
        } else {
            // Unbalanced brace: emit it literally and continue scanning.
            result.push('{');
            rest = &from_open['{'.len_utf8()..];
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
    for (offset, ch) in s[start..].char_indices() {
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

    match parts.len() {
        // Simple: {name}
        1 => {
            let name = parts[0].trim();
            match args.get(name) {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                None => format!("{{{name}}}"),
            }
        }
        // Plural: {name, plural, one {..} other {..}}
        3 if parts[1].trim() == "plural" => {
            let name = parts[0].trim();
            let branches_str = parts[2].trim();

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

/// Selects the content for a plural branch like `one {# minute}` from the branches string.
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
        let keyword = &rest[..kw_end];
        rest = rest[kw_end..].trim_start();

        // The keyword must be followed by a braced block.
        if !rest.starts_with('{') {
            return None;
        }
        let close = find_matching_brace(rest, 0)?;
        let content = &rest[1..close];

        if keyword == category {
            return Some(content.to_string());
        }
        rest = &rest[close + 1..];
    }
}
