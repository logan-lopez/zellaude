//! Claude theme palettes and the role mapping the status bar draws from.
//!
//! Palettes:
//! - Dark: base24 `claude-dark` (sampled from Claude desktop app; neutral surfaces, warm text, Anthropic accents).
//! - Light: base24 `claude-light` (derived from Claude Dark; warm light surfaces, dark warm text, Anthropic accents lowered to clear 4.5:1).

use zellij_tile::prelude::{HostTerminalThemeMode, PaletteColor, Style};

pub type Color = (u8, u8, u8);

/// A colour and its variant for the active tab or selected surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accent {
    pub base: Color,
    pub selected: Color,
}

impl Accent {
    pub const fn new(base: Color, selected: Color) -> Self {
        Self { base, selected }
    }

    pub const fn pick(&self, selected: bool) -> Color {
        if selected {
            self.selected
        } else {
            self.base
        }
    }
}

/// A complete palette containing every role needed to render the status bar.
#[derive(Clone, Copy, Debug)]
pub struct ThemePalette {
    // Surfaces
    pub bar_bg: Color,
    pub prefix_bg: Color,
    pub prefix_bg_settings: Color,
    pub tab_bg_inactive: Color,
    pub tab_bg_active: Color,
    pub tab_marker: Color,
    pub flash_bg: Accent,
    pub text_on_flash: Accent,

    // Text
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_tertiary: Color,
    pub text_on_accent: Color,

    // Menu
    pub menu_on: Color,
    pub menu_partial: Color,
    pub menu_off: Color,
    pub menu_close: Color,

    // Glyphs
    pub fullscreen: Color,
    pub elapsed: Accent,

    // Activities
    pub act_init: Accent,
    pub act_thinking: Accent,
    pub act_tool: Accent,
    pub act_prompting: Accent,
    pub act_waiting: Accent,
    pub act_notification: Accent,
    pub act_done: Accent,
    pub act_agent_done: Accent,
    pub act_idle: Accent,

    // Input modes
    pub mode_normal: Color,
    pub mode_locked: Color,
    pub mode_pane: Color,
    pub mode_tab: Color,
    pub mode_resize: Color,
    pub mode_scroll: Color,
    pub mode_search: Color,
    pub mode_rename: Color,
    pub mode_session: Color,
}

// ── base24: claude-dark ──────────────────────────────────────────────────────
// Neutral surfaces: #151515, #1d1d1d, #353535, #111111
// Warm text ramp: #7d7c76, #bbbaaf, #edede9
// Accents: #d7757e (red), #d97757 (orange), #cfae6e (yellow), #94b87a (green),
//          #72b6b4 (cyan), #6a9bcc (blue), #b68dc4 (purple), #e78e96 (bright red),
//          #a8ce8c (bright green)
pub const DARK_PALETTE: ThemePalette = ThemePalette {
    bar_bg: (17, 17, 17),                  // BASE10 #111111
    prefix_bg: (29, 29, 29),               // BASE01 #1d1d1d
    prefix_bg_settings: (217, 119, 87),    // BASE09 #d97757
    tab_bg_inactive: (29, 29, 29),          // BASE01 #1d1d1d
    tab_bg_active: (217, 119, 87),         // BASE09 #d97757
    tab_marker: (217, 119, 87),            // BASE09 #d97757
    flash_bg: Accent::new((217, 119, 87), (53, 53, 53)), // BASE09, BASE02
    text_on_flash: Accent::new((21, 21, 21), (237, 237, 233)), // BASE00, BASE05

    text_primary: (237, 237, 233),         // BASE05 #edede9
    text_secondary: (187, 186, 175),       // BASE04 #bbbaaf
    text_tertiary: (125, 124, 118),        // BASE03 #7d7c76
    text_on_accent: (21, 21, 21),          // BASE00 #151515

    menu_on: (148, 184, 122),              // BASE0B #94b87a
    menu_partial: (207, 174, 110),         // BASE0A #cfae6e
    menu_off: (125, 124, 118),             // BASE03 #7d7c76
    menu_close: (215, 117, 126),           // BASE08 #d7757e

    fullscreen: (207, 174, 110),           // BASE0A #cfae6e
    elapsed: Accent::new((125, 124, 118), (21, 21, 21)), // BASE03, BASE00

    act_init: Accent::new((187, 186, 175), (21, 21, 21)),       // BASE04, TEXT_ON_ACCENT
    act_thinking: Accent::new((182, 141, 196), (21, 21, 21)),   // BASE0E, TEXT_ON_ACCENT
    act_tool: Accent::new((217, 119, 87), (21, 21, 21)),       // BASE09, TEXT_ON_ACCENT
    act_prompting: Accent::new((148, 184, 122), (21, 21, 21)),  // BASE0B, TEXT_ON_ACCENT
    act_waiting: Accent::new((231, 142, 150), (21, 21, 21)),    // BASE12, TEXT_ON_ACCENT
    act_notification: Accent::new((207, 174, 110), (21, 21, 21)),// BASE0A, TEXT_ON_ACCENT
    act_done: Accent::new((168, 206, 140), (21, 21, 21)),       // BASE14, TEXT_ON_ACCENT
    act_agent_done: Accent::new((148, 184, 122), (21, 21, 21)), // BASE0B, TEXT_ON_ACCENT
    act_idle: Accent::new((125, 124, 118), (21, 21, 21)),       // BASE03, TEXT_ON_ACCENT

    mode_normal: (148, 184, 122),          // BASE0B #94b87a
    mode_locked: (215, 117, 126),          // BASE08 #d7757e
    mode_pane: (106, 155, 204),            // BASE0D #6a9bcc
    mode_tab: (182, 141, 196),             // BASE0E #b68dc4
    mode_resize: (217, 119, 87),           // BASE09 #d97757
    mode_scroll: (207, 174, 110),          // BASE0A #cfae6e
    mode_search: (207, 174, 110),          // BASE0A #cfae6e
    mode_rename: (114, 182, 180),          // BASE0C #72b6b4
    mode_session: (182, 141, 196),         // BASE0E #b68dc4
};

