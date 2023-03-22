use crate::html_search::{search_html, Builder};

#[test]
fn test_search() {
    let result = Builder::new()
        .search_term(Some("apple"))
        .search(
            r#"<h2>Heading</h2><p>Nobody likes maple in their apple flavoured Snapple. APPLE</p>"#,
        )
        .to_string();
    let expected = r#"<h2>Heading</h2><p>Nobody likes maple in their <mark id="search-match">apple</mark> flavoured Sn<mark>apple</mark>. <mark>APPLE</mark></p>"#;
    assert_eq!(result, expected);
}

#[test]
fn search_html_highlight_requested_term() {
    let result = search_html(
        r#"<h2>Heading</h2><p>Nobody likes maple in their apple flavoured Snapple. APPLE</p><p>Paragraph with no matches</p><p>Paragraph which mentions apples again</p>"#,
        "apple",
    )
    .to_string();
    let expected = r#"<h2>Heading</h2><p>Nobody likes maple in their <mark id="search-match">apple</mark> flavoured Sn<mark>apple</mark>. <mark>APPLE</mark></p><p>Paragraph with no matches</p><p>Paragraph which mentions <mark>apple</mark>s again</p>"#;
    assert_eq!(result, expected);
}

#[test]
fn search_html_highlight_does_nothing_when_there_are_no_matches() {
    let result = search_html(
        r#"<h2>Heading</h2><p>Nobody likes maple in their apple flavoured Snapple. APPLE</p>"#,
        "nonsense",
    )
    .to_string();
    let expected =
        r#"<h2>Heading</h2><p>Nobody likes maple in their apple flavoured Snapple. APPLE</p>"#;
    assert_eq!(result, expected);
}

#[test]
fn search_html_highlight_highlights_nested_matches() {
    let result = search_html(
        r#"<h2>Heading</h2><p>Nobody likes maple in their <strong>apple</strong> flavoured Snapple. APPLE</p><p>Paragraph with no matches</p><p>Paragraph which mentions apples again</p>"#,
        "apple",
    )
    .to_string();
    //     let result = search_html(
    // r#"<p>If you have followed me for a while, you will know I am a huge vanilla-extract
    // fan. It has a lot of the features of Tailwind and in fact many of the benefits
    // of CSS in JS frameworks. However, in my opinion, two factors make it worth
    // considering vanilla-extract as an alternative: <strong>TypeScript</strong> and <strong>zero-runtime
    // overhead</strong>. You write your styles in TypeScript, so get the in-editor
    // <strong><abbr>DX<tool-tip inert="" role="tooltip">Developer Experience</tool-tip></abbr>
    // benefits</strong> of Intellisense, checking and completion. The bonus is that the
    // tooling (works with <strong>Vite</strong>, <strong>ESBuild</strong> and WebPack) transpiles the styles,
    // and you only ship CSS to the browser — <strong>no
    // <abbr>UX<tool-tip inert="" role="tooltip">User Experience</tool-tip></abbr>
    // performance&nbsp;hit</strong>.</p>"#,"TypeScript").to_string();

    let expected = r#"<h2>Heading</h2><p>Nobody likes maple in their <strong><mark id="search-match">apple</mark></strong> flavoured Sn<mark>apple</mark>. <mark>APPLE</mark></p><p>Paragraph with no matches</p><p>Paragraph which mentions <mark>apple</mark>s again</p>"#;
    assert_eq!(result, expected);
}
