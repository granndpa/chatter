# Discord Tweaks

Chatter's extras on top of the Discord client. Most are switches under
**Settings, Discord Tweaks**; the rest live in menus and the **Saved** panel.

## Messages

| Tweak | Where | Notes |
| --- | --- | --- |
| Message logger | Always on (switch in Discord Tweaks) | Deleted messages stay in red, edits keep the old text underneath. Right-click a deleted one and choose **Close Deleted Message** to hide it. |
| Translate | Message menu | Replaces the text with English and notes the language. **Show Original** switches back. Sends the text to Google Translate's free endpoint. |
| Real timestamps | Hover a message's time | Full date and time with seconds. |
| Bookmarks | Message menu, **Bookmark Message** | Listed in **Saved, Bookmarks**: search, collections, Up/Down to reorder. Clicking one jumps to the message. |
| Reminders | Message menu, **Remind Me** | 15 minutes, 1 hour, 3 hours, tomorrow at 9:00, or a typed time (`2026-11-09 18:30` or `18:30`). |
| Hall of Fame | Message menu, **Add to Hall of Fame** | Its own tab in **Saved**, with captions, collections, search, sorting and **Export** to PNG. |
| Drafts | Automatic | Unsent text and the reply in progress are kept per conversation and come back after a restart. |

### Reminders

When a reminder comes due, Windows shows a notification from Chatter's tray
icon; clicking it opens the message. Reminders that came due while Chatter was
closed show when it starts. Due reminders stay in **Saved, Reminders** until
you choose **Done**, **Snooze 10 min** or **Change time**.

Linux has no reminder notification yet: due reminders show in the Reminders
tab, marked **Due now**.

### Unavailable messages

A bookmark, reminder or Hall of Fame entry keeps a copy of the message's text,
author and place, so it still shows after the message is deleted or the
channel goes away. Opening one whose conversation is gone says so instead.

## Pictures

- **Full-size images:** the viewer loads pictures at up to 4096 px.
- **Click to zoom:** click an image in the viewer to zoom in where you
  clicked, drag to move around, and click again to zoom out.
- **Save pictures:** right-click a person for **Save Avatar** (and **Save
  Banner** while their profile is open), or a server for **Save Server Icon**
  and **Save Server Banner**.

## Servers and navigation

- **Read All** under Direct Messages marks every server and DM read.
- **Show channels you can't access** lists hidden channels, locked.

## Voice

- **Voice log:** the clock button in the voice panel lists who joined, left
  or moved during the call.
- **Fake deafen:** others see you muted and deafened while you still hear
  everyone.

## Privacy

- **Silent typing:** turn off "Let others see when you're typing".
- **Mark messages read on Discord:** off keeps your read state to yourself.
- **Streamer Mode** turns on by itself while OBS runs.

## Where the data is

Bookmarks, reminders and drafts are JSON files in the data folder, under
`tweaks/<Discord user id>/`, one folder per account: `bookmarks.json`,
`reminders.json` and `drafts.json`. They never leave the computer and hold no
login tokens. Delete the folder to start over.
