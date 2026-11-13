//! The window's colours: a full set of what `Theme` in `ui/theme.slint` holds,
//! which Chatter sets on the window to change them all at once.

use std::{cell::Cell, rc::Rc};

use slint::{Color, ComponentHandle, Model, ModelRc, VecModel};
use chatter_ui::ChatterWindow;

/// Opaque `#rrggbb`.
const fn rgb(hex: u32) -> Color {
    Color::from_argb_encoded(0xff00_0000 | hex)
}

/// `#rrggbbaa`, as Slint writes a colour with alpha.
const fn rgba(hex: u32) -> Color {
    Color::from_argb_encoded(hex.rotate_right(8))
}

/// Declares `Theme` with one field per colour of the Slint global, and the
/// window's getter and setter for each.
macro_rules! theme {
    ($($name:ident ($get:ident, $set:ident),)*) => {
        /// A full set of the window's colours, and whether they make a dark
        /// theme. `ui/theme.slint` describes each.
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct Theme {
            pub dark: bool,
            $(pub $name: Color,)*
        }

        impl Theme {
            /// Sets only the colours that differ from those the window
            /// shows: nothing when it starts in Chatter, its defaults.
            fn set(&self, tokens: &chatter_ui::Theme<'_>) {
                if tokens.get_dark() != self.dark {
                    tokens.set_dark(self.dark);
                }
                $(if tokens.$get() != self.$name {
                    tokens.$set(self.$name);
                })*
            }

            /// Each colour's name, as `ui/theme.slint` has it but with `_`
            /// for `-`, in the order of [`Self::colour`].
            pub const NAMES: &[&str] = &[$(stringify!($name),)*];

            pub fn colour(&self, index: usize) -> Option<Color> {
                [$(self.$name,)*].get(index).copied()
            }

            pub fn colour_mut(&mut self, index: usize) -> Option<&mut Color> {
                [$(&mut self.$name,)*].into_iter().nth(index)
            }

            /// The colours the window shows now.
            #[cfg(test)]
            pub fn shown(window: &ChatterWindow) -> Self {
                let tokens = window.global::<chatter_ui::Theme>();
                Self { dark: tokens.get_dark(), $($name: tokens.$get(),)* }
            }
        }
    };
}

theme! {
    rail(get_rail, set_rail),
    sidebar(get_sidebar, set_sidebar),
    panel(get_panel, set_panel),
    chat(get_chat, set_chat),
    stage(get_stage, set_stage),
    raised(get_raised, set_raised),
    field(get_field, set_field),
    sunken(get_sunken, set_sunken),
    hover(get_hover, set_hover),
    active(get_active, set_active),
    row_hover(get_row_hover, set_row_hover),
    line(get_line, set_line),
    edge(get_edge, set_edge),
    outline(get_outline, set_outline),
    outline_chat(get_outline_chat, set_outline_chat),
    outline_field(get_outline_field, set_outline_field),
    text(get_text, set_text),
    secondary(get_secondary, set_secondary),
    muted(get_muted, set_muted),
    faint(get_faint, set_faint),
    accent(get_accent, set_accent),
    accent_hover(get_accent_hover, set_accent_hover),
    on_accent(get_on_accent, set_on_accent),
    accent_soft(get_accent_soft, set_accent_soft),
    danger(get_danger, set_danger),
    danger_hover(get_danger_hover, set_danger_hover),
    on_danger(get_on_danger, set_on_danger),
    live(get_live, set_live),
    loud(get_loud, set_loud),
    link(get_link, set_link),
    button_default(get_button_default, set_button_default),
    button_default_hover(get_button_default_hover, set_button_default_hover),
    overlay(get_overlay, set_overlay),
    shimmer(get_shimmer, set_shimmer),
    reply_spine(get_reply_spine, set_reply_spine),
    tooltip(get_tooltip, set_tooltip),
    avatar(get_avatar, set_avatar),
}

