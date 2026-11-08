//! The window's colours: a full set of what `Theme` in `ui/theme.slint` holds,
//! which Static sets on the window to change them all at once.

use std::{cell::Cell, rc::Rc};

use slint::{Color, ComponentHandle, Model, ModelRc, VecModel};
use static_ui::StaticWindow;

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
            /// shows: nothing when it starts in Static, its defaults.
            fn set(&self, tokens: &static_ui::Theme<'_>) {
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
            pub fn shown(window: &StaticWindow) -> Self {
                let tokens = window.global::<static_ui::Theme>();
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
pub const STATIC_THEME: Theme = Theme {
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

/// True black for OLED screens, with blue-tinted greys and a soft periwinkle accent.
const MIDNIGHT: Theme = Theme {
    dark: true,
    rail: rgb(0x000000),
    sidebar: rgb(0x000000),
    panel: rgb(0x05060a),
    chat: rgb(0x000000),
    stage: rgb(0x05060a),
    raised: rgb(0x10121a),
    field: rgb(0x0c0e15),
    sunken: rgb(0x000000),
    hover: rgb(0x0d1018),
    active: rgb(0x161a26),
    row_hover: rgb(0x080a10),
    line: rgb(0x151925),
    outline: rgb(0x10131c),
    outline_chat: rgb(0x151925),
    outline_field: rgb(0x1d2232),
    edge: rgb(0x000000),
    text: rgb(0xe6e9f2),
    secondary: rgb(0xa3abc2),
    muted: rgb(0x7d86a0),
    faint: rgb(0x4d5470),
    accent: rgb(0x8ea2ff),
    accent_hover: rgb(0xaab9ff),
    on_accent: rgb(0x070a1a),
    accent_soft: rgba(0x8ea2ff24),
    danger: rgb(0xff5d6c),
    danger_hover: rgb(0xff7d89),
    on_danger: rgb(0x0a0000),
    live: rgb(0x4ade80),
    loud: rgb(0xfbbf24),
    link: rgb(0x7cc8ff),
    button_default: rgb(0x141826),
    button_default_hover: rgb(0x1d2234),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0a),
    reply_spine: rgb(0x2a3045),
    tooltip: rgb(0x11141e),
    avatar: rgb(0x3d4560),
};

/// Deep navy layers like the sea at night, with a bright cyan accent.
const OCEAN: Theme = Theme {
    dark: true,
    rail: rgb(0x08101c),
    sidebar: rgb(0x0d1726),
    panel: rgb(0x0d1726),
    chat: rgb(0x111d30),
    stage: rgb(0x0d1726),
    raised: rgb(0x18273e),
    field: rgb(0x162438),
    sunken: rgb(0x070e19),
    hover: rgb(0x15233a),
    active: rgb(0x1d3150),
    row_hover: rgb(0x0f1a2c),
    line: rgb(0x1c2c44),
    outline: rgb(0x0b1422),
    outline_chat: rgb(0x1a2940),
    outline_field: rgb(0x22344f),
    edge: rgb(0x04080f),
    text: rgb(0xe4eef8),
    secondary: rgb(0xa3b8cf),
    muted: rgb(0x7f96b0),
    faint: rgb(0x506680),
    accent: rgb(0x38bdf8),
    accent_hover: rgb(0x67cdfa),
    on_accent: rgb(0x04131f),
    accent_soft: rgba(0x38bdf826),
    danger: rgb(0xf87171),
    danger_hover: rgb(0xfb8f8f),
    on_danger: rgb(0x1a0505),
    live: rgb(0x34d399),
    loud: rgb(0xfbbf24),
    link: rgb(0x93c5fd),
    button_default: rgb(0x1c2c44),
    button_default_hover: rgb(0x253857),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0c),
    reply_spine: rgb(0x2c4262),
    tooltip: rgb(0x050b14),
    avatar: rgb(0x3b5578),
};

/// Mossy green-grey surfaces after rain, with an emerald accent.
const FOREST: Theme = Theme {
    dark: true,
    rail: rgb(0x0c120f),
    sidebar: rgb(0x121a16),
    panel: rgb(0x121a16),
    chat: rgb(0x161f1a),
    stage: rgb(0x121a16),
    raised: rgb(0x1e2a23),
    field: rgb(0x1b2620),
    sunken: rgb(0x0a0f0c),
    hover: rgb(0x1a251f),
    active: rgb(0x243329),
    row_hover: rgb(0x141c18),
    line: rgb(0x22302a),
    outline: rgb(0x0e1512),
    outline_chat: rgb(0x1f2b25),
    outline_field: rgb(0x2a3a31),
    edge: rgb(0x060907),
    text: rgb(0xe5eee8),
    secondary: rgb(0xa9bcaf),
    muted: rgb(0x86998c),
    faint: rgb(0x56685c),
    accent: rgb(0x3ddc97),
    accent_hover: rgb(0x66e5ad),
    on_accent: rgb(0x06140e),
    accent_soft: rgba(0x3ddc9724),
    danger: rgb(0xf26b5b),
    danger_hover: rgb(0xf58a7d),
    on_danger: rgb(0x1a0603),
    live: rgb(0xa3e635),
    loud: rgb(0xf5b84a),
    link: rgb(0x7dd3c0),
    button_default: rgb(0x22302a),
    button_default_hover: rgb(0x2c3d35),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0b),
    reply_spine: rgb(0x34473c),
    tooltip: rgb(0x070b09),
    avatar: rgb(0x4a6455),
};

