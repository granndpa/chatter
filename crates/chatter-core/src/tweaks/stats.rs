//! Discord Wrapped's numbers: only what Chatter saw on this computer, counted
//! from the day tracking was turned on, per day. Nothing is fetched from
//! Discord to fill in the past.

use std::collections::BTreeMap;

use chrono::{Datelike, Local, NaiveDate, TimeZone};
use serde::{Deserialize, Serialize};

pub const FILE: &str = "stats.json";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Day {
    pub messages: u32,
    pub voice_seconds: u64,
    /// Opens of each server and conversation, by id.
    pub servers: BTreeMap<String, u32>,
    pub channels: BTreeMap<String, u32>,
    /// Emoji in messages sent, by the emoji or `:name:`.
    pub emojis: BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    /// Tracking is on.
    pub enabled: bool,
    /// When it was turned on, in seconds since the Unix epoch.
    pub started: u64,
    /// By local date, `2026-11-09`.
    pub days: BTreeMap<String, Day>,
    /// The names last seen for ids, for the recap.
    pub names: BTreeMap<String, String>,
}

/// A period's recap, its top lists highest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Recap {
    pub messages: u32,
    pub voice_seconds: u64,
    pub servers: Vec<(String, u32)>,
    pub channels: Vec<(String, u32)>,
    pub emojis: Vec<(String, u32)>,
    /// The weekday most messages went on, and the busiest date.
    pub weekday: Option<String>,
    pub busiest: Option<(String, u32)>,
    /// Days with any activity.
    pub active_days: u32,
}

