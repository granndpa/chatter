//! The song this account plays on Spotify, shown as Discord's own client
//! shows it: Discord lends the connected Spotify account's token, Spotify
//! says what plays, and the session adds it to its presence.

use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};
use tokio::sync::watch;

use crate::api::{Api, Spotify};

/// How often Spotify is asked what plays.
const POLL: Duration = Duration::from_secs(5);
/// How long to wait without a Spotify connection, or after Discord failed.
const RETRY: Duration = Duration::from_secs(60);
/// A start that moved more than this was a seek, worth showing again.
const DRIFT_MS: u64 = 2500;

/// Asks Spotify what plays until the receiver is dropped with its session.
pub(crate) fn spawn(api: Arc<Api>) -> watch::Receiver<Option<Arc<Value>>> {
    let (sender, receiver) = watch::channel(None);
    let mut changes = api.spotify_changes();
    tokio::spawn(async move {
        let mut token: Option<String> = None;
        let mut user: Option<String> = None;
        loop {
            let wait = tokio::select! {
                () = sender.closed() => return,
                _ = changes.changed() => {
                    token = None;
                    show(&sender, None);
                    continue;
                }
                wait = poll(&api, &mut token, &mut user, &sender) => wait,
            };
            tokio::select! {
                () = sender.closed() => return,
                _ = changes.changed() => {
                    token = None;
                    show(&sender, None);
                }
                () = tokio::time::sleep(wait) => {}
            }
        }
    });
    receiver
}

async fn poll(
    api: &Api,
    token: &mut Option<String>,
    user: &mut Option<String>,
    sender: &watch::Sender<Option<Arc<Value>>>,
) -> Duration {
    if user.is_none() {
        match api.own_user_id().await {
            Ok(id) => *user = id,
            Err(_) => return RETRY,
        }
    }
    let Some(user) = user.clone() else {
        return RETRY;
    };
    if token.is_none() {
        match api.spotify_access_token().await {
            Ok(Some(fresh)) => *token = Some(fresh),
            _ => {
                show(sender, None);
                return RETRY;
            }
        }
    }
    let Some(current) = token.as_deref() else {
        return RETRY;
    };
    let activity = match api.spotify_now_playing(current).await {
        Ok(Spotify::Playing(answer)) => activity(&user, now_ms(), &answer),
        Ok(Spotify::Idle) => None,
        Ok(Spotify::Expired) => {
            *token = None;
            return Duration::from_secs(1);
        }
        // Keeps what it showed through a brief failure.
        Err(_) => return POLL,
    };
    show(sender, activity);
    POLL
}

/// Sends the activity only when the song, its state or its place changed,
/// so the presence isn't sent again every poll.
fn show(sender: &watch::Sender<Option<Arc<Value>>>, activity: Option<Value>) {
    sender.send_if_modified(|shown| {
        if same(shown.as_deref(), activity.as_ref()) {
            return false;
        }
        *shown = activity.map(Arc::new);
        true
    });
}

fn same(old: Option<&Value>, new: Option<&Value>) -> bool {
    match (old, new) {
        (None, None) => true,
        (Some(old), Some(new)) => {
            old["sync_id"] == new["sync_id"] && start(old).abs_diff(start(new)) < DRIFT_MS
        }
        _ => false,
    }
}

fn start(activity: &Value) -> u64 {
    activity["timestamps"]["start"].as_u64().unwrap_or(0)
}

/// The activity Discord's client sends for a track, from Spotify's
/// currently-playing answer. Nothing while paused, or for ads and episodes.
fn activity(user: &str, now_ms: u64, answer: &Value) -> Option<Value> {
    if answer["is_playing"] != true || answer["currently_playing_type"] != "track" {
        return None;
    }
    let item = &answer["item"];
    let id = item["id"].as_str()?;
    let progress = answer["progress_ms"].as_u64()?;
    let duration = item["duration_ms"].as_u64()?;
    let artists = item["artists"].as_array()?;
    let names: Vec<&str> = artists.iter().filter_map(|a| a["name"].as_str()).collect();
    let artist_ids: Vec<&str> = artists.iter().filter_map(|a| a["id"].as_str()).collect();
    let start = now_ms.saturating_sub(progress);
    let mut activity = json!({
        "type": 2,
        "name": "Spotify",
        "flags": 48,
        "sync_id": id,
        "details": item["name"].as_str().unwrap_or_default(),
        "state": names.join("; "),
        "timestamps": {"start": start, "end": start + duration},
        "party": {"id": format!("spotify:{user}")},
        "assets": {"large_text": item["album"]["name"].as_str().unwrap_or_default()},
        "metadata": {
            "album_id": item["album"]["id"].as_str().unwrap_or_default(),
            "artist_ids": artist_ids,
            "type": "track",
        },
    });
    if let Some(hash) = item["album"]["images"][0]["url"]
        .as_str()
        .and_then(|url| url.rsplit('/').next())
        .filter(|hash| !hash.is_empty())
    {
        activity["assets"]["large_image"] = json!(format!("spotify:{hash}"));
    }
    if let Some(context) = answer["context"]["uri"].as_str() {
        activity["metadata"]["context_uri"] = json!(context);
    }
    Some(activity)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(playing: bool) -> Value {
        json!({
            "is_playing": playing,
            "currently_playing_type": "track",
            "progress_ms": 10_000,
            "context": {"uri": "spotify:playlist:abc"},
            "item": {
                "id": "1zi7xx7UVEFkmKfv06H8x0",
                "name": "One Dance",
                "duration_ms": 174_000,
                "album": {"id": "alb", "name": "Views",
                          "images": [{"url": "https://i.scdn.co/image/ab67616d0000b273abc"}]},
                "artists": [{"id": "a1", "name": "Drake"}, {"id": "a2", "name": "Wizkid"}],
            },
        })
    }

    #[test]
    fn a_playing_track_becomes_discords_spotify_activity() {
        let activity = activity("42", 100_000, &answer(true)).unwrap();
        assert_eq!(activity["sync_id"], "1zi7xx7UVEFkmKfv06H8x0");
        assert_eq!(activity["state"], "Drake; Wizkid");
        assert_eq!(activity["party"]["id"], "spotify:42");
        assert_eq!(
            activity["assets"]["large_image"],
            "spotify:ab67616d0000b273abc"
        );
        assert_eq!(activity["timestamps"]["start"], 90_000);
        assert_eq!(activity["timestamps"]["end"], 264_000);
    }

    #[test]
    fn nothing_shows_while_paused() {
        assert!(activity("42", 100_000, &answer(false)).is_none());
    }

    #[test]
    fn small_drift_is_the_same_song_but_a_seek_is_not() {
        let first = activity("42", 100_000, &answer(true)).unwrap();
        let later = activity("42", 101_000, &answer(true)).unwrap();
        let seeked = activity("42", 160_000, &answer(true)).unwrap();
        assert!(same(Some(&first), Some(&later)));
        assert!(!same(Some(&first), Some(&seeked)));
    }
}