/// Discord's current dark theme: soft charcoal panels, a darker rail and channel list, and blurple.
/// The defaults in `ui/theme.slint` are these, so the first frame shows them.
pub const CHATTER_THEME: Theme = Theme {
    dark: true,
    rail: rgb(0x121214),
    sidebar: rgb(0x121214),
    panel: rgb(0x1a1a1e),
    chat: rgb(0x1a1a1e),
    stage: rgb(0x121214),
    raised: rgb(0x242429),
    field: rgb(0x222327),
    sunken: rgb(0x0e0e10),
    hover: rgb(0x1f1f23),
    active: rgb(0x2b2b30),
    row_hover: rgb(0x1d1d21),
    line: rgb(0x29292e),
    outline: rgb(0x222227),
    outline_chat: rgb(0x29292e),
    outline_field: rgb(0x2f2f35),
    edge: rgb(0x08080a),
    text: rgb(0xf0f1f3),
    secondary: rgb(0xb5bac1),
    muted: rgb(0x949ba4),
    faint: rgb(0x6d6f78),
    accent: rgb(0x5865f2),
    accent_hover: rgb(0x6d78f4),
    on_accent: rgb(0xffffff),
    accent_soft: rgba(0x5865f226),
    danger: rgb(0xda373c),
    danger_hover: rgb(0xe5585c),
    on_danger: rgb(0xffffff),
    live: rgb(0x23a55a),
    loud: rgb(0xf0b232),
    link: rgb(0x00a8fc),
    button_default: rgb(0x2c2c32),
    button_default_hover: rgb(0x36363d),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0d),
    reply_spine: rgb(0x4e5058),
    tooltip: rgb(0x0b0b0d),
    avatar: rgb(0x5f6370),
};

/// Discord's darkest theme: near-black panels with a cooler grey and blurple.
const ONYX: Theme = Theme {
    dark: true,
    rail: rgb(0x000000),
    sidebar: rgb(0x070708),
    panel: rgb(0x0c0c0e),
    chat: rgb(0x0c0c0e),
    stage: rgb(0x070708),
    raised: rgb(0x18181b),
    field: rgb(0x151517),
    sunken: rgb(0x000000),
    hover: rgb(0x141416),
    active: rgb(0x1f1f23),
    row_hover: rgb(0x111113),
    line: rgb(0x1d1d20),
    outline: rgb(0x151518),
    outline_chat: rgb(0x1d1d20),
    outline_field: rgb(0x25252a),
    edge: rgb(0x000000),
    text: rgb(0xececee),
    secondary: rgb(0xadb1b8),
    muted: rgb(0x8b8f97),
    faint: rgb(0x5d6068),
    accent: rgb(0x5865f2),
    accent_hover: rgb(0x7480f5),
    on_accent: rgb(0xffffff),
    accent_soft: rgba(0x5865f22b),
    danger: rgb(0xe0464b),
    danger_hover: rgb(0xea6569),
    on_danger: rgb(0xffffff),
    live: rgb(0x2dbf6a),
    loud: rgb(0xf0b232),
    link: rgb(0x3bb7ff),
    button_default: rgb(0x1e1e22),
    button_default_hover: rgb(0x28282d),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0a),
    reply_spine: rgb(0x3c3e45),
    tooltip: rgb(0x1a1a1d),
    avatar: rgb(0x54575e),
};

/// Discord's classic grey dark theme, a step lighter and softer.
const ASH: Theme = Theme {
    dark: true,
    rail: rgb(0x1e1f22),
    sidebar: rgb(0x2b2d31),
    panel: rgb(0x2b2d31),
    chat: rgb(0x313338),
    stage: rgb(0x2b2d31),
    raised: rgb(0x383a40),
    field: rgb(0x383a40),
    sunken: rgb(0x1e1f22),
    hover: rgb(0x35373c),
    active: rgb(0x404249),
    row_hover: rgb(0x2e3035),
    line: rgb(0x3f4147),
    outline: rgb(0x1f2023),
    outline_chat: rgb(0x26282c),
    outline_field: rgb(0x44464d),
    edge: rgb(0x111214),
    text: rgb(0xf2f3f5),
    secondary: rgb(0xb5bac1),
    muted: rgb(0x949ba4),
    faint: rgb(0x6d6f78),
    accent: rgb(0x5865f2),
    accent_hover: rgb(0x4752c4),
    on_accent: rgb(0xffffff),
    accent_soft: rgba(0x5865f230),
    danger: rgb(0xda373c),
    danger_hover: rgb(0xa12d2f),
    on_danger: rgb(0xffffff),
    live: rgb(0x23a55a),
    loud: rgb(0xf0b232),
    link: rgb(0x00a8fc),
    button_default: rgb(0x4e5058),
    button_default_hover: rgb(0x6d6f78),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0f),
    reply_spine: rgb(0x4e5058),
    tooltip: rgb(0x111214),
    avatar: rgb(0x6d6f78),
};

