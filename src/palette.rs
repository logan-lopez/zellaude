//! claude-dark palette and the role mapping the status bar draws from.
//!
//! Palette source: base24 `claude-dark` (see `claude-dark-theme.md`). Surface and
//! text values were sampled from the Claude desktop app; the accents are the
//! Anthropic brand orange/blue plus invented hues matched to them.
//!
//! Two properties of the palette that this mapping preserves:
//!
//!   1. Surfaces are neutral greys (R == G == B); the warmth lives in the text
//!      ramp. Do not tint a background to "match the feel".
//!   2. Coral (`BASE09`) marks things that *mean* something — a running tool, an
//!      open menu, a permission request. It is not the wash under every active
//!      row. Selection stays neutral (`BASE01`/`BASE02`).
//!
//! To retheme the bar, change the `base*` constants and leave the role
//! constants below alone.

pub type Color = (u8, u8, u8);

// ── base24: claude-dark ──────────────────────────────────────────────────────
// Neutral surfaces
pub const BASE00: Color = (21, 21, 21); // #151515 canvas
pub const BASE01: Color = (29, 29, 29); // #1d1d1d raised surface
pub const BASE02: Color = (53, 53, 53); // #353535 selection / active row
pub const BASE10: Color = (17, 17, 17); // #111111 window chrome (below canvas)
#[allow(dead_code)]
pub const BASE11: Color = (14, 14, 14); // #0e0e0e darkest
#[allow(dead_code)]
pub const HAIRLINE: Color = (42, 42, 42); // #2a2a2a dividers (sampled, unnumbered)

// Warm text ramp
pub const BASE03: Color = (125, 124, 118); // #7d7c76 tertiary text
pub const BASE04: Color = (187, 186, 175); // #bbbaaf secondary text
pub const BASE05: Color = (237, 237, 233); // #edede9 primary text

// Accents
pub const BASE08: Color = (215, 117, 126); // #d7757e red
pub const BASE09: Color = (217, 119, 87); // #d97757 orange (Anthropic brand)
pub const BASE0A: Color = (207, 174, 110); // #cfae6e yellow
pub const BASE0B: Color = (148, 184, 122); // #94b87a green
pub const BASE0C: Color = (114, 182, 180); // #72b6b4 cyan
pub const BASE0D: Color = (106, 155, 204); // #6a9bcc blue (Anthropic brand)
pub const BASE0E: Color = (182, 141, 196); // #b68dc4 purple
#[allow(dead_code)]
pub const BASE0F: Color = (180, 151, 126); // #b4977e brown

/// Bright coral. base24 defines no bright twin for `BASE09`, so this is derived
/// by the palette's own documented rule — +8 lightness, +10 saturation in HLS —
/// which reproduces every published base12–base17 value byte-for-byte from its
/// base08–base0E source. **Derived, not sampled.**
pub const BASE09_BRIGHT: Color = (233, 142, 112); // #e98e70

// Bright accents
pub const BASE12: Color = (231, 142, 150); // #e78e96 bright red
#[allow(dead_code)]
pub const BASE13: Color = (225, 194, 133); // #e1c285 bright yellow
pub const BASE14: Color = (168, 206, 140); // #a8ce8c bright green
#[allow(dead_code)]
pub const BASE15: Color = (132, 205, 202); // #84cdca bright cyan
#[allow(dead_code)]
pub const BASE16: Color = (128, 175, 222); // #80afde bright blue
#[allow(dead_code)]
pub const BASE17: Color = (202, 161, 217); // #caa1d9 bright purple

// ── Role mapping ─────────────────────────────────────────────────────────────
// Surfaces, darkest to lightest: bar ground → tab → active tab.
// The bar is chrome, so it sits *below* the canvas the panes draw on — that
// direction is deliberate and matches the app's sidebar.