// ── base24: claude-light ─────────────────────────────────────────────────────
// Warm light surfaces: #f7f7f5, #fcfcfa, #e7e7e5, #f5f5f3
// Warm dark text: #7e7d77, #515047, #0d0d0b
// Accents (lowered lightness): #b0535d, #b35536, #8b6c2c, #597a3f,
//                             #377b7a, #4675a4, #8a6397, #a03a45, #45662a
pub const LIGHT_PALETTE: ThemePalette = ThemePalette {
    bar_bg: (245, 245, 243),               // BASE10 #f5f5f3
    prefix_bg: (252, 252, 250),            // BASE01 #fcfcfa
    prefix_bg_settings: (179, 85, 54),     // BASE09 #b35536
    tab_bg_inactive: (252, 252, 250),       // BASE01 #fcfcfa
    tab_bg_active: (179, 85, 54),          // BASE09 #b35536
    tab_marker: (179, 85, 54),             // BASE09 #b35536
    flash_bg: Accent::new((179, 85, 54), (231, 231, 229)), // BASE09, BASE02
    text_on_flash: Accent::new((252, 252, 250), (13, 13, 11)), // BASE01, BASE05

    text_primary: (13, 13, 11),            // BASE05 #0d0d0b
    text_secondary: (81, 80, 71),          // BASE04 #515047
    text_tertiary: (126, 125, 119),        // BASE03 #7e7d77
    text_on_accent: (252, 252, 250),       // BASE01 #fcfcfa (light ink on dark coral)

    menu_on: (89, 122, 63),                // BASE0B #597a3f
    menu_partial: (139, 108, 44),          // BASE0A #8b6c2c
    menu_off: (126, 125, 119),             // BASE03 #7e7d77
    menu_close: (176, 83, 93),             // BASE08 #b0535d

    fullscreen: (139, 108, 44),            // BASE0A #8b6c2c
    elapsed: Accent::new((126, 125, 119), (252, 252, 250)), // BASE03, BASE01

    act_init: Accent::new((81, 80, 71), (252, 252, 250)),       // BASE04, TEXT_ON_ACCENT
    act_thinking: Accent::new((138, 99, 151), (252, 252, 250)), // BASE0E, TEXT_ON_ACCENT
    act_tool: Accent::new((179, 85, 54), (252, 252, 250)),     // BASE09, TEXT_ON_ACCENT
    act_prompting: Accent::new((89, 122, 63), (252, 252, 250)),// BASE0B, TEXT_ON_ACCENT
    act_waiting: Accent::new((160, 58, 69), (252, 252, 250)),  // BASE12, TEXT_ON_ACCENT
    act_notification: Accent::new((139, 108, 44), (252, 252, 250)), // BASE0A, TEXT_ON_ACCENT
    act_done: Accent::new((69, 102, 42), (252, 252, 250)),     // BASE14, TEXT_ON_ACCENT
    act_agent_done: Accent::new((89, 122, 63), (252, 252, 250)),// BASE0B, TEXT_ON_ACCENT
    act_idle: Accent::new((126, 125, 119), (252, 252, 250)),   // BASE03, TEXT_ON_ACCENT

    mode_normal: (89, 122, 63),            // BASE0B #597a3f
    mode_locked: (176, 83, 93),            // BASE08 #b0535d
    mode_pane: (70, 117, 164),             // BASE0D #4675a4
    mode_tab: (138, 99, 151),              // BASE0E #8a6397
    mode_resize: (179, 85, 54),            // BASE09 #b35536
    mode_scroll: (139, 108, 44),           // BASE0A #8b6c2c
    mode_search: (139, 108, 44),           // BASE0A #8b6c2c
    mode_rename: (55, 123, 122),           // BASE0C #377b7a
    mode_session: (138, 99, 151),          // BASE0E #8a6397
};

