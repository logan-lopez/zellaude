use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};
use zellij_tile::prelude::*;

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub const FLASH_DURATION_MS: u64 = 2000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Activity {
    Init,
    Thinking,
    Tool(String),
    Prompting,
    Waiting,
    Notification,
    Done,
    AgentDone,
    Idle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub pane_id: u32,
    pub activity: Activity,
    pub tab_name: Option<String>,
    pub tab_index: Option<usize>,
    pub last_event_ts: u64,
    pub cwd: Option<String>,
    #[serde(default)]
    pub last_ts_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct HookPayload {
    pub session_id: Option<String>,
    pub pane_id: u32,
    pub hook_event: String,
    pub tool_name: Option<String>,
    pub cwd: Option<String>,
    pub zellij_session: Option<String>,
    pub term_program: Option<String>,
    pub ts_ms: Option<u64>,
}

pub struct ClickRegion {
    pub start_col: usize,
    pub end_col: usize,
    pub tab_index: usize,
    pub pane_id: u32,
    pub is_waiting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum NotifyMode {
    Never,
    Unfocused,
    #[default]
    Always,
}

impl NotifyMode {
    pub fn cycle(self) -> Self {
        match self {
            Self::Always => Self::Unfocused,
            Self::Unfocused => Self::Never,
            Self::Never => Self::Always,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum FlashMode {
    Off,
    #[default]
    Once,
    Persist,
}

impl FlashMode {
    pub fn cycle(self) -> Self {
        match self {
            Self::Once => Self::Persist,
            Self::Persist => Self::Off,
            Self::Off => Self::Once,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemePreference {
    #[default]
    Auto,
    Light,
    Dark,
}

impl ThemePreference {
    pub fn cycle(self) -> Self {
        match self {
            Self::Auto => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::Auto,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub notifications: NotifyMode,
    pub flash: FlashMode,
    pub elapsed_time: bool,
    pub mode_indicator: bool,
    pub theme: ThemePreference,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notifications: NotifyMode::Always,
            flash: FlashMode::Once,
            elapsed_time: true,
            mode_indicator: true,
            theme: ThemePreference::Auto,
        }
    }
}

#[derive(Default, PartialEq)]
pub enum ViewMode {
    #[default]
    Normal,
    Settings,
}

#[derive(Clone, Copy)]
pub enum SettingKey {
    Notifications,
    Flash,
    ElapsedTime,
    ModeIndicator,
    Theme,
}

pub enum MenuAction {
    ToggleSetting(SettingKey),
    CloseMenu,
}

pub struct MenuClickRegion {
    pub start_col: usize,
    pub end_col: usize,
    pub action: MenuAction,
}

#[derive(Default)]
pub struct State {
    pub sessions: BTreeMap<u32, SessionInfo>,
    pub pane_to_tab: HashMap<u32, (usize, String)>,
    pub tabs: Vec<TabInfo>,
    pub pane_manifest: Option<PaneManifest>,
    pub active_tab_index: Option<usize>,
    pub click_regions: Vec<ClickRegion>,
    /// pane_id -> flash deadline in ms (for waiting animation)
    pub flash_deadlines: HashMap<u32, u64>,
    pub zellij_session_name: Option<String>,
    pub term_program: Option<String>,
    pub input_mode: InputMode,
    pub settings: Settings,
    pub view_mode: ViewMode,
    pub prefix_click_region: Option<(usize, usize)>,
    pub menu_click_regions: Vec<MenuClickRegion>,
    pub config_loaded: bool,
    pub hooks_installed: bool,
    pub detected_host_theme: Option<HostTerminalThemeMode>,
    pub inferred_theme: Option<HostTerminalThemeMode>,
}

impl State {
    pub fn resolved_theme(&self) -> HostTerminalThemeMode {
        match self.settings.theme {
            ThemePreference::Dark => HostTerminalThemeMode::Dark,
            ThemePreference::Light => HostTerminalThemeMode::Light,
            ThemePreference::Auto => {
                if let Some(host) = self.detected_host_theme {
                    host
                } else if let Some(inferred) = self.inferred_theme {
                    inferred
                } else {
                    HostTerminalThemeMode::Dark
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default_theme() {
        let settings = Settings::default();
        assert_eq!(settings.theme, ThemePreference::Auto);
    }

    #[test]
    fn test_settings_deserialize_old_json() {
        let json = r#"{"notifications":"Always","flash":"Once","elapsed_time":true,"mode_indicator":true}"#;
        let settings: Settings = serde_json::from_str(json).expect("deserialize old settings");
        assert_eq!(settings.theme, ThemePreference::Auto);
    }

    #[test]
    fn test_settings_deserialize_new_json() {
        let json = r#"{"notifications":"Always","flash":"Once","elapsed_time":true,"mode_indicator":true,"theme":"Light"}"#;
        let settings: Settings = serde_json::from_str(json).expect("deserialize new settings");
        assert_eq!(settings.theme, ThemePreference::Light);
    }

    #[test]
    fn test_theme_preference_cycle() {
        assert_eq!(ThemePreference::Auto.cycle(), ThemePreference::Light);
        assert_eq!(ThemePreference::Light.cycle(), ThemePreference::Dark);
        assert_eq!(ThemePreference::Dark.cycle(), ThemePreference::Auto);
    }

    #[test]
    fn test_resolved_theme_precedence() {
        let mut state = State::default();
        // Precedence 4: neither signal -> Dark
        assert_eq!(state.resolved_theme(), HostTerminalThemeMode::Dark);

        // Precedence 3: infer from Zellij background when no host theme
        state.inferred_theme = Some(HostTerminalThemeMode::Light);
        assert_eq!(state.resolved_theme(), HostTerminalThemeMode::Light);

        state.inferred_theme = Some(HostTerminalThemeMode::Dark);
        assert_eq!(state.resolved_theme(), HostTerminalThemeMode::Dark);

        // Precedence 2: host theme overrides inferred
        state.inferred_theme = Some(HostTerminalThemeMode::Dark);
        state.detected_host_theme = Some(HostTerminalThemeMode::Light);
        assert_eq!(state.resolved_theme(), HostTerminalThemeMode::Light);

        // Precedence 1: manual override beats host theme
        state.settings.theme = ThemePreference::Dark;
        assert_eq!(state.resolved_theme(), HostTerminalThemeMode::Dark);

        state.settings.theme = ThemePreference::Light;
        assert_eq!(state.resolved_theme(), HostTerminalThemeMode::Light);

        // Switching back to Auto immediately respects the detected host theme
        state.settings.theme = ThemePreference::Auto;
        assert_eq!(state.resolved_theme(), HostTerminalThemeMode::Light);
    }
}
