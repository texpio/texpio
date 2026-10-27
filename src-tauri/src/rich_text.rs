//! A bounded Tiptap document subset. HTML is generated, never trusted from input.
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashSet;

const MAX_DOCUMENT_BYTES: usize = crate::library::MAX_RICH_BODY_BYTES;
const MAX_HTML_BYTES: usize = 4 * 1024 * 1024;
const MAX_NODES: usize = 100_000;
const MAX_DEPTH: usize = 24;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    doc: Node,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Node {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    content: Vec<Node>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    marks: Vec<Mark>,
    #[serde(default)]
    attrs: Option<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mark {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    attrs: Option<Value>,
}

pub(crate) struct RenderedDocument {
    pub plain: String,
    pub html: String,
}

pub(crate) fn render(json: &str) -> Result<RenderedDocument, ()> {
    if json.len() > MAX_DOCUMENT_BYTES {
        return Err(());
    }
    let envelope: Envelope = serde_json::from_str(json).map_err(|_| ())?;
    if envelope.version != 1 || envelope.doc.kind != "doc" {
        return Err(());
    }
    let mut count = 0;
    let (plain, html) = blocks(&envelope.doc, 0, &mut count)?;
    Ok(RenderedDocument {
        plain,
        html: format!("<div style=\"white-space:pre-wrap\">{html}</div>"),
    })
}

fn normal_container(node: &Node) -> Result<(), ()> {
    if node.text.is_some() || !node.marks.is_empty() || node.attrs.is_some() {
        return Err(());
    }
    Ok(())
}

fn count_node(depth: usize, count: &mut usize) -> Result<(), ()> {
    *count += 1;
    if depth > MAX_DEPTH || *count > MAX_NODES {
        Err(())
    } else {
        Ok(())
    }
}

fn blocks(node: &Node, depth: usize, count: &mut usize) -> Result<(String, String), ()> {
    count_node(depth, count)?;
    normal_container(node)?;
    let mut plain = Vec::new();
    let mut html = String::new();
    for child in &node.content {
        let (text, output) = block(child, depth + 1, count)?;
        plain.push(text);
        append_html(&mut html, &output)?;
    }
    Ok((plain.join("\n"), html))
}

fn block(node: &Node, depth: usize, count: &mut usize) -> Result<(String, String), ()> {
    count_node(depth, count)?;
    if node.text.is_some() || !node.marks.is_empty() {
        return Err(());
    }
    match node.kind.as_str() {
        "paragraph" => {
            if node.attrs.is_some() {
                return Err(());
            }
            let mut plain = String::new();
            let mut html = String::from("<p dir=\"auto\">");
            for child in &node.content {
                count_node(depth + 1, count)?;
                match child.kind.as_str() {
                    "hardBreak" => {
                        if child.text.is_some()
                            || !child.content.is_empty()
                            || child.attrs.is_some()
                        {
                            return Err(());
                        }
                        plain.push('\n');
                        append_html(&mut html, &render_marks(&child.marks, "<br>".to_owned())?)?;
                    }
                    "text" => {
                        if !child.content.is_empty() || child.attrs.is_some() {
                            return Err(());
                        }
                        let text = child.text.as_ref().filter(|s| !s.is_empty()).ok_or(())?;
                        if text
                            .chars()
                            .any(|c| c.is_control() && !matches!(c, '\t' | '\n'))
                        {
                            return Err(());
                        }
                        plain.push_str(text);
                        append_html(&mut html, &render_marks(&child.marks, escape(text))?)?;
                    }
                    _ => return Err(()),
                }
            }
            html.push_str("</p>");
            Ok((plain, html))
        }
        "bulletList" | "orderedList" => {
            let ordered = node.kind == "orderedList";
            let (start, list_type) = if ordered {
                ordered_attributes(node.attrs.as_ref())?
            } else {
                if node.attrs.is_some() {
                    return Err(());
                }
                (1, None)
            };
            if node.content.is_empty() {
                return Err(());
            }
            let mut lines = Vec::new();
            let mut html = if ordered {
                match list_type {
                    Some(kind) => format!("<ol start=\"{start}\" type=\"{kind}\">"),
                    None => format!("<ol start=\"{start}\">"),
                }
            } else {
                "<ul>".to_owned()
            };
            for (index, item) in node.content.iter().enumerate() {
                if item.kind != "listItem"
                    || item.content.is_empty()
                    || item.content[0].kind != "paragraph"
                {
                    return Err(());
                }
                let (text, item_html) = blocks(item, depth + 1, count)?;
                let prefix = if ordered {
                    format!("{}. ", start + index as i64)
                } else {
                    "- ".to_owned()
                };
                let mut split = text.split('\n');
                let mut line = format!("{prefix}{}", split.next().unwrap_or(""));
                for part in split {
                    line.push_str("\n  ");
                    line.push_str(part);
                }
                lines.push(line);
                append_html(&mut html, "<li dir=\"auto\">")?;
                append_html(&mut html, &item_html)?;
                append_html(&mut html, "</li>")?;
            }
            html.push_str(if ordered { "</ol>" } else { "</ul>" });
            Ok((lines.join("\n"), html))
        }
        _ => Err(()),
    }
}

fn ordered_attributes(attrs: Option<&Value>) -> Result<(i64, Option<&str>), ()> {
    let Some(attrs) = attrs else {
        return Ok((1, None));
    };
    let attrs = attrs.as_object().ok_or(())?;
    if attrs.keys().any(|key| key != "start" && key != "type") {
        return Err(());
    }
    let list_type = match attrs.get("type") {
        None | Some(Value::Null) => None,
        Some(value) => match value.as_str() {
            Some(kind @ ("1" | "a" | "A" | "i" | "I")) => Some(kind),
            _ => return Err(()),
        },
    };
    let start = match attrs.get("start") {
        None => 1,
        Some(value) => value.as_i64().ok_or(())?,
    };
    if !(-1_000_000..=1_000_000).contains(&start) {
        return Err(());
    }
    Ok((start, list_type))
}

fn render_marks(marks: &[Mark], mut fragment: String) -> Result<String, ()> {
    let mut seen = HashSet::new();
    for mark in marks {
        if !seen.insert(mark.kind.as_str()) {
            return Err(());
        }
        let (start, end) = match mark.kind.as_str() {
            "bold" if mark.attrs.is_none() => ("<strong>".to_owned(), "</strong>"),
            "italic" if mark.attrs.is_none() => ("<em>".to_owned(), "</em>"),
            "underline" if mark.attrs.is_none() => ("<u>".to_owned(), "</u>"),
            "link" => (
                format!("<a href=\"{}\">", escape(safe_href(mark.attrs.as_ref())?)),
                "</a>",
            ),
            _ => return Err(()),
        };
        fragment = format!("{start}{fragment}{end}");
        if fragment.len() > MAX_HTML_BYTES {
            return Err(());
        }
    }
    Ok(fragment)
}

fn safe_href(attrs: Option<&Value>) -> Result<&str, ()> {
    let attrs = attrs.and_then(Value::as_object).ok_or(())?;
    if attrs
        .keys()
        .any(|key| !["href", "target", "rel", "class", "title"].contains(&key.as_str()))
    {
        return Err(());
    }
    let href = attrs.get("href").and_then(Value::as_str).ok_or(())?;
    if href.len() > 4096
        || href
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control() || ch == '\u{feff}')
    {
        return Err(());
    }
    let lower = href.to_ascii_lowercase();
    if !(lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:"))
    {
        return Err(());
    }
    Ok(href)
}

fn append_html(destination: &mut String, value: &str) -> Result<(), ()> {
    if destination.len().saturating_add(value.len()) > MAX_HTML_BYTES {
        return Err(());
    }
    destination.push_str(value);
    Ok(())
}

fn escape(value: &str) -> String {
    let mut result = String::new();
    for ch in value.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(ch),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_and_escapes_formatting_and_unicode() {
        let json = r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hi <世界>","marks":[{"type":"bold"}]},{"type":"hardBreak"},{"type":"text","text":"line\t2"}]}]}}"#;
        let rendered = render(json).unwrap();
        assert_eq!(rendered.plain, "Hi <世界>\nline\t2");
        assert!(rendered.html.contains("<strong>Hi &lt;世界&gt;</strong>"));
        assert!(rendered.html.contains("<br>"));
    }

    #[test]
    fn projects_ordered_and_nested_lists() {
        let json = r#"{"version":1,"doc":{"type":"doc","content":[{"type":"orderedList","attrs":{"start":3,"type":null},"content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"one"}]},{"type":"bulletList","content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"nested"}]}]}]}]}]}]}}"#;
        let rendered = render(json).unwrap();
        assert_eq!(rendered.plain, "3. one\n  - nested");
        assert!(rendered.html.contains("<ol start=\"3\">"));
        assert!(rendered.html.contains("<ul>"));
    }

    #[test]
    fn accepts_valid_marks_on_hard_breaks_and_escapes_links() {
        let json = r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"a"},{"type":"hardBreak","marks":[{"type":"bold"},{"type":"link","attrs":{"href":"https://example.com/?a=1&b=2"}}]},{"type":"text","text":"b"}]}]}}"#;
        let rendered = render(json).unwrap();
        assert_eq!(rendered.plain, "a\nb");
        assert!(
            rendered
                .html
                .contains("<a href=\"https://example.com/?a=1&amp;b=2\"><strong><br></strong></a>")
        );
        assert!(
            render(&json.replace("https://example.com/?a=1&b=2", "javascript:alert(1)")).is_err()
        );
    }

    #[test]
    fn renders_signed_ordered_starts_and_safe_html_list_types_with_auto_direction() {
        let json = r#"{"version":1,"doc":{"type":"doc","content":[{"type":"orderedList","attrs":{"start":-1,"type":"a"},"content":[{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"hello"}]}]},{"type":"listItem","content":[{"type":"paragraph","content":[{"type":"text","text":"مرحبا"}]}]}]}]}}"#;
        let rendered = render(json).unwrap();
        assert_eq!(rendered.plain, "-1. hello\n0. مرحبا");
        assert!(rendered.html.contains("<ol start=\"-1\" type=\"a\">"));
        assert!(rendered.html.contains("<li dir=\"auto\"><p dir=\"auto\">"));
        for kind in ["1", "A", "i", "I"] {
            assert!(
                render(&json.replace("\"type\":\"a\"", &format!("\"type\":\"{kind}\""))).is_ok()
            );
        }
        assert!(render(&json.replace("\"type\":\"a\"", "\"type\":\"unsafe\"")).is_err());
        assert!(render(&json.replace("\"start\":-1", "\"start\":1000001")).is_err());
        assert!(render(&json.replace("\"start\":-1", "\"start\":-1000001")).is_err());
    }

    #[test]
    fn rejects_unsafe_links_unknown_nodes_and_invalid_versions() {
        for json in [
            r#"{"version":2,"doc":{"type":"doc","content":[]}}"#,
            r#"{"version":1,"doc":{"type":"doc","content":[{"type":"image","attrs":{"src":"https://x"}}]}}"#,
            r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"x","marks":[{"type":"link","attrs":{"href":"javascript:alert(1)"}}]}]}]}}"#,
        ] {
            assert!(render(json).is_err());
        }
    }

    #[test]
    fn refuses_carriage_returns_and_html_amplification() {
        let with_cr = serde_json::json!({"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"a\rb"}]}]}}).to_string();
        assert!(render(&with_cr).is_err());
        let href = format!("https://{}", "&".repeat(4000));
        let nodes: Vec<_> = (0..500).map(|_| serde_json::json!({"type":"text","text":"x","marks":[{"type":"link","attrs":{"href":href}}]})).collect();
        let amplified = serde_json::json!({"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":nodes}]}}).to_string();
        assert!(amplified.len() < MAX_DOCUMENT_BYTES);
        assert!(render(&amplified).is_err());
    }
}
