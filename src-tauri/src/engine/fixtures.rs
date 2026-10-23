use serde::Serialize;

#[derive(Clone, Debug, Serialize, specta::Type)]
pub struct Fixture {
    pub id: &'static str,
    pub title: &'static str,
    pub trigger: &'static str,
    pub body: String,
}

pub fn fixtures() -> Vec<Fixture> {
    [
        ("hello", "Greeting", ";hello", "Hello from Texpio!".into()),
        (
            "address",
            "Synthetic address",
            ";addr",
            "17 Example Road, Sample City".into(),
        ),
        ("accent", "Accented text", ";accent", "Café déjà vu".into()),
        ("devanagari", "Devanagari", ";hindi", "नमस्ते दुनिया".into()),
        (
            "emoji",
            "Emoji and combining mark",
            ";emoji",
            "Hello 👋 e\u{301}".into(),
        ),
        (
            "multiline",
            "Multiple lines",
            ";lines",
            "Line one\nLine two".into(),
        ),
        ("long", "Long plain text", ";long", "x".repeat(2_000)),
    ]
    .into_iter()
    .map(|(id, title, trigger, body)| Fixture {
        id,
        title,
        trigger,
        body,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_preserve_unicode_and_long_output() {
        let items = fixtures();
        assert_eq!(items.len(), 7);
        assert_eq!(
            items.iter().find(|f| f.id == "emoji").unwrap().body,
            "Hello 👋 e\u{301}"
        );
        assert_eq!(
            items.iter().find(|f| f.id == "long").unwrap().body,
            "x".repeat(2_000)
        );
        let hello = serde_json::to_value(&items[0]).unwrap();
        assert_eq!(
            hello,
            serde_json::json!({"id":"hello","title":"Greeting","trigger":";hello","body":"Hello from Texpio!"})
        );
    }
}
