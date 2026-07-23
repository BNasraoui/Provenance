use super::*;

#[track_caller]
fn css_rule(selector: &str) -> &'static str {
    let prefix = format!("{selector} {{");
    let declarations = WIKI_CSS
        .split_once(&prefix)
        .unwrap_or_else(|| panic!("missing CSS rule: {selector}"))
        .1;

    declarations
        .split_once('}')
        .unwrap_or_else(|| panic!("unterminated CSS rule: {selector}"))
        .0
}

#[test]
fn css_carries_all_five_theme_token_blocks() {
    for theme in ["statesman", "piano", "latte", "mocha", "dracula"] {
        assert!(
            WIKI_CSS.contains(&format!("[data-theme=\"{theme}\"]")),
            "missing theme block: {theme}"
        );
    }
}

#[test]
fn css_defines_the_atlas_type_tokens() {
    for token in [
        "--pv-source:",
        "--pv-requirement:",
        "--pv-resolution:",
        "--pv-rule:",
        "--pv-thread:",
    ] {
        assert!(WIKI_CSS.contains(token), "missing token: {token}");
    }
}

#[test]
fn css_keeps_the_mockup_grammar() {
    for selector in [
        ".accent-bar",
        ".chrome-inner",
        ".body-grid",
        ".body-margin",
        ".territory-card",
        ".citation.gap",
        ".field-notes",
        ".role-badge",
        ".lineage",
    ] {
        assert!(WIKI_CSS.contains(selector), "missing selector: {selector}");
    }
}

#[test]
fn css_colors_traversal_links_with_theme_tokens() {
    let link_color = declaration_value(WIKI_CSS, ".link-list a", "color")
        .expect(".link-list a should declare its link color");
    assert!(
        link_color.contains("var(--pv-"),
        ".link-list a should use a --pv custom property for color, got {link_color}"
    );

    for (section_class, token) in [
        ("sh-requirement", "--pv-requirement"),
        ("sh-resolution", "--pv-resolution"),
        ("sh-rule", "--pv-rule"),
        ("sh-source", "--pv-source"),
    ] {
        let selector = format!("section:has(> .{section_class})");
        let link_color_override = declaration_value(WIKI_CSS, &selector, "--pv-link-color")
            .unwrap_or_else(|| panic!("{selector} should override --pv-link-color"));
        assert_eq!(link_color_override, format!("var({token})"));
    }
}

