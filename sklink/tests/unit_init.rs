use sklink::init::render_default_config;

#[test]
fn render_default_config_contains_default_platforms() {
    let cfg = render_default_config();
    assert!(cfg.contains(r#"[platforms.kimi]"#));
    assert!(cfg.contains(r#"{ dir = "~/.kimi/skills" }"#));
    assert!(cfg.contains(r#"[platforms.trae]"#));
    assert!(cfg.contains(r#"{ dir = "~/.trae/skills" }"#));
}