/// True black for OLED screens, with faint blue greys and a periwinkle accent.
const MIDNIGHT: Theme = Theme {
    dark: true,
    rail: rgb(0x050508),
    sidebar: rgb(0x07070a),
    panel: rgb(0x090a0e),
    chat: rgb(0x090a0e),
    stage: rgb(0x07070a),
    raised: rgb(0x161822),
    field: rgb(0x0b0c11),
    sunken: rgb(0x040406),
    hover: rgb(0x0b0c11),
    active: rgb(0x1a1c28),
    row_hover: rgb(0x0a0b10),
    line: rgb(0x171a24),
    outline: rgb(0x0b0c11),
    outline_chat: rgb(0x0d0f15),
    outline_field: rgb(0x1d212e),
    edge: rgb(0x030304),
    text: rgb(0xeaebf0),
    secondary: rgb(0xbabdc9),
    muted: rgb(0x9296a5),
    faint: rgb(0x646878),
    accent: rgb(0x8ea2ff),
    accent_hover: rgb(0xb2bfff),
    on_accent: rgb(0x0b0b10),
    accent_soft: rgba(0x8ea2ff29),
    danger: rgb(0xf2545b),
    danger_hover: rgb(0xf5777c),
    on_danger: rgb(0xffffff),
    live: rgb(0x3dd68c),
    loud: rgb(0xf5b73b),
    link: rgb(0xcbd4ff),
    button_default: rgb(0x0e0f16),
    button_default_hover: rgb(0x11131b),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0b),
    reply_spine: rgb(0x454854),
    tooltip: rgb(0x050608),
    avatar: rgb(0x5a5f72),
};

/// Deep navy layers, like the sea at night, with a clear sky-blue accent.
const OCEAN: Theme = Theme {
    dark: true,
    rail: rgb(0x0a0e12),
    sidebar: rgb(0x0d1319),
    panel: rgb(0x121921),
    chat: rgb(0x121921),
    stage: rgb(0x0d1319),
    raised: rgb(0x19232e),
    field: rgb(0x161f29),
    sunken: rgb(0x070a0d),
    hover: rgb(0x161f29),
    active: rgb(0x1d2937),
    row_hover: rgb(0x151d26),
    line: rgb(0x1b2532),
    outline: rgb(0x161f29),
    outline_chat: rgb(0x1b2532),
    outline_field: rgb(0x222f3f),
    edge: rgb(0x05070a),
    text: rgb(0xeaedf0),
    secondary: rgb(0xbac1c9),
    muted: rgb(0x929ba5),
    faint: rgb(0x646d78),
    accent: rgb(0x38bdf8),
    accent_hover: rgb(0x5ac8f9),
    on_accent: rgb(0x0b0b10),
    accent_soft: rgba(0x38bdf829),
    danger: rgb(0xf2545b),
    danger_hover: rgb(0xf5777c),
    on_danger: rgb(0xffffff),
    live: rgb(0x3dd68c),
    loud: rgb(0xf5b73b),
    link: rgb(0x73d1fa),
    button_default: rgb(0x1c2733),
    button_default_hover: rgb(0x233141),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0b),
    reply_spine: rgb(0x454c54),
    tooltip: rgb(0x0b0f14),
    avatar: rgb(0x5a6572),
};

/// Dark pine greens, calm and low-contrast, with a fresh mint accent.
const FOREST: Theme = Theme {
    dark: true,
    rail: rgb(0x0c110e),
    sidebar: rgb(0x101714),
    panel: rgb(0x151e1a),
    chat: rgb(0x151e1a),
    stage: rgb(0x101714),
    raised: rgb(0x1d2a25),
    field: rgb(0x1a2621),
    sunken: rgb(0x080c0b),
    hover: rgb(0x1a2621),
    active: rgb(0x23322b),
    row_hover: rgb(0x18231e),
    line: rgb(0x1f2d27),
    outline: rgb(0x1a2621),
    outline_chat: rgb(0x1f2d27),
    outline_field: rgb(0x283932),
    edge: rgb(0x060908),
    text: rgb(0xeaf0ee),
    secondary: rgb(0xbac9c3),
    muted: rgb(0x92a59d),
    faint: rgb(0x64786f),
    accent: rgb(0x4ade80),
    accent_hover: rgb(0x68e495),
    on_accent: rgb(0x0b0b10),
    accent_soft: rgba(0x4ade8029),
    danger: rgb(0xf2545b),
    danger_hover: rgb(0xf5777c),
    on_danger: rgb(0xffffff),
    live: rgb(0x3dd68c),
    loud: rgb(0xf5b73b),
    link: rgb(0x5cb8ff),
    button_default: rgb(0x202f29),
    button_default_hover: rgb(0x293b33),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0b),
    reply_spine: rgb(0x45544e),
    tooltip: rgb(0x0d1210),
    avatar: rgb(0x5a7268),
};