/// Warm charcoal with a deep red tint and a bold crimson accent.
const CRIMSON: Theme = Theme {
    dark: true,
    rail: rgb(0x0f0b0c),
    sidebar: rgb(0x171112),
    panel: rgb(0x171112),
    chat: rgb(0x1c1516),
    stage: rgb(0x171112),
    raised: rgb(0x281d1f),
    field: rgb(0x241a1c),
    sunken: rgb(0x0b0809),
    hover: rgb(0x231a1b),
    active: rgb(0x332427),
    row_hover: rgb(0x1a1314),
    line: rgb(0x2d2123),
    outline: rgb(0x120d0e),
    outline_chat: rgb(0x2a1f21),
    outline_field: rgb(0x3a2a2d),
    edge: rgb(0x070505),
    text: rgb(0xf5ecec),
    secondary: rgb(0xc4b0b2),
    muted: rgb(0xa08a8d),
    faint: rgb(0x6e5a5d),
    accent: rgb(0xe5364f),
    accent_hover: rgb(0xec5d72),
    on_accent: rgb(0xffffff),
    accent_soft: rgba(0xe5364f26),
    danger: rgb(0xff7a45),
    danger_hover: rgb(0xff9568),
    on_danger: rgb(0x1a0800),
    live: rgb(0x4ade80),
    loud: rgb(0xffc247),
    link: rgb(0xff9eb0),
    button_default: rgb(0x2f2224),
    button_default_hover: rgb(0x3c2b2e),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0c),
    reply_spine: rgb(0x4a3437),
    tooltip: rgb(0x080506),
    avatar: rgb(0x6b4a50),
};

/// Dusky plum and warm brown under an orange sky, with a coral accent and pink links.
const SUNSET: Theme = Theme {
    dark: true,
    rail: rgb(0x140e13),
    sidebar: rgb(0x1c141a),
    panel: rgb(0x1c141a),
    chat: rgb(0x22181f),
    stage: rgb(0x1c141a),
    raised: rgb(0x2f2129),
    field: rgb(0x2a1d25),
    sunken: rgb(0x100b0f),
    hover: rgb(0x291d24),
    active: rgb(0x3a2832),
    row_hover: rgb(0x1f161c),
    line: rgb(0x34252e),
    outline: rgb(0x160f14),
    outline_chat: rgb(0x30222a),
    outline_field: rgb(0x44303b),
    edge: rgb(0x0a0609),
    text: rgb(0xfbeee8),
    secondary: rgb(0xd1b7ae),
    muted: rgb(0xab918a),
    faint: rgb(0x76605c),
    accent: rgb(0xff8a5b),
    accent_hover: rgb(0xffa47e),
    on_accent: rgb(0x1f0d05),
    accent_soft: rgba(0xff8a5b26),
    danger: rgb(0xff5c7a),
    danger_hover: rgb(0xff7d96),
    on_danger: rgb(0x1a0207),
    live: rgb(0x5eead4),
    loud: rgb(0xffc04d),
    link: rgb(0xff9ec7),
    button_default: rgb(0x36262f),
    button_default_hover: rgb(0x44303b),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0c),
    reply_spine: rgb(0x55394a),
    tooltip: rgb(0x0b070a),
    avatar: rgb(0x7a5163),
};

