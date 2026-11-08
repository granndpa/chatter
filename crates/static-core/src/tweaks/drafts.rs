//! Unfinished messages, kept on disk so they come back after a restart:
//! each conversation's text, and the message it's replying to.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub const FILE: &str = "drafts.json";

/// The message a draft replies to.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reply {
    pub item: String,
    pub author: String,
    pub body: String,
    /// The reply notifies the person it replies to.
    pub ping: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Drafts {
    /// Each conversation's unsent text, by room id.
    pub text: BTreeMap<String, String>,
    /// Each conversation's reply in progress, by room id.
    pub replies: BTreeMap<String, Reply>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drafts_keep_lines_and_replies_through_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut drafts = Drafts::default();
        drafts.text.insert("discord:1:2".into(), "first line\n**second**".into());
        drafts.replies.insert(
            "discord:1:2".into(),
            Reply { item: "m".into(), author: "Ann".into(), body: "hi".into(), ping: true },
        );
        super::super::save(dir.path(), FILE, &drafts).unwrap();
        assert_eq!(super::super::load::<Drafts>(dir.path(), FILE), drafts);
    }
}