/// Warm near-black with a hint of wine, and a rose-red accent.
const CRIMSON: Theme = Theme {
    dark: true,
    rail: rgb(0x110c0c),
    sidebar: rgb(0x171011),
    panel: rgb(0x1e1516),
    chat: rgb(0x1e1516),
    stage: rgb(0x171011),
    raised: rgb(0x2a1d1f),
    field: rgb(0x261a1c),
    sunken: rgb(0x0c0809),
    hover: rgb(0x261a1c),
    active: rgb(0x322325),
    row_hover: rgb(0x23181a),
    line: rgb(0x2d1f22),
    outline: rgb(0x261a1c),
    outline_chat: rgb(0x2d1f22),
    outline_field: rgb(0x39282b),
    edge: rgb(0x090607),
    text: rgb(0xf0eaeb),
    secondary: rgb(0xc9babd),
    muted: rgb(0xa59295),
    faint: rgb(0x786467),
    accent: rgb(0xf43f5e),
    accent_hover: rgb(0xf6617a),
    on_accent: rgb(0xffffff),
    accent_soft: rgba(0xf43f5e29),
    danger: rgb(0xf2545b),
    danger_hover: rgb(0xf5777c),
    on_danger: rgb(0xffffff),
    live: rgb(0x3dd68c),
    loud: rgb(0xf5b73b),
    link: rgb(0x5cb8ff),
    button_default: rgb(0x2f2023),
    button_default_hover: rgb(0x3b292c),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0b),
    reply_spine: rgb(0x544547),
    tooltip: rgb(0x120d0d),
    avatar: rgb(0x725a5e),
};

/// Dusky brown-greys with a warm orange accent, easy on the eyes at night.
const SUNSET: Theme = Theme {
    dark: true,
    rail: rgb(0x110d0b),
    sidebar: rgb(0x17120f),
    panel: rgb(0x1f1814),
    chat: rgb(0x1f1814),
    stage: rgb(0x17120f),
    raised: rgb(0x2b211d),
    field: rgb(0x261e1a),
    sunken: rgb(0x0c0a08),
    hover: rgb(0x261e1a),
    active: rgb(0x322722),
    row_hover: rgb(0x231b17),
    line: rgb(0x2e241f),
    outline: rgb(0x261e1a),
    outline_chat: rgb(0x2e241f),
    outline_field: rgb(0x3a2d27),
    edge: rgb(0x090706),
    text: rgb(0xf0ecea),
    secondary: rgb(0xc9bfba),
    muted: rgb(0xa59892),
    faint: rgb(0x786a64),
    accent: rgb(0xfb923c),
    accent_hover: rgb(0xfca65f),
    on_accent: rgb(0x0b0b10),
    accent_soft: rgba(0xfb923c29),
    danger: rgb(0xf2545b),
    danger_hover: rgb(0xf5777c),
    on_danger: rgb(0xffffff),
    live: rgb(0x3dd68c),
    loud: rgb(0xf5b73b),
    link: rgb(0x5cb8ff),
    button_default: rgb(0x2f2520),
    button_default_hover: rgb(0x3c2e28),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0b),
    reply_spine: rgb(0x544a45),
    tooltip: rgb(0x120e0c),
    avatar: rgb(0x72625a),
};

/// Soft violet greys with a lavender accent.
const LAVENDER: Theme = Theme {
    dark: true,
    rail: rgb(0x0d0b11),
    sidebar: rgb(0x120f18),
    panel: rgb(0x181320),
    chat: rgb(0x181320),
    stage: rgb(0x120f18),
    raised: rgb(0x211b2c),
    field: rgb(0x1e1828),
    sunken: rgb(0x0a080d),
    hover: rgb(0x1e1828),
    active: rgb(0x272034),
    row_hover: rgb(0x1b1624),
    line: rgb(0x241d2f),
    outline: rgb(0x1e1828),
    outline_chat: rgb(0x241d2f),
    outline_field: rgb(0x2d253c),
    edge: rgb(0x070609),
    text: rgb(0xeceaf0),
    secondary: rgb(0xc0bac9),
    muted: rgb(0x9992a5),
    faint: rgb(0x6b6478),
    accent: rgb(0xa78bfa),
    accent_hover: rgb(0xc1adfb),
    on_accent: rgb(0xffffff),
    accent_soft: rgba(0xa78bfa29),
    danger: rgb(0xf2545b),
    danger_hover: rgb(0xf5777c),
    on_danger: rgb(0xffffff),
    live: rgb(0x3dd68c),
    loud: rgb(0xf5b73b),
    link: rgb(0x5cb8ff),
    button_default: rgb(0x251e31),
    button_default_hover: rgb(0x2f263e),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0b),
    reply_spine: rgb(0x4a4554),
    tooltip: rgb(0x0e0c13),
    avatar: rgb(0x635a72),
};

