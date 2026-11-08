//! Bookmarked messages, in collections, and the Hall of Fame, which is the
//! same list with its own flag, captions and collections.

use serde::{Deserialize, Serialize};

use super::SavedMessage;

pub const FILE: &str = "bookmarks.json";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Bookmark {
    pub id: u64,
    pub message: SavedMessage,
    /// The collection it's in; empty for none.
    pub collection: String,
    /// In the Hall of Fame rather than the bookmarks.
    pub hall_of_fame: bool,
    /// A personal caption, shown in the Hall of Fame.
    pub caption: String,
    /// When it was saved, in seconds since the Unix epoch.
    pub added: u64,
}

/// How a list of bookmarks is ordered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    /// Newest saved first, or the order it was arranged in.
    #[default]
    Saved,
    /// By when the message was sent, newest first.
    Newest,
    Oldest,
    /// By author's name.
    Author,
}

impl Sort {
    pub fn from_index(index: i32) -> Self {
        match index {
            1 => Self::Newest,
            2 => Self::Oldest,
            3 => Self::Author,
            _ => Self::Saved,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Bookmarks {
    pub next_id: u64,
    /// In their arranged order, newest saved first unless moved.
    pub list: Vec<Bookmark>,
}

impl Bookmarks {
    /// Saves a message, or finds it when it's saved already. Its id either way.
    pub fn add(&mut self, message: SavedMessage, hall_of_fame: bool, now: u64) -> u64 {
        if let Some(found) = self
            .list
            .iter()
            .find(|found| found.message.item == message.item && found.hall_of_fame == hall_of_fame)
        {
            return found.id;
        }
        self.next_id += 1;
        self.list.insert(
            0,
            Bookmark {
                id: self.next_id,
                message,
                collection: String::new(),
                hall_of_fame,
                caption: String::new(),
                added: now,
            },
        );
        self.next_id
    }

    pub fn get(&self, id: u64) -> Option<&Bookmark> {
        self.list.iter().find(|bookmark| bookmark.id == id)
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let before = self.list.len();
        self.list.retain(|bookmark| bookmark.id != id);
        self.list.len() != before
    }

    pub fn set_collection(&mut self, id: u64, collection: &str) {
        if let Some(bookmark) = self.list.iter_mut().find(|bookmark| bookmark.id == id) {
            bookmark.collection = collection.trim().to_owned();
        }
    }

    pub fn set_caption(&mut self, id: u64, caption: &str) {
        if let Some(bookmark) = self.list.iter_mut().find(|bookmark| bookmark.id == id) {
            bookmark.caption = caption.trim().to_owned();
        }
    }

    /// Moves a bookmark up (negative) or down among those of its kind.
    pub fn shift(&mut self, id: u64, by: i32) {
        let Some(from) = self.list.iter().position(|bookmark| bookmark.id == id) else {
            return;
        };
        let hall = self.list[from].hall_of_fame;
        let same: Vec<usize> = (0..self.list.len())
            .filter(|&index| self.list[index].hall_of_fame == hall)
            .collect();
        let Some(at) = same.iter().position(|&index| index == from) else {
            return;
        };
        let to = at as i64 + i64::from(by);
        if to < 0 || to >= same.len() as i64 {
            return;
        }
        self.list.swap(from, same[to as usize]);
    }

    /// The collections in use, by name.
    pub fn collections(&self, hall_of_fame: bool) -> Vec<String> {
        let mut names: Vec<String> = self
            .list
            .iter()
            .filter(|bookmark| bookmark.hall_of_fame == hall_of_fame && !bookmark.collection.is_empty())
            .map(|bookmark| bookmark.collection.clone())
            .collect();
        names.sort_by_key(|name| name.to_lowercase());
        names.dedup();
        names
    }

    /// Those of a kind whose text, author, place, caption or collection has
    /// `query` in it, in `collection` unless that's empty, in `sort`'s order.
    pub fn search(&self, hall_of_fame: bool, query: &str, collection: &str, sort: Sort) -> Vec<&Bookmark> {
        let query = query.trim().to_lowercase();
        let mut found: Vec<&Bookmark> = self
            .list
            .iter()
            .filter(|bookmark| bookmark.hall_of_fame == hall_of_fame)
            .filter(|bookmark| collection.is_empty() || bookmark.collection == collection)
            .filter(|bookmark| {
                query.is_empty() || {
                    let message = &bookmark.message;
                    [
                        &message.body,
                        &message.author,
                        &message.server,
                        &message.channel,
                        &bookmark.caption,
                        &bookmark.collection,
                    ]
                    .iter()
                    .any(|text| text.to_lowercase().contains(&query))
                }
            })
            .collect();
        match sort {
            Sort::Saved => {}
            Sort::Newest => found.sort_by(|a, b| b.message.sent.cmp(&a.message.sent)),
            Sort::Oldest => found.sort_by(|a, b| a.message.sent.cmp(&b.message.sent)),
            Sort::Author => found.sort_by_key(|bookmark| bookmark.message.author.to_lowercase()),
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(item: &str, author: &str, body: &str, sent: u64) -> SavedMessage {
        SavedMessage {
            room: "discord:1:2".into(),
            item: item.into(),
            author: author.into(),
            body: body.into(),
            sent,
            ..Default::default()
        }
    }

    #[test]
    fn saving_a_message_twice_keeps_one_bookmark() {
        let mut bookmarks = Bookmarks::default();
        let first = bookmarks.add(message("m1", "Ann", "hi", 1), false, 10);
        assert_eq!(bookmarks.add(message("m1", "Ann", "hi", 1), false, 11), first);
        // The Hall of Fame is its own list.
        assert_ne!(bookmarks.add(message("m1", "Ann", "hi", 1), true, 12), first);
        assert_eq!(bookmarks.list.len(), 2);
    }

    #[test]
    fn search_filters_by_text_kind_and_collection_and_sorts() {
        let mut bookmarks = Bookmarks::default();
        let a = bookmarks.add(message("a", "Zed", "pizza tonight", 5), false, 1);
        bookmarks.add(message("b", "Ann", "Pizza recipe", 9), false, 2);
        bookmarks.add(message("c", "Bob", "pizza", 7), true, 3);
        bookmarks.set_collection(a, "Food");
        let found = bookmarks.search(false, "PIZZA", "", Sort::Newest);
        assert_eq!(found.iter().map(|b| b.message.item.as_str()).collect::<Vec<_>>(), ["b", "a"]);
        let food = bookmarks.search(false, "", "Food", Sort::Saved);
        assert_eq!(food.len(), 1);
        assert_eq!(bookmarks.collections(false), ["Food"]);
        let by_author = bookmarks.search(false, "", "", Sort::Author);
        assert_eq!(by_author[0].message.author, "Ann");
    }

    #[test]
    fn bookmarks_move_and_go() {
        let mut bookmarks = Bookmarks::default();
        let a = bookmarks.add(message("a", "A", "", 0), false, 1);
        let b = bookmarks.add(message("b", "B", "", 0), false, 2);
        // Newest first: b, a.
        bookmarks.shift(a, -1);
        assert_eq!(bookmarks.list[0].id, a);
        bookmarks.shift(a, -1);
        assert_eq!(bookmarks.list[0].id, a, "the first can't go higher");
        assert!(bookmarks.remove(b));
        assert!(!bookmarks.remove(b));
    }
}