/// Ground the bar fills with, and the gap between adjacent tabs.
pub const BAR_BG: Color = BASE10;
/// The clickable "Zellaude (session)" chip.
pub const PREFIX_BG: Color = BASE01;
/// Same chip while the settings menu is open — coral because it marks state.
pub const PREFIX_BG_SETTINGS: Color = BASE09;
/// Inactive tab.
pub const TAB_BG_INACTIVE: Color = BASE01;
/// Active tab: neutral selection, not a coral wash (see rule 2 above).
pub const TAB_BG_ACTIVE: Color = BASE02;
/// Marks the active tab's leading separator, so position reads at a glance
/// without tinting the tab surface.
pub const TAB_MARKER: Color = BASE09;
/// Permission-request pulse. The one place coral is allowed to be a surface:
/// it means something and it is asking for you.
pub const FLASH_BG: Color = BASE09;

// Text
pub const TEXT_PRIMARY: Color = BASE05;
pub const TEXT_SECONDARY: Color = BASE04;
pub const TEXT_TERTIARY: Color = BASE03;
/// Text drawn on top of a coral or accent surface.
pub const TEXT_ON_ACCENT: Color = BASE00;

// Menu
pub const MENU_ON: Color = BASE0B;
pub const MENU_PARTIAL: Color = BASE0A;
pub const MENU_OFF: Color = BASE03;
pub const MENU_CLOSE: Color = BASE08;

// Misc glyphs
pub const FULLSCREEN: Color = BASE0A;
pub const ELAPSED: Accent = Accent::new(BASE03, BASE04);

/// A colour and its variant for the selected (`BASE02`) surface.
///
/// The theme does this itself: its `text_selected` / `table_cell_selected`
/// blocks swap the `base0*` accents for the brighter `base1*` ones, because the
/// selected surface is 24 points lighter and the base accents lose contrast on
/// it. Every accent the bar draws over a tab follows the same rule.
#[derive(Clone, Copy)]
pub struct Accent {
    pub base: Color,
    pub selected: Color,
}

impl Accent {
    const fn new(base: Color, selected: Color) -> Self {
        Self { base, selected }
    }

    /// Flat accent — same colour on both surfaces.
    const fn flat(c: Color) -> Self {
        Self { base: c, selected: c }
    }

    pub const fn pick(&self, selected: bool) -> Color {
        if selected {
            self.selected
        } else {
            self.base
        }
    }
}

// Activity symbols. Contrast on both tab surfaces is checked in
// `tests::accents_are_legible_on_both_tab_surfaces`.
pub const ACT_INIT: Accent = Accent::flat(BASE04);
pub const ACT_THINKING: Accent = Accent::new(BASE0E, BASE17);
pub const ACT_TOOL: Accent = Accent::new(BASE09, BASE09_BRIGHT);
pub const ACT_PROMPTING: Accent = Accent::new(BASE0B, BASE14);
pub const ACT_WAITING: Accent = Accent::flat(BASE12);
pub const ACT_NOTIFICATION: Accent = Accent::new(BASE0A, BASE13);
pub const ACT_DONE: Accent = Accent::flat(BASE14);
pub const ACT_AGENT_DONE: Accent = Accent::new(BASE0B, BASE14);
/// Deliberately the quietest glyph in the bar; still lifted on the selected
/// surface, where `BASE03` would fall to 2.9:1.
pub const ACT_IDLE: Accent = Accent::new(BASE03, BASE04);

// Input modes
pub const MODE_NORMAL: Color = BASE0B;
pub const MODE_LOCKED: Color = BASE08;
pub const MODE_PANE: Color = BASE0D;
pub const MODE_TAB: Color = BASE0E;
pub const MODE_RESIZE: Color = BASE09;
pub const MODE_SCROLL: Color = BASE0A;
pub const MODE_SEARCH: Color = BASE0A;
pub const MODE_RENAME: Color = BASE0C;
pub const MODE_SESSION: Color = BASE0E;
