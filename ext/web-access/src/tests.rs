use super::MyWebAccess;

#[test]
fn test_clean_html_basic() {
    let html = "<html><head><title>Test</title></head><body><h1>Hello World</h1><p>This is a test.</p></body></html>";
    let cleaned = MyWebAccess::clean_html(html);
    assert_eq!(cleaned, "Hello World This is a test.");
}

#[test]
fn test_clean_html_skips_scripts_and_styles() {
    let html =
        "<div>Text <script>alert(1);</script> <style>body { color: red; }</style> visible</div>";
    let cleaned = MyWebAccess::clean_html(html);
    assert_eq!(cleaned, "Text visible");
}

#[test]
fn test_clean_html_complex_nesting() {
    let html = "<nav><ul><li>Home</li></ul></nav><article><h1>Article</h1><p>Paragraph</p></article><footer>Footer info</footer>";
    let cleaned = MyWebAccess::clean_html(html);
    assert_eq!(cleaned, "Article Paragraph");
}
