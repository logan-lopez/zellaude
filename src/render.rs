use crate::palette::{Accent, Color, ThemePalette};
use crate::state::{
    unix_now, unix_now_ms, Activity, ClickRegion, FlashMode, MenuAction, MenuClickRegion,
    NotifyMode, SessionInfo, SettingKey, State, ThemePreference, ViewMode,
};
use std::fmt::Write;
use std::io::Write as IoWrite;
use zellij_tile::prelude::{InputMode, TabInfo};

struct Style {
    symbol: &'static str,
    color: Accent,
}

fn activity_priority(activity: &Activity) -> u8 {
    match activity {
        Activity::Waiting => 8,
        Activity::Tool(_) => 7,
        Activity::Thinking => 6,
        Activity::Prompting => 5,
        Activity::Notification => 4,
        Activity::Init => 3,
        Activity::Done => 2,
        Activity::AgentDone => 1,
        Activity::Idle => 0,
    }
}

fn activity_style(activity: &Activity, pal: &ThemePalette) -> Style {
    match activity {
        Activity::Init => Style { symbol: "◆", color: pal.act_init },
        Activity::Thinking => Style { symbol: "●", color: pal.act_thinking },
        Activity::Tool(name) => {
            let symbol = match name.as_str() {
                "Bash" => "⚡",
                "Read" | "Glob" | "Grep" => "◉",
                "Edit" | "Write" => "✎",
                "Task" => "⊜",
                "WebSearch" | "WebFetch" => "◈",
                _ => "⚙",
            };
            Style { symbol, color: pal.act_tool }
        }
        Activity::Prompting => Style { symbol: "▶", color: pal.act_prompting },
        Activity::Waiting => Style { symbol: "⚠", color: pal.act_waiting },
        Activity::Notification => Style { symbol: "◇", color: pal.act_notification },
        Activity::Done => Style { symbol: "✓", color: pal.act_done },
        Activity::AgentDone => Style { symbol: "✓", color: pal.act_agent_done },
        Activity::Idle => Style { symbol: "○", color: pal.act_idle },
    }
}

/// SGR truecolor foreground.
fn fgc((r, g, b): Color) -> String {
    format!("\x1b[38;2;{r};{g};{b}m")
}

/// SGR truecolor background.
fn bgc((r, g, b): Color) -> String {
    format!("\x1b[48;2;{r};{g};{b}m")
}

fn display_width(s: &str) -> usize {
    s.chars().count()
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const ELAPSED_THRESHOLD: u64 = 30;
const SEPARATOR: &str = "\u{e0b0}";

/// Write a powerline arrow: fg=from_bg, bg=to_bg, then separator char.
fn arrow(buf: &mut String, col: &mut usize, from: Color, to: Color) {
    arrow_tinted(buf, col, from, to);
}

/// Same, but the glyph itself is drawn in `tint` rather than in `from`.
/// Used to flag the active tab with a coral tick without tinting its surface.
fn arrow_tinted(buf: &mut String, col: &mut usize, tint: Color, to: Color) {
    let _ = write!(buf, "{}{}{SEPARATOR}", fgc(tint), bgc(to));
    *col += 1;
}

fn format_elapsed(secs: u64) -> String {
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h", secs / 3600)
    }
}

fn mode_style(mode: InputMode, pal: &ThemePalette) -> (Color, &'static str) {
    match mode {
        InputMode::Normal => (pal.mode_normal, "NORMAL"),
        InputMode::Locked => (pal.mode_locked, "LOCKED"),
        InputMode::Pane => (pal.mode_pane, "PANE"),
        InputMode::Tab => (pal.mode_tab, "TAB"),
        InputMode::Resize => (pal.mode_resize, "RESIZE"),
        InputMode::Move => (pal.mode_resize, "MOVE"),
        InputMode::Scroll => (pal.mode_scroll, "SCROLL"),
        InputMode::EnterSearch => (pal.mode_search, "SEARCH"),
        InputMode::Search => (pal.mode_search, "SEARCH"),
        InputMode::RenameTab => (pal.mode_rename, "RENAME"),
        InputMode::RenamePane => (pal.mode_rename, "RENAME"),
        InputMode::Session => (pal.mode_session, "SESSION"),
        InputMode::Prompt => (pal.mode_normal, "PROMPT"),
        InputMode::Tmux => (pal.mode_normal, "TMUX"),
    }
}

