//! Pinned servers and friends, which the rail shows at its top in the
//! order they were arranged.

use serde::{Deserialize, Serialize};

pub const FILE: &str = "pins.json";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pins {
    /// Server ids, as in `ChatServer::id`.
    pub servers: Vec<String>,
    /// Discord user ids.
    pub friends: Vec<String>,
}

/// Pins `id` at the end, or unpins it. Whether it's pinned now.
pub fn toggle(list: &mut Vec<String>, id: &str) -> bool {
    match list.iter().position(|pinned| pinned == id) {
        Some(at) => {
            list.remove(at);
            false
        }
        None => {
            list.push(id.to_owned());
            true
        }
    }
}

/// Moves a pin up (negative) or down, staying in the list.
pub fn shift(list: &mut [String], id: &str, by: i32) {
    let Some(from) = list.iter().position(|pinned| pinned == id) else {
        return;
    };
    let to = from as i64 + i64::from(by);
    if (0..list.len() as i64).contains(&to) {
        list.swap(from, to as usize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_toggle_and_move_within_the_list() {
        let mut list = Vec::new();
        assert!(toggle(&mut list, "a"));
        assert!(toggle(&mut list, "b"));
        shift(&mut list, "b", -1);
        assert_eq!(list, ["b", "a"]);
        shift(&mut list, "b", -1);
        assert_eq!(list, ["b", "a"], "the first stays first");
        assert!(!toggle(&mut list, "b"));
        assert_eq!(list, ["a"]);
    }
}