fn date_of(seconds: u64) -> String {
    Local
        .timestamp_opt(seconds as i64, 0)
        .single()
        .map(|at| at.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

impl Stats {
    pub fn start(&mut self, now: u64) {
        if !self.enabled {
            self.enabled = true;
            if self.started == 0 {
                self.started = now;
            }
        }
    }

    pub fn stop(&mut self) {
        self.enabled = false;
    }

    /// Forgets everything, and tracking stops.
    pub fn delete(&mut self) {
        *self = Self::default();
    }

    fn today(&mut self, now: u64) -> Option<&mut Day> {
        self.enabled.then(|| self.days.entry(date_of(now)).or_default())
    }

    pub fn message(&mut self, now: u64, text: &str) {
        let Some(day) = self.today(now) else { return };
        day.messages += 1;
        for emoji in emojis(text) {
            *day.emojis.entry(emoji).or_default() += 1;
        }
    }

    pub fn voice(&mut self, now: u64, seconds: u64) {
        if let Some(day) = self.today(now) {
            day.voice_seconds += seconds;
        }
    }

    /// A conversation opened, and its server if it has one, with their names.
    pub fn visit(&mut self, now: u64, room: (&str, &str), server: Option<(&str, &str)>) {
        if !self.enabled {
            return;
        }
        self.names.insert(room.0.to_owned(), room.1.to_owned());
        if let Some((id, name)) = server {
            self.names.insert(id.to_owned(), name.to_owned());
        }
        let Some(day) = self.today(now) else { return };
        *day.channels.entry(room.0.to_owned()).or_default() += 1;
        if let Some((id, _)) = server {
            *day.servers.entry(id.to_owned()).or_default() += 1;
        }
    }

    /// The recap of the last `days` days to `now`, or of everything with none.
    pub fn recap(&self, now: u64, days: Option<u32>) -> Recap {
        let today = date_of(now);
        let first = days.and_then(|days| {
            NaiveDate::parse_from_str(&today, "%Y-%m-%d")
                .ok()
                .map(|date| (date - chrono::Duration::days(i64::from(days) - 1)).format("%Y-%m-%d").to_string())
        });
        let mut recap = Recap::default();
        let (mut servers, mut channels, mut emojis) = (BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
        let mut weekdays: BTreeMap<u32, (String, u32)> = BTreeMap::new();
        for (date, day) in &self.days {
            if first.as_ref().is_some_and(|first| date < first) || date > &today {
                continue;
            }
            recap.messages += day.messages;
            recap.voice_seconds += day.voice_seconds;
            recap.active_days += 1;
            for (id, count) in &day.servers {
                *servers.entry(id.clone()).or_insert(0) += count;
            }
            for (id, count) in &day.channels {
                *channels.entry(id.clone()).or_insert(0) += count;
            }
            for (emoji, count) in &day.emojis {
                *emojis.entry(emoji.clone()).or_insert(0) += count;
            }
            if recap.busiest.as_ref().is_none_or(|(_, most)| day.messages > *most) && day.messages > 0 {
                recap.busiest = Some((date.clone(), day.messages));
            }
            if let Ok(parsed) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                let weekday = parsed.weekday();
                let entry = weekdays
                    .entry(weekday.num_days_from_monday())
                    .or_insert_with(|| (format!("{weekday:?}"), 0));
                entry.1 += day.messages;
            }
        }
        let named = |counts: BTreeMap<String, u32>, names: &BTreeMap<String, String>| {
            let mut list: Vec<(String, u32)> = counts
                .into_iter()
                .map(|(id, count)| (names.get(&id).cloned().unwrap_or(id), count))
                .collect();
            list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            list.truncate(5);
            list
        };
        recap.servers = named(servers, &self.names);
        recap.channels = named(channels, &self.names);
        recap.emojis = named(emojis, &BTreeMap::new());
        recap.weekday = weekdays
            .into_values()
            .filter(|(_, count)| *count > 0)
            .max_by_key(|(_, count)| *count)
            .map(|(name, _)| full_weekday(&name));
        recap
    }
}

fn full_weekday(short: &str) -> String {
    match short {
        "Mon" => "Monday",
        "Tue" => "Tuesday",
        "Wed" => "Wednesday",
        "Thu" => "Thursday",
        "Fri" => "Friday",
        "Sat" => "Saturday",
        _ => "Sunday",
    }
    .to_owned()
}

/// The emoji in a message: custom ones as `:name:`, and pictographic
/// characters.
fn emojis(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    // Custom emoji are written <:name:id> or <a:name:id>.
    for part in text.split('<').skip(1) {
        let inner = part.split('>').next().unwrap_or_default();
        let mut pieces = inner.split(':');
        let first = pieces.next().unwrap_or_default();
        if (first.is_empty() || first == "a")
            && let Some(name) = pieces.next()
            && !name.is_empty()
        {
            found.push(format!(":{name}:"));
        }
    }
    found.extend(
        text.chars()
            .filter(|c| {
                let code = *c as u32;
                (0x1F300..=0x1FAFF).contains(&code) || (0x2600..=0x27BF).contains(&code)
            })
            .map(String::from),
    );
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_counts_until_tracking_starts() {
        let mut stats = Stats::default();
        stats.message(1_800_000_000, "hi");
        assert!(stats.days.is_empty());
        stats.start(1_800_000_000);
        stats.message(1_800_000_000, "hi \u{1F600} <:pog:123>");
        stats.voice(1_800_000_000, 30);
        stats.visit(1_800_000_000, ("r1", "general"), Some(("s1", "Server")));
        let recap = stats.recap(1_800_000_000, Some(7));
        assert_eq!(recap.messages, 1);
        assert_eq!(recap.voice_seconds, 30);
        assert_eq!(recap.servers, [("Server".to_owned(), 1)]);
        assert_eq!(recap.channels, [("general".to_owned(), 1)]);
        assert_eq!(recap.emojis.len(), 2);
        assert!(recap.weekday.is_some());
    }

    #[test]
    fn old_days_fall_outside_a_week() {
        let mut stats = Stats::default();
        let now = 1_800_000_000;
        stats.start(now - 30 * 86_400);
        stats.message(now - 20 * 86_400, "old");
        stats.message(now, "new");
        assert_eq!(stats.recap(now, Some(7)).messages, 1);
        assert_eq!(stats.recap(now, None).messages, 2);
        stats.delete();
        assert!(!stats.enabled && stats.days.is_empty());
    }
}