pub fn render_status_bar(state: &mut State, _rows: usize, cols: usize) {
    state.click_regions.clear();
    state.menu_click_regions.clear();

    let theme = state.resolved_theme();
    let pal = crate::palette::get_palette(theme);

    let mut buf = String::with_capacity(cols * 4);
    // Terminal setup for a 1-row status bar:
    //  \x1b[H     — cursor home (prevent scroll from cursor at end-of-line)
    //  \x1b[?7l   — disable auto-wrap (clip overflow instead of scroll)
    //  \x1b[?25l  — hide cursor
    buf.push_str("\x1b[H\x1b[?7l\x1b[?25l");
    let bar_bg_str = bgc(pal.bar_bg);

    // Bail early if terminal is too narrow
    if cols < 5 {
        let _ = write!(buf, "{bar_bg_str}{:width$}{RESET}", "", width = cols);
        print!("{buf}");
        let _ = std::io::stdout().flush();
        return;
    }

    let (prefix_bg, prefix_fg) = if state.view_mode == ViewMode::Settings {
        (pal.prefix_bg_settings, pal.text_on_accent)
    } else {
        (pal.prefix_bg, pal.text_primary)
    };

    // Build prefix: " Zellaude (session) MODE "
    let (mode_bg, mode_text) = mode_style(state.input_mode, pal);
    let show_mode = state.settings.mode_indicator;
    let session_part = match state.zellij_session_name.as_deref() {
        Some(name) => format!(" ({name})"),
        None => String::new(),
    };
    let prefix_text = format!(" Zellaude{session_part} ");
    let prefix_width = display_width(&prefix_text);
    let mode_pill_width = if show_mode { 1 + mode_text.len() + 1 } else { 0 };
    let total_prefix_width = prefix_width + mode_pill_width;

    // Render prefix segment (truncate if wider than cols)
    let mut col;
    if total_prefix_width <= cols {
        let _ = write!(
            buf,
            "{}{}{BOLD}{prefix_text}{RESET}",
            bgc(prefix_bg),
            fgc(prefix_fg),
        );
        if show_mode {
            let _ = write!(
                buf,
                "{}{}{BOLD} {mode_text} {RESET}",
                bgc(mode_bg),
                fgc(pal.text_on_accent),
            );
        }
        col = total_prefix_width;
    } else if prefix_width <= cols {
        // Fit the name part but skip mode pill
        let _ = write!(
            buf,
            "{}{}{BOLD}{prefix_text}{RESET}",
            bgc(prefix_bg),
            fgc(prefix_fg),
        );
        col = prefix_width;
    } else {
        // Even name doesn't fit — just show what we can
        let avail = cols.saturating_sub(2); // leave room for fill
        let short: String = prefix_text.chars().take(avail).collect();
        let _ = write!(
            buf,
            "{}{}{BOLD}{short}{RESET}",
            bgc(prefix_bg),
            fgc(prefix_fg),
        );
        col = display_width(&short);
    }
    state.prefix_click_region = Some((0, col));

    let last_prefix_bg = if show_mode && total_prefix_width <= cols { mode_bg } else { prefix_bg };
    let prefix_used = col;

    if col < cols {
        match state.view_mode {
            ViewMode::Normal => {
                render_tabs(state, pal, &mut buf, &mut col, cols, last_prefix_bg, prefix_used);
            }
            ViewMode::Settings => {
                arrow(&mut buf, &mut col, last_prefix_bg, pal.bar_bg);
                let _ = write!(buf, "{bar_bg_str}");
                render_settings_menu(state, pal, &mut buf, &mut col);
            }
        }
    }

    // Fill remaining width with bar background — never exceed cols
    if col < cols {
        let remaining = cols - col;
        let _ = write!(buf, "{bar_bg_str}{:width$}", "", width = remaining);
    }
    let _ = write!(buf, "{RESET}");

    print!("{buf}");
    let _ = std::io::stdout().flush();
}