/// Coffee browns with a caramel accent, cosy and muted.
const MOCHA: Theme = Theme {
    dark: true,
    rail: rgb(0x100e0c),
    sidebar: rgb(0x161310),
    panel: rgb(0x1d1916),
    chat: rgb(0x1d1916),
    stage: rgb(0x161310),
    raised: rgb(0x29231f),
    field: rgb(0x24201b),
    sunken: rgb(0x0c0a09),
    hover: rgb(0x24201b),
    active: rgb(0x302a24),
    row_hover: rgb(0x211d19),
    line: rgb(0x2c2621),
    outline: rgb(0x24201b),
    outline_chat: rgb(0x2c2621),
    outline_field: rgb(0x37302a),
    edge: rgb(0x090807),
    text: rgb(0xf0edea),
    secondary: rgb(0xc9c1ba),
    muted: rgb(0xa59b92),
    faint: rgb(0x786d64),
    accent: rgb(0xd9a066),
    accent_hover: rgb(0xe0b283),
    on_accent: rgb(0x0b0b10),
    accent_soft: rgba(0xd9a06629),
    danger: rgb(0xf2545b),
    danger_hover: rgb(0xf5777c),
    on_danger: rgb(0xffffff),
    live: rgb(0x3dd68c),
    loud: rgb(0xf5b73b),
    link: rgb(0x5cb8ff),
    button_default: rgb(0x2d2722),
    button_default_hover: rgb(0x39312b),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0b),
    reply_spine: rgb(0x544c45),
    tooltip: rgb(0x110f0d),
    avatar: rgb(0x72655a),
};

/// Discord's light theme: white chat, light grey sidebars and blurple.
const DISCORD_LIGHT: Theme = Theme {
    dark: false,
    rail: rgb(0xe3e5e8),
    sidebar: rgb(0xf2f3f5),
    panel: rgb(0xf2f3f5),
    chat: rgb(0xffffff),
    stage: rgb(0xf2f3f5),
    raised: rgb(0xffffff),
    field: rgb(0xebedef),
    sunken: rgb(0xe3e5e8),
    hover: rgb(0xe8e9ec),
    active: rgb(0xd7d9dc),
    row_hover: rgb(0xf6f6f7),
    line: rgb(0xe1e2e4),
    outline: rgb(0xe1e2e4),
    outline_chat: rgb(0xe1e2e4),
    outline_field: rgb(0xd4d7dc),
    edge: rgb(0xd4d7dc),
    text: rgb(0x060607),
    secondary: rgb(0x4e5058),
    muted: rgb(0x5c5e66),
    faint: rgb(0x80848e),
    accent: rgb(0x5865f2),
    accent_hover: rgb(0x4752c4),
    on_accent: rgb(0xffffff),
    accent_soft: rgba(0x5865f21f),
    danger: rgb(0xda373c),
    danger_hover: rgb(0xa12d2f),
    on_danger: rgb(0xffffff),
    live: rgb(0x248046),
    loud: rgb(0xc27c0e),
    link: rgb(0x006ce7),
    button_default: rgb(0xe3e5e8),
    button_default_hover: rgb(0xd4d7dc),
    overlay: rgb(0x000000),
    shimmer: rgba(0x0000000a),
    reply_spine: rgb(0xc4c9ce),
    tooltip: rgb(0xffffff),
    avatar: rgb(0x80848e),
};

