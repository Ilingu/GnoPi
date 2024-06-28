use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    time::Duration,
};

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

/// Simple macro to return default when error (can be seen as an enhance '?')
macro_rules! tod {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => return Self::default(),
        }
    };
}

impl AppPreferences {
    /// return the path to the app's config file (and ensure that all the necessary directories and files exists)
    fn get_config_file_path() -> Result<PathBuf, ()> {
        let mut config_path = dirs::config_dir().ok_or(())?;

        config_path.push("gnopi");
        fs::create_dir_all(&config_path).map_err(|_| ())?;

        config_path.push("preferences");
        if !Path::exists(&config_path) {
            File::create(&config_path).map_err(|_| ())?;
        }

        Ok(config_path)
    }

    fn as_bytes(&self) -> [u8; 5] {
        let mut bytes = [0; 5];
        bytes[0] = self.mode as u8; // if less than 255 appmode it should be ok...

        let timeout_bytes = self.timeout.unwrap_or_default().as_secs_f32().to_be_bytes();
        bytes[1..].copy_from_slice(&timeout_bytes);

        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() != 5 {
            return Err(()); // data corrupted
        }

        let mode = AppMode::try_from(bytes[0])?;

        let raw_timeout = f32::from_be_bytes(bytes[1..].try_into().unwrap());
        if raw_timeout < 0.0 {
            return Err(()); // data corrupted
        }
        let timeout = match raw_timeout == 0.0 {
            true => None,
            false => Some(Duration::from_secs_f32(raw_timeout)),
        };

        Ok(AppPreferences { mode, timeout })
    }

    pub fn load() -> Self {
        let config_file_path = tod!(Self::get_config_file_path());
        let bytes = tod!(fs::read(config_file_path));
        tod!(Self::from_bytes(&bytes))
    }

    pub fn set(new_pref: Self) -> Result<(), ()> {
        let config_file_path = Self::get_config_file_path()?;
        fs::write(config_file_path, new_pref.as_bytes()).map_err(|_| ())
    }
}