fn render_tabs(
    state: &mut State,
    pal: &ThemePalette,
    buf: &mut String,
    col: &mut usize,
    cols: usize,
    prefix_bg: Color,
    prefix_width: usize,
) {
    let now_s = unix_now();
    let now_ms = unix_now_ms();

    // Sort tabs by position
    let mut tabs: Vec<&TabInfo> = state.tabs.iter().collect();
    tabs.sort_by_key(|t| t.position);

    let count = tabs.len();
    if count == 0 {
        arrow(buf, col, prefix_bg, pal.bar_bg);
        return;
    }

    // For each tab, find the best (highest-priority) Claude session
    let best_sessions: Vec<Option<&SessionInfo>> = tabs
        .iter()
        .map(|tab| {
            state
                .sessions
                .values()
                .filter(|s| s.tab_index == Some(tab.position))
                .max_by_key(|s| activity_priority(&s.activity))
        })
        .collect();

    // Pre-compute elapsed strings (only for Claude tabs)
    let elapsed_strs: Vec<Option<String>> = best_sessions
        .iter()
        .map(|session: &Option<&SessionInfo>| {
            if !state.settings.elapsed_time {
                return None;
            }
            session.and_then(|s| {
                let elapsed = now_s.saturating_sub(s.last_event_ts);
                if elapsed >= ELAPSED_THRESHOLD {
                    Some(format_elapsed(elapsed))
                } else {
                    None
                }
            })
        })
        .collect();

    // Compute overhead: varies per tab type
    let total_elapsed_width: usize = elapsed_strs
        .iter()
        .map(|e: &Option<String>| e.as_ref().map_or(0, |s| s.len() + 1))
        .sum();
    let per_tab_overhead: usize = best_sessions
        .iter()
        .map(|s: &Option<&SessionInfo>| if s.is_some() { 4 } else { 2 })
        .sum();
    let overhead = prefix_width + 2 * count + per_tab_overhead + total_elapsed_width;
    let max_name_len = if overhead < cols {
        ((cols - overhead) / count).min(20)
    } else {
        0
    };

    let mut prev_bg = prefix_bg;
    // Tracked explicitly rather than inferred from `prev_bg == prefix_bg`: under
    // a neutral theme the prefix chip and an inactive tab share a surface colour,
    // and comparing colours would collapse the gap between adjacent tabs.
    let mut rendered_any = false;

    for (i, tab) in tabs.iter().enumerate() {
        // Stop if we'd overflow — need room for at least arrow + closing arrow
        let arrows_needed = if rendered_any { 2 } else { 1 };
        if *col + arrows_needed + 3 > cols {
            break;
        }

        let session = best_sessions[i];
        let is_claude = session.is_some();
        let tab_name = &tab.name;

        // Truncate name
        let char_count = tab_name.chars().count();
        let truncated = if max_name_len == 0 {
            String::new()
        } else if char_count > max_name_len {
            let s: String = tab_name.chars().take(max_name_len.saturating_sub(1)).collect();
            format!("{s}…")
        } else {
            tab_name.to_string()
        };

        // Check flash for any session in this tab
        let is_flash_bright = state
            .sessions
            .values()
            .filter(|s| s.tab_index == Some(tab.position))
            .any(|s| {
                state
                    .flash_deadlines
                    .get(&s.pane_id)
                    .map(|&deadline| now_ms < deadline && (now_ms / 250) % 2 == 0)
                    .unwrap_or(false)
            });

        let is_active = tab.active;

        // Pick tab background color
        let tab_bg = if is_flash_bright {
            pal.flash_bg.pick(is_active)
        } else if is_active {
            pal.tab_bg_active
        } else {
            pal.tab_bg_inactive
        };

        // Arrow: close previous segment, then open this tab. The opening arrow
        // of the active tab is drawn in coral — a one-column "you are here" tick
        // that leaves the tab's own surface neutral.
        let open_from = if rendered_any { pal.bar_bg } else { prev_bg };
        let open_tint = if is_active { pal.tab_marker } else { open_from };
        if rendered_any {
            arrow(buf, col, prev_bg, pal.bar_bg);
        }
        arrow_tinted(buf, col, open_tint, tab_bg);

        let tab_bg_str = bgc(tab_bg);
        let region_start = *col;

        if is_claude {
            let s = session.unwrap();
            let style = activity_style(&s.activity, pal);

            let (sym_fg, name_fg, name_bold) = if is_flash_bright {
                let on_flash = fgc(pal.text_on_flash.pick(is_active));
                (on_flash.clone(), on_flash, true)
            } else if is_active {
                (fgc(style.color.pick(true)), fgc(pal.text_on_accent), true)
            } else {
                (fgc(style.color.pick(false)), fgc(pal.text_secondary), false)
            };

            // Leading space
            let _ = write!(buf, "{tab_bg_str} ");
            *col += 1;

            // Symbol
            let _ = write!(buf, "{sym_fg}{}", style.symbol);
            *col += display_width(style.symbol);

            // Space + name
            if !truncated.is_empty() {
                let bold_str = if name_bold { BOLD } else { "" };
                let _ = write!(buf, " {bold_str}{name_fg}{truncated}{RESET}{tab_bg_str}");
                *col += 1 + display_width(&truncated);
            }

            // Elapsed suffix
            if let Some(ref es) = elapsed_strs[i] {
                if *col + 1 + es.len() + 1 < cols {
                    let _ = write!(buf, " {}{es}", fgc(pal.elapsed.pick(is_active)));
                    *col += 1 + es.len();
                }
            }

            // Fullscreen indicator
            if tab.is_fullscreen_active && *col + 3 < cols {
                let _ = write!(buf, " {}F{RESET}{tab_bg_str}", fgc(pal.fullscreen));
                *col += 2;
            }

            // Trailing space
            let _ = write!(buf, " ");
            *col += 1;

            // Click region: if any session is waiting, use its pane_id for focus
            let waiting_session = state
                .sessions
                .values()
                .filter(|s| s.tab_index == Some(tab.position))
                .find(|s| matches!(s.activity, Activity::Waiting));

            state.click_regions.push(ClickRegion {
                start_col: region_start,
                end_col: *col,
                tab_index: tab.position,
                pane_id: waiting_session.map_or(0, |s| s.pane_id),
                is_waiting: waiting_session.is_some(),
            });
        } else {
            // Non-Claude tab: dimmer, no symbol
            let name_fg = if is_active {
                fgc(pal.text_on_accent)
            } else {
                fgc(pal.text_tertiary)
            };
            let name_bold = is_active;

            // Leading space
            let _ = write!(buf, "{tab_bg_str} ");
            *col += 1;

            // Name only (no symbol)
            if !truncated.is_empty() {
                let bold_str = if name_bold { BOLD } else { "" };
                let _ = write!(buf, "{bold_str}{name_fg}{truncated}{RESET}{tab_bg_str}");
                *col += display_width(&truncated);
            }

            // Fullscreen indicator
            if tab.is_fullscreen_active && *col + 3 < cols {
                let _ = write!(buf, " {}F{RESET}{tab_bg_str}", fgc(pal.fullscreen));
                *col += 2;
            }

            // Trailing space
            let _ = write!(buf, " ");
            *col += 1;

            state.click_regions.push(ClickRegion {
                start_col: region_start,
                end_col: *col,
                tab_index: tab.position,
                pane_id: 0,
                is_waiting: false,
            });
        }

        prev_bg = tab_bg;
        rendered_any = true;
    }

    // Arrow from last tab → bar background (only if we rendered any tabs)
    if prev_bg != prefix_bg || count > 0 {
        arrow(buf, col, prev_bg, pal.bar_bg);
    }
}