pub fn get_palette(theme: HostTerminalThemeMode) -> &'static ThemePalette {
    match theme {
        HostTerminalThemeMode::Dark => &DARK_PALETTE,
        HostTerminalThemeMode::Light => &LIGHT_PALETTE,
    }
}

pub fn eight_bit_to_rgb(code: u8) -> (u8, u8, u8) {
    match code {
        0 => (0, 0, 0),
        1 => (128, 0, 0),
        2 => (0, 128, 0),
        3 => (128, 128, 0),
        4 => (0, 0, 128),
        5 => (128, 0, 128),
        6 => (0, 128, 128),
        7 => (192, 192, 192),
        8 => (128, 128, 128),
        9 => (255, 0, 0),
        10 => (0, 255, 0),
        11 => (255, 255, 0),
        12 => (0, 0, 255),
        13 => (255, 0, 255),
        14 => (0, 255, 255),
        15 => (255, 255, 255),
        16..=231 => {
            let i = code - 16;
            let b = i % 6;
            let g = (i / 6) % 6;
            let r = i / 36;
            let val = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
            (val(r), val(g), val(b))
        }
        232..=255 => {
            let gray = 8 + (code - 232) * 10;
            (gray, gray, gray)
        }
    }
}

pub fn relative_luminance((r, g, b): Color) -> f64 {
    fn channel(v: u8) -> f64 {
        let v = v as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

pub fn is_light_color(r: u8, g: u8, b: u8) -> bool {
    relative_luminance((r, g, b)) > 0.5
}

pub fn classify_palette_color(color: PaletteColor) -> HostTerminalThemeMode {
    let (r, g, b) = match color {
        PaletteColor::Rgb(rgb) => rgb,
        PaletteColor::EightBit(code) => eight_bit_to_rgb(code),
    };
    if is_light_color(r, g, b) {
        HostTerminalThemeMode::Light
    } else {
        HostTerminalThemeMode::Dark
    }
}

pub fn infer_theme_from_style(style: &Style) -> Option<HostTerminalThemeMode> {
    let bg = style.colors.text_unselected.background;
    Some(classify_palette_color(bg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eight_bit_to_rgb() {
        assert_eq!(eight_bit_to_rgb(0), (0, 0, 0));
        assert_eq!(eight_bit_to_rgb(15), (255, 255, 255));
        assert_eq!(eight_bit_to_rgb(238), (68, 68, 68)); // Zellij default GRAY
    }

    #[test]
    fn test_classify_palette_color() {
        // Dark RGB
        assert_eq!(
            classify_palette_color(PaletteColor::Rgb((21, 21, 21))),
            HostTerminalThemeMode::Dark
        );
        // Light RGB
        assert_eq!(
            classify_palette_color(PaletteColor::Rgb((247, 247, 245))),
            HostTerminalThemeMode::Light
        );
        // 8-bit dark (gray 238)
        assert_eq!(
            classify_palette_color(PaletteColor::EightBit(238)),
            HostTerminalThemeMode::Dark
        );
        // 8-bit light (bright white 15)
        assert_eq!(
            classify_palette_color(PaletteColor::EightBit(15)),
            HostTerminalThemeMode::Light
        );
    }

    #[test]
    fn test_get_palette() {
        let dark = get_palette(HostTerminalThemeMode::Dark);
        assert_eq!(dark.bar_bg, (17, 17, 17));
        assert_eq!(dark.text_on_accent, (21, 21, 21));

        let light = get_palette(HostTerminalThemeMode::Light);
        assert_eq!(light.bar_bg, (245, 245, 243));
        assert_eq!(light.text_on_accent, (252, 252, 250));
    }
}
