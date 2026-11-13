# Themes

Settings → Appearance shows the window in one of the themes Chatter comes with,
or one of the person's own. Each is a full set of the colours of the Slint
global `Theme` (`ui/theme.slint`), plus whether it's dark, which sets the
std-widgets' colour scheme and the scrollbars'. `src/ui/theme.rs` holds the built-in ones as
`BUILT_IN`; `ui/theme.slint` says what each colour paints. The choice
is kept in `layout.json` as `theme`, the theme's id, and applied before the
window's first frame, and again to a window made anew. An id Chatter doesn't
know, or a deleted theme's, shows Chatter.

## Custom themes

New theme opens an editor on a copy of the theme shown; START FROM swaps it
for a copy of any other. The editor lists every colour by its name in
`ui/theme.slint`, with a swatch and its hex, which takes `#rgb`, `#rrggbb` or
`#rrggbbaa` and marks anything else. Each change shows at once, so the whole
window is the preview. Save keeps the theme and chooses it; Cancel, leaving
Appearance or closing settings shows the theme from before. A saved theme's
card has a pencil that opens it again to rename, change or delete it;
deleting the chosen one shows Chatter. A window rebuilt while the editor is open,
as closing to the [tray](tray.md) does, drops the unsaved edit.

`layout.json` keeps them in `custom_themes`, in the order Appearance shows
them after the built-in ones:

```json
{
  "id": "custom:1",
  "name": "Plum",
  "base": "yap",
  "dark": true,
  "colours": { "rail": "#111215", "chat": "#2a1a3a", "row-hover": "#1a1b20" }
}
```

`base` is the built-in theme the first copy came from (a copy of a custom
theme takes its base). Chatter writes every colour, but a colour that's missing,
such as one added in a later version, or that isn't a colour, takes the
base's value, and a name Chatter doesn't know is ignored, so themes survive new
colours. Ids are `custom:` and a number, and no built-in id has a colon, so
they can't collide; on reading, Chatter gives a fresh one to any custom theme
whose id isn't of that form or repeats another's. The editor's rows are a
Slint model only while it's open; otherwise a custom theme costs its parsed
colours and a card.

| Theme | Id | Dark | Accent | Look |
| --- | --- | --- | --- | --- |
| Discord Dark | `yap` | yes | blurple | Discord's current dark theme; the default. The stored `yap` id is kept for existing settings |
| Discord Onyx | `onyx` | yes | blurple | Discord's near-black theme |
| Discord Ash | `ash` | yes | blurple | Discord's classic grey dark theme |
| Midnight | `midnight` | yes | periwinkle | True black for OLED screens |
| Ocean | `ocean` | yes | cyan | Deep navy |
| Forest | `forest` | yes | emerald | Mossy green-grey |
| Crimson | `crimson` | yes | crimson | Warm charcoal with a red tint |
| Sunset | `sunset` | yes | coral | Plum and warm brown, pink links |
| Lavender | `lavender` | yes | lilac | Soft violet-grey, low contrast |
| Mocha | `mocha` | yes | caramel | Coffee browns |
| Discord Light | `light` | no | blurple | Discord's light theme |
| Paper | `paper` | no | teal | Warm off-white |

All are Chatter's own palettes. The Discord ones follow the CSS variables of
Discord's web client: the rail and sidebar are `--background-base-lowest`
(#121214) and the chat `--background-base-lower` (#1a1a1e) on Dark. A layout
that chose a theme this version no longer has shows Discord Dark, and a custom
theme based on one keeps its own colours and takes the rest from Discord Dark.

The ground steps go rail, sidebar, chat from the deepest to the one text sits
on, with raised surfaces and the composer one step off the chat. Light themes
wash hovers with a dark overlay instead of white and draw the tooltip light.

Discord role colours stay as servers set them, and some read faintly on a
light theme; Chatter doesn't adjust them yet.
