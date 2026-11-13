//! The Mentions Inbox: messages that mentioned or replied to this account,
//! and DMs, as they arrive, newest first.

use serde::{Deserialize, Serialize};

use super::SavedMessage;

pub const FILE: &str = "inbox.json";

/// The most entries kept; older ones go first.
pub const KEPT: usize = 500;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Entry {
    pub id: u64,
    pub message: SavedMessage,
    pub read: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Inbox {
    pub next_id: u64,
    /// Newest first.
    pub entries: Vec<Entry>,
}

impl Inbox {
    /// Adds a message, unless it's in already. Whether it was added.
    pub fn add(&mut self, message: SavedMessage) -> bool {
        if self
            .entries
            .iter()
            .any(|entry| entry.message.room == message.room && entry.message.item == message.item)
        {
            return false;
        }
        self.next_id += 1;
        self.entries.insert(
            0,
            Entry {
                id: self.next_id,
                message,
                read: false,
            },
        );
        self.entries.truncate(KEPT);
        true
    }

    pub fn get(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn mark(&mut self, id: u64, read: bool) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.read = read;
        }
    }

    pub fn mark_all(&mut self) {
        for entry in &mut self.entries {
            entry.read = true;
        }
    }

    pub fn unread(&self) -> usize {
        self.entries.iter().filter(|entry| !entry.read).count()
    }

    /// The entries shown: all, or only unread ones.
    pub fn listed(&self, unread_only: bool) -> impl Iterator<Item = &Entry> {
        self.entries
            .iter()
            .filter(move |entry| !unread_only || !entry.read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(item: &str) -> SavedMessage {
        SavedMessage {
            room: "discord:1:2".into(),
            item: item.into(),
            ..Default::default()
        }
    }

    #[test]
    fn the_same_message_comes_in_once_and_reads_can_be_marked() {
        let mut inbox = Inbox::default();
        assert!(inbox.add(message("d:1")));
        assert!(!inbox.add(message("d:1")));
        assert!(inbox.add(message("d:2")));
        assert_eq!(inbox.entries[0].message.item, "d:2", "newest first");
        assert_eq!(inbox.unread(), 2);
        inbox.mark(inbox.entries[1].id, true);
        assert_eq!(inbox.listed(true).count(), 1);
        inbox.mark_all();
        assert_eq!(inbox.unread(), 0);
    }

    #[test]
    fn only_the_newest_are_kept() {
        let mut inbox = Inbox::default();
        for index in 0..KEPT + 5 {
            inbox.add(message(&format!("d:{index}")));
        }
        assert_eq!(inbox.entries.len(), KEPT);
    }
}
