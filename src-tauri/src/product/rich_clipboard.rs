//! Prepare both formats without accessing the system clipboard.
use super::effects::ProductError;

const MAX_FORMAT_BYTES: usize = 8 * 1024 * 1024;
const PREFIX: &str = "<html><body><!--StartFragment-->";
const SUFFIX: &str = "<!--EndFragment--></body></html>";
const HEADER_TEMPLATE: &str = "Version:1.0\r\nStartHTML:0000000000\r\nEndHTML:0000000000\r\nStartFragment:0000000000\r\nEndFragment:0000000000\r\n";

pub(super) fn rich_formats(
    html_format: u32,
    fragment: &str,
    text: &str,
) -> Result<Vec<(u32, Vec<u8>)>, ProductError> {
    if html_format == 0
        || html_format == 13
        || fragment.contains('\0')
        || text.contains('\0')
        || fragment.contains("<!--StartFragment-->")
        || fragment.contains("<!--EndFragment-->")
        || fragment.len() > MAX_FORMAT_BYTES
        || text.len() > MAX_FORMAT_BYTES
    {
        return Err(ProductError::Clipboard);
    }

    let start_html = HEADER_TEMPLATE.len();
    let start_fragment = start_html + PREFIX.len();
    let end_fragment = start_fragment + fragment.len();
    let end_html = end_fragment + SUFFIX.len();
    let text_units = text.encode_utf16().count();
    if end_html + 1 > MAX_FORMAT_BYTES || (text_units + 1) * 2 > MAX_FORMAT_BYTES {
        return Err(ProductError::Clipboard);
    }
    let header = format!(
        "Version:1.0\r\nStartHTML:{start_html:010}\r\nEndHTML:{end_html:010}\r\nStartFragment:{start_fragment:010}\r\nEndFragment:{end_fragment:010}\r\n"
    );
    let mut html = Vec::with_capacity(end_html + 1);
    html.extend_from_slice(header.as_bytes());
    html.extend_from_slice(PREFIX.as_bytes());
    html.extend_from_slice(fragment.as_bytes());
    html.extend_from_slice(SUFFIX.as_bytes());
    html.push(0);
    let text = text
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect();
    Ok(vec![(html_format, html), (13, text)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offset(header: &str, name: &str) -> usize {
        header
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .unwrap()
            .parse()
            .unwrap()
    }

    #[test]
    fn utf8_byte_offsets_select_the_exact_fragment_and_context() {
        let formats = rich_formats(0xc001, "<div>Café 👋</div>", "Café 👋").unwrap();
        assert_eq!(
            formats.iter().map(|entry| entry.0).collect::<Vec<_>>(),
            [0xc001, 13]
        );
        let bytes = &formats[0].1;
        assert_eq!(bytes.last(), Some(&0));
        let all = std::str::from_utf8(&bytes[..bytes.len() - 1]).unwrap();
        let start_html = offset(all, "StartHTML:");
        let end_html = offset(all, "EndHTML:");
        let start = offset(all, "StartFragment:");
        let end = offset(all, "EndFragment:");
        assert_eq!(&bytes[start..end], "<div>Café 👋</div>".as_bytes());
        assert_eq!(
            &bytes[start_html..start],
            b"<html><body><!--StartFragment-->"
        );
        assert_eq!(&bytes[end..end_html], b"<!--EndFragment--></body></html>");
        assert_eq!(end_html, bytes.len() - 1);
        let text_units: Vec<_> = formats[1]
            .1
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes(*pair))
            .collect();
        assert_eq!(String::from_utf16(&text_units).unwrap(), "Café 👋\0");
    }

    #[test]
    fn refuses_embedded_nul_or_fragment_delimiters() {
        for (html, text) in [
            ("<div>\0</div>", "ok"),
            ("<div>ok</div>", "bad\0text"),
            ("<div><!--StartFragment--></div>", "ok"),
            ("<div><!--EndFragment--></div>", "ok"),
        ] {
            assert_eq!(
                rich_formats(0xc001, html, text),
                Err(ProductError::Clipboard)
            );
        }
    }

    #[test]
    fn refuses_oversized_html_and_unicode_text() {
        let large = "x".repeat(8 * 1024 * 1024);
        assert_eq!(
            rich_formats(0xc001, &large, "ok"),
            Err(ProductError::Clipboard)
        );
        assert_eq!(
            rich_formats(0xc001, "<div>ok</div>", &large),
            Err(ProductError::Clipboard)
        );
    }
}