/// Warm off-white like a notebook page, with ink-dark text and a deep teal accent.
const PAPER: Theme = Theme {
    dark: false,
    rail: rgb(0xe7e0d4),
    sidebar: rgb(0xf2ece2),
    panel: rgb(0xf2ece2),
    chat: rgb(0xfbf8f2),
    stage: rgb(0xf2ece2),
    raised: rgb(0xfffdf9),
    field: rgb(0xede6da),
    sunken: rgb(0xe7e0d4),
    hover: rgb(0xebe4d8),
    active: rgb(0xddd4c5),
    row_hover: rgb(0xf5f1e9),
    line: rgb(0xe3dbcd),
    outline: rgb(0xddd4c5),
    outline_chat: rgb(0xe3dbcd),
    outline_field: rgb(0xd3c9b8),
    edge: rgb(0xcfc5b3),
    text: rgb(0x2a241d),
    secondary: rgb(0x5c5246),
    muted: rgb(0x776b5d),
    faint: rgb(0xa09584),
    accent: rgb(0x0f7f82),
    accent_hover: rgb(0x0b6a6c),
    on_accent: rgb(0xffffff),
    accent_soft: rgba(0x0f7f821f),
    danger: rgb(0xc2402f),
    danger_hover: rgb(0xa33424),
    on_danger: rgb(0xffffff),
    live: rgb(0x2e7d32),
    loud: rgb(0xb86e00),
    link: rgb(0x1f62a8),
    button_default: rgb(0xe6dfd2),
    button_default_hover: rgb(0xdad1c2),
    overlay: rgb(0x000000),
    shimmer: rgba(0x0000000a),
    reply_spine: rgb(0xcdc3b1),
    tooltip: rgb(0xfffdf9),
    avatar: rgb(0x8c8070),
};

/// A theme Chatter comes with: the id a layout stores and the name Appearance
/// shows.
pub struct BuiltIn {
    pub id: &'static str,
    pub name: &'static str,
    pub theme: Theme,
}

/// The themes Chatter comes with, as Appearance lists them: the dark ones, then
/// the light ones.
pub const BUILT_IN: [BuiltIn; 12] = [
    BuiltIn {
        id: "yap",
        name: "Discord Dark",
        theme: CHATTER_THEME,
    },
    BuiltIn {
        id: "onyx",
        name: "Discord Onyx",
        theme: ONYX,
    },
    BuiltIn {
        id: "ash",
        name: "Discord Ash",
        theme: ASH,
    },
    BuiltIn {
        id: "midnight",
        name: "Midnight",
        theme: MIDNIGHT,
    },
    BuiltIn {
        id: "ocean",
        name: "Ocean",
        theme: OCEAN,
    },
    BuiltIn {
        id: "forest",
        name: "Forest",
        theme: FOREST,
    },
    BuiltIn {
        id: "crimson",
        name: "Crimson",
        theme: CRIMSON,
    },
    BuiltIn {
        id: "sunset",
        name: "Sunset",
        theme: SUNSET,
    },
    BuiltIn {
        id: "lavender",
        name: "Lavender",
        theme: LAVENDER,
    },
    BuiltIn {
        id: "mocha",
        name: "Mocha",
        theme: MOCHA,
    },
    BuiltIn {
        id: "light",
        name: "Discord Light",
        theme: DISCORD_LIGHT,
    },
    BuiltIn {
        id: "paper",
        name: "Paper",
        theme: PAPER,
    },
];

pub fn find(id: &str) -> Option<&'static BuiltIn> {
    BUILT_IN.iter().find(|theme| theme.id == id)
}

/// A theme Appearance offers, built in or the person's own, and its id.
pub fn lookup<'a>(id: &str, custom: &'a [Custom]) -> Option<(&'a str, Theme)> {
    find(id)
        .map(|built_in| (built_in.id, built_in.theme))
        .or_else(|| {
            custom
                .iter()
                .find(|custom| custom.id == id)
                .map(|custom| (custom.id.as_str(), custom.theme))
        })
}

/// The theme a layout chose, and its id: Discord Dark when it chose none, or one
/// this version doesn't have or the person deleted.
pub fn chosen<'a>(id: Option<&str>, custom: &'a [Custom]) -> (&'a str, Theme) {
    id.and_then(|id| lookup(id, custom))
        .unwrap_or((BUILT_IN[0].id, CHATTER_THEME))
}

/// A colour's name as `ui/theme.slint` and `layout.json` have it.
fn token(name: &str) -> String {
    name.replace('_', "-")
}

/// Where a colour named as `layout.json` has it is in [`Theme::colour`].
fn token_index(token: &str) -> Option<usize> {
    Theme::NAMES.iter().position(|name| {
        name.bytes()
            .map(|byte| if byte == b'_' { b'-' } else { byte })
            .eq(token.bytes())
    })
}

/// A colour written `#rgb`, `#rrggbb` or `#rrggbbaa`, as Slint writes them.
pub fn parse(text: &str) -> Option<Color> {
    let digits = text.trim().strip_prefix('#')?;
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(digits, 16).ok()?;
    match digits.len() {
        3 => {
            let nibble = |shift: u32| ((value >> shift) & 0xf) * 0x11;
            Some(rgb((nibble(8) << 16) | (nibble(4) << 8) | nibble(0)))
        }
        6 => Some(rgb(value)),
        8 => Some(rgba(value)),
        _ => None,
    }
}