fn notify_mode_label(mode: NotifyMode, pal: &ThemePalette) -> (&'static str, &'static str, String, String) {
    match mode {
        NotifyMode::Always => ("●", "Notify: always", fgc(pal.menu_on), fgc(pal.text_primary)),
        NotifyMode::Unfocused => ("◐", "Notify: unfocused", fgc(pal.menu_partial), fgc(pal.menu_partial)),
        NotifyMode::Never => ("○", "Notify: off", fgc(pal.menu_off), fgc(pal.menu_off)),
    }
}

fn flash_mode_label(mode: FlashMode, pal: &ThemePalette) -> (&'static str, &'static str, String, String) {
    match mode {
        FlashMode::Persist => ("●", "Flash: persist", fgc(pal.menu_on), fgc(pal.text_primary)),
        FlashMode::Once => ("◐", "Flash: brief", fgc(pal.menu_partial), fgc(pal.menu_partial)),
        FlashMode::Off => ("○", "Flash: off", fgc(pal.menu_off), fgc(pal.menu_off)),
    }
}

fn theme_mode_label(mode: ThemePreference, pal: &ThemePalette) -> (&'static str, &'static str, String, String) {
    match mode {
        ThemePreference::Auto => ("◐", "Theme: auto", fgc(pal.menu_partial), fgc(pal.text_primary)),
        ThemePreference::Light => ("●", "Theme: light", fgc(pal.menu_on), fgc(pal.text_primary)),
        ThemePreference::Dark => ("○", "Theme: dark", fgc(pal.menu_on), fgc(pal.text_primary)),
    }
}

