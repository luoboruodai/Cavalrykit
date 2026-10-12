//! Versioned local preferences. No user files or application state are sent off-device.
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Zh,
    En,
    Ja,
    Ko,
}
impl Language {
    pub const ALL: [Self; 4] = [Self::Zh, Self::En, Self::Ja, Self::Ko];
    pub fn name(self) -> &'static str {
        match self {
            Self::Zh => "简体中文",
            Self::En => "English",
            Self::Ja => "日本語",
            Self::Ko => "한국어",
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::Zh => 0,
            Self::En => 1,
            Self::Ja => 2,
            Self::Ko => 3,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Cavalry,
    Light,
    Custom,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RestartPrompt {
    #[default]
    All,
    Plugins,
    Never,
}
impl RestartPrompt {
    pub fn wants(self, scripts: usize, plugins: usize) -> bool {
        match self {
            Self::All => scripts + plugins > 0,
            Self::Plugins => plugins > 0,
            Self::Never => false,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema: u32,
    pub language: Language,
    pub theme: Theme,
    pub accent: [u8; 3],
    pub background: [u8; 3],
    pub scale_percent: u16,
    pub reduce_motion: bool,
    pub refresh_on_focus: bool,
    pub restart_prompt: RestartPrompt,
    /// Optional application path. Only used when Cavalry is not already running.
    pub cavalry_path: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            schema: 1,
            language: Language::Zh,
            theme: Theme::Cavalry,
            accent: [7, 132, 48],
            background: [19, 23, 31],
            scale_percent: 100,
            reduce_motion: false,
            refresh_on_focus: true,
            restart_prompt: RestartPrompt::All,
            cavalry_path: String::new(),
        }
    }
}
impl Settings {
    pub fn normalized(mut self) -> Self {
        self.scale_percent = self.scale_percent.clamp(90, 125);
        self.cavalry_path = self.cavalry_path.trim().to_owned();
        self
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if meta.len() > 64 * 1024 {
            return Err("Preferences file exceeds 64 KiB".into());
        }
        let value: Self = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if value.schema != 1 {
            return Err("Unsupported preferences schema; original file kept".into());
        }
        Ok(value.normalized())
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let parent = path.parent().ok_or("Missing preferences directory")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        let data =
            serde_json::to_vec_pretty(&self.clone().normalized()).map_err(|e| e.to_string())?;
        temp.write_all(&data).map_err(|e| e.to_string())?;
        temp.as_file().sync_all().map_err(|e| e.to_string())?;
        // persist replaces the destination atomically on supported desktop filesystems.
        temp.persist(path).map_err(|e| e.error.to_string())?;
        Ok(())
    }
}
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("CavalryKit").join("preferences.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistence_is_versioned_atomic_and_clamped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let mut s = Settings {
            language: Language::Ko,
            scale_percent: 1000,
            ..Default::default()
        };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap().scale_percent, 125);
        s.language = Language::Ja;
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap().language, Language::Ja);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn broken_or_newer_preferences_are_not_silently_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("settings.json");
        for data in ["{bad-json", "{\"schema\":9}"] {
            std::fs::write(&p, data).unwrap();
            assert!(Settings::load(&p).is_err());
            assert_eq!(std::fs::read_to_string(&p).unwrap(), data);
        }
        assert!(Settings::default().save(dir.path()).is_err());
    }
    #[test]
    fn restart_prompt_respects_success_and_user_preference() {
        assert!(!RestartPrompt::All.wants(0, 0));
        assert!(RestartPrompt::All.wants(1, 0));
        assert!(!RestartPrompt::Plugins.wants(1, 0));
        assert!(RestartPrompt::Plugins.wants(0, 1));
        assert!(!RestartPrompt::Never.wants(1, 1));
    }
}
