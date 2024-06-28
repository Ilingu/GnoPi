use std::time::Duration;

use super::AppMode;

#[derive(Debug, Copy, Clone)]
pub struct AppPreferences {
    pub mode: AppMode,
    pub timeout: Option<Duration>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            mode: AppMode::Visible,
            timeout: None,
        }
    }
}

impl AppPreferences {
    pub fn load() -> Self {
        Self::default()
    }

    pub fn set(new_pref: Self) {}
}