/// Render a three-state toggle and register its click region.
/// Assumes the caller has already set the desired background color.
fn render_tristate(
    buf: &mut String,
    col: &mut usize,
    state_regions: &mut Vec<MenuClickRegion>,
    key: SettingKey,
    symbol: &str,
    label: &str,
    sym_color: &str,
    label_color: &str,
) {
    let region_start = *col;
    let width = display_width(symbol) + 1 + label.len();
    *col += width;

    state_regions.push(MenuClickRegion {
        start_col: region_start,
        end_col: *col,
        action: MenuAction::ToggleSetting(key),
    });

    let _ = write!(buf, "{sym_color}{symbol} {label_color}{label}");
}

fn render_settings_menu(state: &mut State, pal: &ThemePalette, buf: &mut String, col: &mut usize) {
    // Leading space after arrow
    let _ = write!(buf, " ");
    *col += 1;

    // --- Notifications (three-state) ---
    {
        let (symbol, label, sym_color, label_color) =
            notify_mode_label(state.settings.notifications, pal);
        render_tristate(
            buf, col, &mut state.menu_click_regions,
            SettingKey::Notifications, symbol, label, &sym_color, &label_color,
        );
    }

    // --- Flash (three-state) ---
    {
        let _ = write!(buf, "  ");
        *col += 2;
        let (symbol, label, sym_color, label_color) =
            flash_mode_label(state.settings.flash, pal);
        render_tristate(
            buf, col, &mut state.menu_click_regions,
            SettingKey::Flash, symbol, label, &sym_color, &label_color,
        );
    }

    // --- Elapsed time (bool) ---
    {
        let _ = write!(buf, "  ");
        *col += 2;
        let enabled = state.settings.elapsed_time;
        let (symbol, sym_color, label_color) = if enabled {
            ("●", fgc(pal.menu_on), fgc(pal.text_primary))
        } else {
            ("○", fgc(pal.menu_off), fgc(pal.menu_off))
        };
        let label = if enabled { "Elapsed time: on" } else { "Elapsed time: off" };
        render_tristate(
            buf, col, &mut state.menu_click_regions,
            SettingKey::ElapsedTime, symbol, label, &sym_color, &label_color,
        );
    }

    // --- Mode indicator (bool) ---
    {
        let _ = write!(buf, "  ");
        *col += 2;
        let enabled = state.settings.mode_indicator;
        let (symbol, sym_color, label_color) = if enabled {
            ("●", fgc(pal.menu_on), fgc(pal.text_primary))
        } else {
            ("○", fgc(pal.menu_off), fgc(pal.menu_off))
        };
        let label = if enabled { "Mode indicator: on" } else { "Mode indicator: off" };
        render_tristate(
            buf, col, &mut state.menu_click_regions,
            SettingKey::ModeIndicator, symbol, label, &sym_color, &label_color,
        );
    }

    // --- Theme (three-state: auto / light / dark) ---
    {
        let _ = write!(buf, "  ");
        *col += 2;
        let (symbol, label, sym_color, label_color) =
            theme_mode_label(state.settings.theme, pal);
        render_tristate(
            buf, col, &mut state.menu_click_regions,
            SettingKey::Theme, symbol, label, &sym_color, &label_color,
        );
    }

    // Close button
    let _ = write!(buf, "  ");
    *col += 2;
    let close_start = *col;
    let _ = write!(buf, "{}×", fgc(pal.menu_close));
    *col += 1;

    state.menu_click_regions.push(MenuClickRegion {
        start_col: close_start,
        end_col: *col,
        action: MenuAction::CloseMenu,
    });
}