/// A colour as `#rrggbb`, or `#rrggbbaa` when it isn't opaque.
pub fn hex(colour: Color) -> String {
    let rgb = colour.as_argb_encoded() & 0xff_ffff;
    match colour.alpha() {
        0xff => format!("#{rgb:06x}"),
        alpha => format!("#{rgb:06x}{alpha:02x}"),
    }
}

/// One of the person's own themes, as a layout keeps it.
#[derive(Clone, Debug, PartialEq)]
pub struct Custom {
    /// `custom:` and a number: no built-in id has a colon. See [`tidy`].
    pub id: String,
    pub name: String,
    /// The built-in theme it began as a copy of, whose colours stand in for
    /// any it doesn't name, such as one added since it was saved.
    pub base: &'static str,
    pub theme: Theme,
}

/// A custom theme in `layout.json`: each colour as text, by its name in
/// `ui/theme.slint`.
#[derive(Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Stored {
    id: String,
    name: String,
    base: String,
    dark: Option<bool>,
    colours: std::collections::BTreeMap<String, String>,
}

impl From<Stored> for Custom {
    fn from(stored: Stored) -> Self {
        let base = find(&stored.base).unwrap_or(&BUILT_IN[0]);
        let mut theme = base.theme;
        theme.dark = stored.dark.unwrap_or(theme.dark);
        for (token, text) in &stored.colours {
            if let (Some(slot), Some(colour)) = (
                token_index(token).and_then(|index| theme.colour_mut(index)),
                parse(text),
            ) {
                *slot = colour;
            }
        }
        Self {
            id: stored.id,
            name: stored.name,
            base: base.id,
            theme,
        }
    }
}

impl From<&Custom> for Stored {
    fn from(custom: &Custom) -> Self {
        let colours = Theme::NAMES
            .iter()
            .enumerate()
            .filter_map(|(index, name)| Some((token(name), hex(custom.theme.colour(index)?))))
            .collect();
        Self {
            id: custom.id.clone(),
            name: custom.name.clone(),
            base: custom.base.to_owned(),
            dark: Some(custom.theme.dark),
            colours,
        }
    }
}

impl serde::Serialize for Custom {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Stored::from(self).serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Custom {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Stored::deserialize(deserializer).map(Self::from)
    }
}

/// The number in a custom theme's id, if it's one Chatter gives.
fn custom_number(id: &str) -> Option<u32> {
    id.strip_prefix("custom:")?.parse().ok()
}

/// An id for a new custom theme, after those there are.
pub fn new_id(custom: &[Custom]) -> String {
    let last = custom
        .iter()
        .filter_map(|custom| custom_number(&custom.id))
        .max()
        .unwrap_or(0);
    format!("custom:{}", last + 1)
}

/// Gives a new id to each custom theme from `layout.json` whose id Chatter
/// wouldn't have given, or that another has, so that ids stay apart from
/// the built-in ones and from each other.
pub fn tidy(custom: &mut [Custom]) {
    let mut seen = std::collections::HashSet::new();
    let unfit: Vec<usize> = (0..custom.len())
        .filter(|&index| {
            custom_number(&custom[index].id).is_none() || !seen.insert(custom[index].id.clone())
        })
        .collect();
    for index in unfit {
        custom[index].id = new_id(custom);
    }
}

/// A theme a new one can start as.
pub struct Start<'a> {
    pub id: &'a str,
    /// The built-in theme it is or began as.
    pub base: &'static str,
    pub name: &'a str,
    pub theme: Theme,
}

/// What a new theme can start as: every theme Appearance offers, in its
/// order.
pub fn starts(custom: &[Custom]) -> impl Iterator<Item = Start<'_>> {
    BUILT_IN
        .iter()
        .map(|built_in| Start {
            id: built_in.id,
            base: built_in.id,
            name: built_in.name,
            theme: built_in.theme,
        })
        .chain(custom.iter().map(|custom| Start {
            id: &custom.id,
            base: custom.base,
            name: &custom.name,
            theme: custom.theme,
        }))
}

thread_local! {
    /// The colour of a mention without one of its own, as `0xrrggbb`: the
    /// theme's link colour. Rows build their markup with it, outside the
    /// window, on the window's thread.
    static MENTION: Cell<u32> = const { Cell::new(0xc9cdfb) };
    /// The theme's text and muted colours, the same way, for a notice's
    /// name, links and time.
    static TEXT: Cell<u32> = const { Cell::new(0xf0f1f3) };
    static MUTED: Cell<u32> = const { Cell::new(0x949ba4) };
}

