//! Reminders about messages: due at a time, then shown as a notification
//! and kept as "due" until they're cancelled or snoozed.

use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};

use super::SavedMessage;

pub const FILE: &str = "reminders.json";

/// How long Snooze puts a reminder off.
pub const SNOOZE_SECONDS: u64 = 10 * 60;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reminder {
    pub id: u64,
    pub message: SavedMessage,
    /// When it's due, in seconds since the Unix epoch.
    pub due: u64,
    /// Its notification was shown; it waits to be cancelled or snoozed.
    pub fired: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reminders {
    pub next_id: u64,
    pub list: Vec<Reminder>,
}

impl Reminders {
    pub fn add(&mut self, message: SavedMessage, due: u64) -> u64 {
        self.next_id += 1;
        self.list.push(Reminder {
            id: self.next_id,
            message,
            due,
            fired: false,
        });
        self.next_id
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let before = self.list.len();
        self.list.retain(|reminder| reminder.id != id);
        self.list.len() != before
    }

    /// Moves a reminder to a new time, which also makes a due one wait again.
    pub fn set_due(&mut self, id: u64, due: u64) {
        if let Some(reminder) = self.list.iter_mut().find(|reminder| reminder.id == id) {
            reminder.due = due;
            reminder.fired = false;
        }
    }

    /// Every reminder, due ones first, then soonest first.
    pub fn listed(&self) -> Vec<&Reminder> {
        let mut listed: Vec<&Reminder> = self.list.iter().collect();
        listed.sort_by_key(|reminder| (!reminder.fired, reminder.due));
        listed
    }

    /// The reminders that came due by `now` and haven't been shown, marked
    /// shown. Those that came due while Chatter was closed come too.
    pub fn take_due(&mut self, now: u64) -> Vec<Reminder> {
        let mut due = Vec::new();
        for reminder in &mut self.list {
            if !reminder.fired && reminder.due <= now {
                reminder.fired = true;
                due.push(reminder.clone());
            }
        }
        due
    }
}

/// When a Remind Me choice is due, from `now`: 0 in 15 minutes, 1 in an
/// hour, 2 in three hours, 3 tomorrow at 9:00.
pub fn preset(choice: i32, now: u64) -> Option<u64> {
    match choice {
        0 => Some(now + 15 * 60),
        1 => Some(now + 60 * 60),
        2 => Some(now + 3 * 60 * 60),
        3 => {
            let today = Local.timestamp_opt(now as i64, 0).single()?.date_naive();
            local_seconds(today.succ_opt()?.and_time(NaiveTime::from_hms_opt(9, 0, 0)?))
        }
        _ => None,
    }
}

/// A time typed in as `2026-11-09 18:30`, or `18:30` for the next time the
/// clock shows it; none when it can't be read or is already past.
pub fn parse(text: &str, now: u64) -> Option<u64> {
    let text = text.trim();
    let due = if let Ok(time) = NaiveTime::parse_from_str(text, "%H:%M") {
        let today = Local.timestamp_opt(now as i64, 0).single()?.date_naive();
        let at = local_seconds(today.and_time(time))?;
        if at > now {
            at
        } else {
            local_seconds((today + Duration::days(1)).and_time(time))?
        }
    } else {
        let at = NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M")
            .or_else(|_| NaiveDateTime::parse_from_str(text, "%d/%m/%Y %H:%M"))
            .ok()?;
        local_seconds(at)?
    };
    (due > now).then_some(due)
}

fn local_seconds(at: NaiveDateTime) -> Option<u64> {
    let local = Local.from_local_datetime(&at).earliest()?;
    u64::try_from(local.timestamp()).ok()
}

/// A due time as a short description, such as `Today at 18:30`.
pub fn describe(due: u64, now: u64, twelve_hour: bool) -> String {
    let (Some(due_at), Some(now_at)) = (
        Local.timestamp_opt(due as i64, 0).single(),
        Local.timestamp_opt(now as i64, 0).single(),
    ) else {
        return String::new();
    };
    let clock = if twelve_hour {
        due_at.format("%-I:%M %p").to_string()
    } else {
        due_at.format("%H:%M").to_string()
    };
    let (day, today): (NaiveDate, NaiveDate) = (due_at.date_naive(), now_at.date_naive());
    let when = if day == today {
        "Today".to_owned()
    } else if Some(day) == today.succ_opt() {
        "Tomorrow".to_owned()
    } else if day.year() == today.year() {
        due_at.format("%-d %B").to_string()
    } else {
        due_at.format("%-d %B %Y").to_string()
    };
    format!("{when} at {clock}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_reminders_fire_once_even_after_being_closed() {
        let mut reminders = Reminders::default();
        let early = reminders.add(SavedMessage::default(), 100);
        reminders.add(SavedMessage::default(), 500);
        // Chatter was closed at 100 and opened at 200.
        let due = reminders.take_due(200);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, early);
        assert!(reminders.take_due(200).is_empty(), "shown once");
        assert_eq!(reminders.listed()[0].id, early, "due ones list first");
        reminders.set_due(early, 300);
        assert_eq!(reminders.take_due(300).len(), 1, "a snoozed one fires again");
        assert!(reminders.remove(early));
    }

    #[test]
    fn presets_count_from_now() {
        assert_eq!(preset(0, 1_000), Some(1_000 + 900));
        assert_eq!(preset(2, 1_000), Some(1_000 + 10_800));
        let tomorrow = preset(3, 1_800_000_000).unwrap();
        assert!(tomorrow > 1_800_000_000 && tomorrow - 1_800_000_000 <= 2 * 24 * 3600);
        assert_eq!(preset(9, 1_000), None);
    }

    #[test]
    fn typed_times_must_be_readable_and_ahead() {
        let now = 1_800_000_000;
        assert!(parse("2099-01-01 10:00", now).is_some());
        assert!(parse("2000-01-01 10:00", now).is_none());
        assert!(parse("soon", now).is_none());
        let clock = parse("23:59", now).unwrap();
        assert!(clock > now && clock - now <= 24 * 3600);
    }
}
