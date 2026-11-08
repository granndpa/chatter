//! Discord Tweaks: translates a message into English with Google Translate's
//! free endpoint, and says which language it was in.

use std::process::Command;

/// The message in English, and the name of the language it was written in.
pub fn to_english(text: &str) -> Result<(String, String), String> {
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl=en&dt=t&q={}",
        encode(text)
    );
    let mut curl = Command::new("curl");
    curl.args(["-sSfL", "--max-time", "15"]).arg(&url);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // No console window flashes up.
        curl.creation_flags(0x0800_0000);
    }
    let output = curl.output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("Google Translate didn't answer".into());
    }
    parse(&output.stdout)
}

fn parse(body: &[u8]) -> Result<(String, String), String> {
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| "Google Translate's answer was unreadable".to_owned())?;
    let translated: String = value[0]
        .as_array()
        .ok_or("Google Translate's answer was unreadable")?
        .iter()
        .filter_map(|part| part[0].as_str())
        .collect();
    let code = value[2].as_str().unwrap_or_default();
    Ok((translated.trim().to_owned(), language(code)))
}

/// Percent-encodes text for a query string.
fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// A language's English name from its code, such as `es` for Spanish.
fn language(code: &str) -> String {
    let base = code.split('-').next().unwrap_or(code);
    let name = match base {
        "af" => "Afrikaans", "ar" => "Arabic", "bg" => "Bulgarian", "bn" => "Bengali",
        "ca" => "Catalan", "cs" => "Czech", "cy" => "Welsh", "da" => "Danish",
        "de" => "German", "el" => "Greek", "en" => "English", "es" => "Spanish",
        "et" => "Estonian", "fa" => "Persian", "fi" => "Finnish", "fil" | "tl" => "Filipino",
        "fr" => "French", "ga" => "Irish", "he" | "iw" => "Hebrew", "hi" => "Hindi",
        "hr" => "Croatian", "hu" => "Hungarian", "hy" => "Armenian", "id" => "Indonesian",
        "is" => "Icelandic", "it" => "Italian", "ja" => "Japanese", "ka" => "Georgian",
        "kk" => "Kazakh", "ko" => "Korean", "la" => "Latin", "lt" => "Lithuanian",
        "lv" => "Latvian", "mk" => "Macedonian", "ms" => "Malay", "nl" => "Dutch",
        "no" | "nb" => "Norwegian", "pl" => "Polish", "pt" => "Portuguese", "ro" => "Romanian",
        "ru" => "Russian", "sk" => "Slovak", "sl" => "Slovenian", "sq" => "Albanian",
        "sr" => "Serbian", "sv" => "Swedish", "sw" => "Swahili", "ta" => "Tamil",
        "th" => "Thai", "tr" => "Turkish", "uk" => "Ukrainian", "ur" => "Urdu",
        "uz" => "Uzbek", "vi" => "Vietnamese", "zh" => "Chinese",
        "" => "an unknown language",
        other => return other.to_uppercase(),
    };
    name.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn googles_answer_gives_the_text_and_its_language() {
        let body = br#"[[["Hello ","Hola ",null,null,10],["world","mundo",null,null,10]],null,"es"]"#;
        assert_eq!(parse(body).unwrap(), ("Hello world".into(), "Spanish".into()));
    }

    #[test]
    fn text_is_percent_encoded() {
        assert_eq!(encode("a b&ø"), "a%20b%26%C3%B8");
    }
}
