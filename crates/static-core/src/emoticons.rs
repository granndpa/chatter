//! Emoticons typed as text, such as `:)`, sent as the emoji they stand for,
//! as Discord's own setting does.

use std::borrow::Cow;

/// Each emoticon and its emoji; longer ones first, so `:'(` wins over `:(`.
const EMOTICONS: &[(&str, &str)] = &[
    (":'(", "\u{1F622}"),
    ("</3", "\u{1F494}"),
    (":-)", "\u{1F642}"),
    (":-(", "\u{1F641}"),
    (":-D", "\u{1F604}"),
    (":-P", "\u{1F61B}"),
    (";-)", "\u{1F609}"),
    (":)", "\u{1F642}"),
    (":(", "\u{1F641}"),
    (":D", "\u{1F604}"),
    (":P", "\u{1F61B}"),
    (":p", "\u{1F61B}"),
    (";)", "\u{1F609}"),
    (":O", "\u{1F62E}"),
    (":o", "\u{1F62E}"),
    (":|", "\u{1F610}"),
    ("<3", "\u{2764}\u{FE0F}"),
    ("xD", "\u{1F606}"),
    ("XD", "\u{1F606}"),
];

/// `text` with each emoticon that stands as a word of its own turned into
/// its emoji. Text with code in it stays as typed, as an emoticon there is
/// usually code.
pub fn convert(text: &str) -> Cow<'_, str> {
    if text.contains('`') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut changed = false;
    let mut rest = text;
    while !rest.is_empty() {
        let space = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let (word, after) = rest.split_at(space);
        match EMOTICONS.iter().find(|(emoticon, _)| *emoticon == word) {
            Some((_, emoji)) => {
                out.push_str(emoji);
                changed = true;
            }
            None => out.push_str(word),
        }
        let gap = after
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(after.len());
        out.push_str(&after[..gap]);
        rest = &after[gap..];
    }
    if changed {
        Cow::Owned(out)
    } else {
        Cow::Borrowed(text)
    }
}

#[cfg(test)]
mod tests {
    use super::convert;

    #[test]
    fn emoticons_on_their_own_become_emoji() {
        assert_eq!(convert("hi :)"), "hi \u{1F642}");
        assert_eq!(convert(":D  ok\n<3"), "\u{1F604}  ok\n\u{2764}\u{FE0F}");
        assert_eq!(convert(":'("), "\u{1F622}");
    }

    #[test]
    fn emoticons_inside_words_links_and_code_stay() {
        assert_eq!(convert("see http://x.y/:)"), "see http://x.y/:)");
        assert_eq!(convert("a:)b"), "a:)b");
        assert_eq!(convert("`:)` :)"), "`:)` :)");
        assert_eq!(convert("no emoticons"), "no emoticons");
        assert!(matches!(convert("plain"), std::borrow::Cow::Borrowed(_)));
    }
}
