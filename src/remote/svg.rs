use std::collections::HashMap;

/// Extracts CSS custom property definitions from the light-mode block of an SVG's <style> tag
/// and replaces all `var(--name)` references with their concrete values.
///
/// Typst's SVG renderer does not support CSS custom properties (var()), so this function
/// inlines the variable values directly into the SVG attributes and style rules.
pub fn resolve_css_variables(svg: &str) -> String {
    let variables = extract_css_variables(svg);
    if variables.is_empty() {
        return svg.to_string();
    }

    let mut result = svg.to_string();

    // Replace var(--name) with the concrete value
    for (name, value) in &variables {
        let pattern = format!("var(--{})", name);
        result = result.replace(&pattern, value);
    }

    // Remove the <style> block entirely since all variables are now resolved
    // and Typst doesn't need the CSS rules (animations, hover effects, etc. don't work in PDF)
    result = remove_style_block(&result);

    result
}

/// Extracts CSS custom properties from the light-mode (non-dark) block in the SVG's <style> tag.
///
/// The SVG defines variables on the root element ID selector, then overrides them in
/// @media (prefers-color-scheme: dark) and .dark-mode blocks. We only want the light-mode
/// values since PDF output is static.
fn extract_css_variables(svg: &str) -> HashMap<String, String> {
    let style_block = match extract_style_block(svg) {
        Some(block) => block,
        None => return HashMap::new(),
    };

    let mut variables = HashMap::new();

    let lines: Vec<&str> = style_block.lines().collect();
    let mut in_dark_block = false;
    let mut brace_depth = 0;

    for line in lines {
        let trimmed = line.trim();

        // Detect @media (prefers-color-scheme: dark) block start
        if trimmed.contains("@media") && trimmed.contains("dark") {
            in_dark_block = true;
            if trimmed.contains('{') {
                brace_depth += trimmed.chars().filter(|&c| c == '{').count();
            }
            if trimmed.contains('}') {
                brace_depth -= trimmed.chars().filter(|&c| c == '}').count();
            }
            continue;
        }

        // Detect .dark-mode block start
        if trimmed.contains(".dark-mode") {
            in_dark_block = true;
            if trimmed.contains('{') {
                brace_depth += trimmed.chars().filter(|&c| c == '{').count();
            }
            if trimmed.contains('}') {
                brace_depth -= trimmed.chars().filter(|&c| c == '}').count();
            }
            continue;
        }

        // Track brace depth for dark blocks
        if in_dark_block {
            brace_depth += trimmed.chars().filter(|&c| c == '{').count();
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count();
            if brace_depth <= 0 {
                in_dark_block = false;
                brace_depth = 0;
            }
            continue;
        }

        // Parse CSS variable definitions (e.g., --bar-blue-0: #4F7FAE;)
        if trimmed.starts_with("--") {
            if let Some(colon_pos) = trimmed.find(':') {
                let name = trimmed[..colon_pos].trim().trim_start_matches('-').to_string();
                let value_part = trimmed[colon_pos + 1..].trim();
                // Remove trailing semicolon
                let value = value_part.trim_end_matches(';').trim().to_string();
                if !name.is_empty() && !value.is_empty() {
                    variables.insert(name, value);
                }
            }
        }
    }

    variables
}

/// Extracts the content between <style> and </style> tags.
fn extract_style_block(svg: &str) -> Option<String> {
    let start = svg.find("<style>")?;
    let end = svg.find("</style>")?;
    Some(svg[start + 8..end].to_string())
}

/// Removes the entire <style>...</style> block from the SVG.
fn remove_style_block(svg: &str) -> String {
    if let Some(start) = svg.find("<style>") {
        if let Some(end) = svg[start..].find("</style>") {
            let end_pos = start + end + "</style>".len();
            return format!("{}{}", &svg[..start], &svg[end_pos..]);
        }
    }
    svg.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_css_variables() {
        let svg = r#"<svg>
<style>
#root {
    --bg: #F6F8FB;
    --text: #17202A;
}
@media (prefers-color-scheme: dark) {
    #root {
        --bg: #111827;
        --text: #F9FAFB;
    }
}
</style>
</svg>"#;

        let vars = extract_css_variables(svg);
        assert_eq!(vars.get("bg"), Some(&"#F6F8FB".to_string()));
        assert_eq!(vars.get("text"), Some(&"#17202A".to_string()));
        // Dark mode values should NOT be present
        assert_ne!(vars.get("bg"), Some(&"#111827".to_string()));
    }

    #[test]
    fn test_resolve_css_variables() {
        let svg = r##"<svg>
<style>
#root {
    --bg: #F6F8FB;
    --text: #17202A;
}
</style>
<rect fill="var(--bg)"/>
<text fill="var(--text)">Hello</text>
</svg>"##;

        let result = resolve_css_variables(svg);
        assert!(result.contains("fill=\"#F6F8FB\""));
        assert!(result.contains("fill=\"#17202A\""));
        assert!(!result.contains("var("));
        assert!(!result.contains("<style>"));
    }

    #[test]
    fn test_resolve_css_variables_no_style() {
        let svg = r#"<svg><rect fill="red"/></svg>"#;
        let result = resolve_css_variables(svg);
        assert_eq!(result, svg);
    }

    #[test]
    fn test_resolve_css_variables_with_dark_mode() {
        let svg = r#"<svg>
<style>
#root {
    --bg: #FFFFFF;
}
@media (prefers-color-scheme: dark) {
    #root {
        --bg: #000000;
    }
}
#root.dark-mode {
    --bg: #111111;
}
</style>
<rect fill="var(--bg)"/>
</svg>"#;

        let result = resolve_css_variables(svg);
        assert!(result.contains("fill=\"#FFFFFF\""));
        assert!(!result.contains("var("));
    }

    #[test]
    fn test_remove_style_block() {
        let svg = r#"<svg><defs></defs><style>.foo{}</style><rect/></svg>"#;
        let result = remove_style_block(svg);
        assert_eq!(result, r#"<svg><defs></defs><rect/></svg>"#);
    }
}