#[test]
fn light_theme_link_tokens_meet_body_text_contrast() {
    for (theme, selector) in [
        ("statesman", r#":root, :root[data-theme="statesman"]"#),
        ("piano", r#":root[data-theme="piano"]"#),
        ("latte", r#":root[data-theme="latte"]"#),
    ] {
        let background = declaration_value(WIKI_CSS, selector, "--pv-canvas-bg")
            .unwrap_or_else(|| panic!("{theme} should declare --pv-canvas-bg"));
        for token in [
            "--pv-source",
            "--pv-requirement",
            "--pv-resolution",
            "--pv-rule",
        ] {
            let foreground = declaration_value(WIKI_CSS, selector, token)
                .unwrap_or_else(|| panic!("{theme} should declare {token}"));
            let contrast = contrast_ratio(foreground, background)
                .expect("light theme link tokens should be hex colors");
            assert!(
                contrast >= 4.5,
                "{theme} {token} contrast against {background} is {contrast:.2}:1"
            );
        }
    }
}

fn declaration_value<'a>(css: &'a str, selector: &str, property: &str) -> Option<&'a str> {
    let selector_start = css.find(selector)?;
    let after_selector = &css[selector_start + selector.len()..];
    let block_start = after_selector.find('{')?;
    let block = &after_selector[block_start + 1..];
    let block_end = block.find('}')?;

    block[..block_end].split(';').find_map(|declaration| {
        let (name, value) = declaration.split_once(':')?;
        (name.trim() == property).then_some(value.trim())
    })
}

fn contrast_ratio(foreground: &str, background: &str) -> Option<f64> {
    let foreground_luminance = relative_luminance(hex_rgb(foreground)?);
    let background_luminance = relative_luminance(hex_rgb(background)?);
    let lighter = foreground_luminance.max(background_luminance);
    let darker = foreground_luminance.min(background_luminance);

    Some((lighter + 0.05) / (darker + 0.05))
}

fn relative_luminance([red, green, blue]: [u8; 3]) -> f64 {
    let red = linearized_srgb(red);
    let green = linearized_srgb(green);
    let blue = linearized_srgb(blue);

    0.0722_f64.mul_add(blue, 0.2126_f64.mul_add(red, 0.7152 * green))
}

fn linearized_srgb(channel: u8) -> f64 {
    let normalized = f64::from(channel) / 255.0;
    if normalized <= 0.040_45 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

fn hex_rgb(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }

    Some([
        u8::from_str_radix(hex.get(0..2)?, 16).ok()?,
        u8::from_str_radix(hex.get(2..4)?, 16).ok()?,
        u8::from_str_radix(hex.get(4..6)?, 16).ok()?,
    ])
}

#[test]
fn css_allows_deep_breadcrumbs_to_wrap_in_chrome() {
    assert!(
        css_rule(".chrome-inner").contains("align-items: flex-start;"),
        "chrome header should align cleanly when breadcrumbs wrap"
    );
    assert!(
        WIKI_CSS.contains(".chrome nav { display: flex; flex: 1 1 auto; min-width: 0;"),
        "breadcrumb nav should be shrink-safe inside the chrome flex row"
    );
    assert!(
        css_rule(".chrome nav a").contains("overflow-wrap: anywhere;"),
        "breadcrumb links should not force horizontal overflow"
    );
}

#[test]
fn css_wraps_long_classification_values_in_the_margin() {
    let value_rule = css_rule(".classification .v");
    assert!(
        value_rule.contains("min-width: 0;"),
        "classification values should be shrink-safe inside their flex row"
    );
    assert!(
        value_rule.contains("overflow-wrap: anywhere;"),
        "classification values should wrap instead of widening the fixed margin"
    );
    assert!(
        css_rule(".classification .v.mono").contains("word-break: break-all;"),
        "mono classification values should wrap long document paths like citation references"
    );
}

#[test]
fn css_makes_no_external_requests() {
    assert!(!WIKI_CSS.contains("http://"));
    assert!(!WIKI_CSS.contains("https://"));
    assert!(!WIKI_CSS.contains("@import"));
    assert!(!WIKI_CSS.contains("url("));
}

#[test]
fn theme_script_persists_the_choice_to_local_storage() {
    assert!(THEME_SCRIPT.contains("provenance-wiki-theme"));
    assert!(THEME_SCRIPT.contains("localStorage"));
    assert!(THEME_SCRIPT.contains("data-theme"));
}

#[test]
fn search_script_filters_rendered_nodes_without_html_injection_or_fetching() {
    assert!(SEARCH_SCRIPT.contains("data-search-title"));
    assert!(SEARCH_SCRIPT.contains("data-search-statement"));
    assert!(SEARCH_SCRIPT.contains("terms.every"));
    assert!(SEARCH_SCRIPT.contains("textContent"));
    assert!(!SEARCH_SCRIPT.contains("innerHTML"));
    assert!(!SEARCH_SCRIPT.contains("fetch("));
}

#[test]
fn css_styles_domain_navigation_and_search_responsively() {
    for selector in [
        ".global-nav",
        ".domain-group",
        ".domain-records",
        ".search-box",
        ".search-results",
        ".data-note",
    ] {
        assert!(WIKI_CSS.contains(selector), "missing selector: {selector}");
    }
    assert!(WIKI_CSS.contains("@media (max-width: 760px)"));
}

#[test]
fn icon_defs_cover_the_symbols_the_renderer_uses() {
    for symbol in [
        "i-git-branch",
        "i-scale",
        "i-book-open",
        "i-search",
        "i-check-circle",
        "i-arrow-left",
        "i-message-square",
        "i-user",
        "i-calendar",
        "i-shield",
        "i-gauge",
        "i-bot",
    ] {
        assert!(
            ICON_DEFS.contains(&format!("id=\"{symbol}\"")),
            "missing icon: {symbol}"
        );
    }
}