pub fn mention_color() -> u32 {
    MENTION.get()
}

pub fn text_color() -> u32 {
    TEXT.get()
}

pub fn muted_color() -> u32 {
    MUTED.get()
}

impl Theme {
    /// Shows the window in these colours. Returns whether colours that rows
    /// carry in their markup change, as rows built before have the old ones.
    pub fn apply(&self, window: &ChatterWindow) -> bool {
        self.set(&window.global::<chatter_ui::Theme>());
        let rgb = |colour: Color| colour.as_argb_encoded() & 0xff_ffff;
        // The Discord themes draw a mention's text as Discord does, a light
        // blurple; the other themes use their link colour.
        let mention_colour = if self.dark && rgb(self.link) == rgb(CHATTER_THEME.link) {
            0xc9cdfb
        } else {
            rgb(self.link)
        };
        let mention = MENTION.replace(mention_colour) != mention_colour;
        let text = TEXT.replace(rgb(self.text)) != rgb(self.text);
        let muted = MUTED.replace(rgb(self.muted)) != rgb(self.muted);
        mention || text || muted
    }
}

/// Appearance's cards: each theme's name and the colours its preview draws
/// with, the built-in ones first.
pub fn cards(custom: &[Custom]) -> ModelRc<chatter_ui::ThemeCard> {
    let card = |id: &str, name: &str, custom: bool, theme: &Theme| chatter_ui::ThemeCard {
        id: id.into(),
        name: name.into(),
        custom,
        rail: theme.rail,
        sidebar: theme.sidebar,
        chat: theme.chat,
        raised: theme.raised,
        accent: theme.accent,
        text: theme.text,
        muted: theme.muted,
    };
    let cards: Vec<_> = BUILT_IN
        .iter()
        .map(|built_in| card(built_in.id, built_in.name, false, &built_in.theme))
        .chain(
            custom
                .iter()
                .map(|custom| card(&custom.id, &custom.name, true, &custom.theme)),
        )
        .collect();
    ModelRc::new(VecModel::from(cards))
}

/// Appearance's theme editor while it's open: the theme as edited so far,
/// and a row for each colour, the model it lends the window until it closes.
pub struct Editor {
    /// The custom theme being edited, by id; `None` for a new one.
    pub editing: Option<String>,
    pub base: &'static str,
    pub theme: Theme,
    rows: Rc<VecModel<chatter_ui::ThemeColour>>,
}

impl Editor {
    pub fn new(editing: Option<String>, base: &'static str, theme: Theme) -> Self {
        let editor = Self {
            editing,
            base,
            theme,
            rows: Rc::default(),
        };
        editor.fill();
        editor
    }

    /// Writes every row again, which also gives each field its text anew.
    fn fill(&self) {
        let rows: Vec<_> = Theme::NAMES
            .iter()
            .enumerate()
            .filter_map(|(index, name)| {
                let colour = self.theme.colour(index)?;
                Some(chatter_ui::ThemeColour {
                    name: token(name).into(),
                    colour,
                    hex: hex(colour).into(),
                    wrong: false,
                })
            })
            .collect();
        self.rows.set_vec(rows);
    }

    pub fn rows(&self) -> ModelRc<chatter_ui::ThemeColour> {
        ModelRc::from(self.rows.clone())
    }

    /// Starts again from another theme's colours.
    pub fn restart(&mut self, base: &'static str, theme: Theme) {
        self.base = base;
        self.theme = theme;
        self.fill();
    }

    /// Takes a colour's text as typed: the theme takes the colour if it is
    /// one, and the row says so if not. Returns whether the theme changed.
    pub fn edit(&mut self, index: usize, text: &str) -> bool {
        let Some(mut row) = self.rows.row_data(index) else {
            return false;
        };
        let colour = parse(text);
        row.hex = text.into();
        row.wrong = colour.is_none();
        row.colour = colour.unwrap_or(row.colour);
        self.rows.set_row_data(index, row);
        match (colour, self.theme.colour_mut(index)) {
            (Some(colour), Some(slot)) if *slot != colour => {
                *slot = colour;
                true
            }
            _ => false,
        }
    }

    /// Whether every field holds a colour.
    pub fn valid(&self) -> bool {
        !self.rows.iter().any(|row| row.wrong)
    }
}