/// Soft violet-grey surfaces, calm and low contrast, with a lilac accent.
const LAVENDER: Theme = Theme {
    dark: true,
    rail: rgb(0x131119),
    sidebar: rgb(0x1a1723),
    panel: rgb(0x1a1723),
    chat: rgb(0x1f1b2a),
    stage: rgb(0x1a1723),
    raised: rgb(0x2a2538),
    field: rgb(0x262133),
    sunken: rgb(0x0f0d15),
    hover: rgb(0x242031),
    active: rgb(0x322b45),
    row_hover: rgb(0x1c1926),
    line: rgb(0x2e2840),
    outline: rgb(0x15121c),
    outline_chat: rgb(0x2a253a),
    outline_field: rgb(0x3a3352),
    edge: rgb(0x09080d),
    text: rgb(0xefeaf8),
    secondary: rgb(0xbdb3d4),
    muted: rgb(0x9a90b3),
    faint: rgb(0x665e80),
    accent: rgb(0xb69cff),
    accent_hover: rgb(0xc9b5ff),
    on_accent: rgb(0x160f2c),
    accent_soft: rgba(0xb69cff26),
    danger: rgb(0xff6b8a),
    danger_hover: rgb(0xff8aa3),
    on_danger: rgb(0x1a0309),
    live: rgb(0x6ee7b7),
    loud: rgb(0xfcd34d),
    link: rgb(0x9ecbff),
    button_default: rgb(0x2e2840),
    button_default_hover: rgb(0x3a3352),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0c),
    reply_spine: rgb(0x463e60),
    tooltip: rgb(0x0b0a10),
    avatar: rgb(0x5f5680),
};

/// Roasted coffee browns, warm and easy on the eyes at night, with a caramel accent.
const MOCHA: Theme = Theme {
    dark: true,
    rail: rgb(0x15110e),
    sidebar: rgb(0x1d1814),
    panel: rgb(0x1d1814),
    chat: rgb(0x231d18),
    stage: rgb(0x1d1814),
    raised: rgb(0x2f2720),
    field: rgb(0x2b231d),
    sunken: rgb(0x110e0b),
    hover: rgb(0x29221c),
    active: rgb(0x3a3027),
    row_hover: rgb(0x201a16),
    line: rgb(0x352c24),
    outline: rgb(0x17130f),
    outline_chat: rgb(0x312820),
    outline_field: rgb(0x443830),
    edge: rgb(0x0a0806),
    text: rgb(0xf3ebe2),
    secondary: rgb(0xcbbcab),
    muted: rgb(0xa7968a),
    faint: rgb(0x74665a),
    accent: rgb(0xe0a96d),
    accent_hover: rgb(0xeabf8f),
    on_accent: rgb(0x1c1209),
    accent_soft: rgba(0xe0a96d26),
    danger: rgb(0xec6a5e),
    danger_hover: rgb(0xf0897f),
    on_danger: rgb(0x1a0603),
    live: rgb(0x9ccc65),
    loud: rgb(0xf6c453),
    link: rgb(0x8fc1e3),
    button_default: rgb(0x372e26),
    button_default_hover: rgb(0x453a30),
    overlay: rgb(0xffffff),
    shimmer: rgba(0xffffff0c),
    reply_spine: rgb(0x574839),
    tooltip: rgb(0x0b0907),
    avatar: rgb(0x7a6450),
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

/// A theme Static comes with: the id a layout stores and the name Appearance
/// shows.
pub struct BuiltIn {
    pub id: &'static str,
    pub name: &'static str,
    pub theme: Theme,
}

/// The themes Static comes with, as Appearance lists them: the dark ones, then
/// the light ones.
pub const BUILT_IN: [BuiltIn; 12] = [
    BuiltIn {
        id: "yap",
        name: "Discord Dark",
        theme: STATIC_THEME,
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
        .unwrap_or((BUILT_IN[0].id, STATIC_THEME))
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

/// The number in a custom theme's id, if it's one Static gives.
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

/// Gives a new id to each custom theme from `layout.json` whose id Static
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
    pub fn apply(&self, window: &StaticWindow) -> bool {
        self.set(&window.global::<static_ui::Theme>());
        let rgb = |colour: Color| colour.as_argb_encoded() & 0xff_ffff;
        // The Discord themes draw a mention's text as Discord does, a light
        // blurple; the other themes use their link colour.
        let mention_colour = if self.dark && rgb(self.link) == rgb(STATIC_THEME.link) {
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
pub fn cards(custom: &[Custom]) -> ModelRc<static_ui::ThemeCard> {
    let card = |id: &str, name: &str, custom: bool, theme: &Theme| static_ui::ThemeCard {
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
    rows: Rc<VecModel<static_ui::ThemeColour>>,
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
                Some(static_ui::ThemeColour {
                    name: token(name).into(),
                    colour,
                    hex: hex(colour).into(),
                    wrong: false,
                })
            })
            .collect();
        self.rows.set_vec(rows);
    }

    pub fn rows(&self) -> ModelRc<static_ui::ThemeColour> {
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
