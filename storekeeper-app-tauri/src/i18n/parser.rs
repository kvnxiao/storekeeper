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
    let chars: Vec<char> = template.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '{' {
            if let Some(close) = find_matching_brace(&chars, i) {
                let inner: String = chars[i + 1..close].iter().collect();
                let formatted = format_placeholder(&inner, &arg_map, plural_rules);
                result.push_str(&formatted);
                i = close + 1;
            } else {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Finds the matching closing brace, respecting nested braces.
fn find_matching_brace(chars: &[char], start: usize) -> Option<usize> {
    let mut depth = 0;
    for (idx, &ch) in chars.iter().enumerate().skip(start) {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
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
    // Find "category {content}" pattern, handling nested braces
    let chars: Vec<char> = branches.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        // Read keyword
        let kw_start = i;
        while i < len && !chars[i].is_whitespace() && chars[i] != '{' {
            i += 1;
        }
        let keyword: String = chars[kw_start..i].iter().collect();

        // Skip whitespace before '{'
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        // Read braced content
        if i < len && chars[i] == '{' {
            if let Some(close) = find_matching_brace(&chars, i) {
                let content: String = chars[i + 1..close].iter().collect();
                if keyword == category {
                    return Some(content);
                }
                i = close + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    None
}
