use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};
use tauri::Manager;
use tauri_plugin_global_shortcut::Shortcut;

pub const DEFAULT_MODEL: &str = "google/gemini-2.5-flash-lite";

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub instructions: String,
    pub shortcut: String,
    pub model: String,
    pub enabled: bool,
    pub output: String,
}

impl Prompt {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.len() > 120 {
            return Err("Give each prompt a name (up to 120 characters).".into());
        }
        if self.instructions.trim().is_empty() || self.instructions.len() > 32_000 {
            return Err(format!(
                "{}: add instructions (up to 32,000 bytes).",
                self.name
            ));
        }
        if !["replace", "review"].contains(&self.output.as_str()) {
            return Err("Choose Replace selection or Review result.".into());
        }
        if !self.shortcut.is_empty() {
            let shortcut = self
                .shortcut
                .parse::<Shortcut>()
                .map_err(|_| format!("{}: invalid shortcut.", self.name))?;
            if shortcut.mods.is_empty() {
                return Err(
                    "Shortcuts must include a modifier such as Ctrl, Alt or Command.".into(),
                );
            }
        }
        if !self.model.is_empty() {
            validate_model(&self.model)?;
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub default_model: String,
    pub launch_at_login: bool,
    pub prompts: Vec<Prompt>,
}

fn validate_model(value: &str) -> Result<(), String> {
    if value.len() > 200 || !value.contains('/') || value.chars().any(char::is_whitespace) {
        return Err(
            "Enter an OpenRouter model ID, for example google/gemini-2.5-flash-lite.".into(),
        );
    }
    Ok(())
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("This settings file uses an unsupported version.".into());
        }
        validate_model(&self.default_model)?;
        if self.prompts.len() > 100 {
            return Err("Use up to 100 prompts.".into());
        }
        let mut ids = HashSet::new();
        let mut shortcuts = HashSet::new();
        for prompt in &self.prompts {
            prompt.validate()?;
            if prompt.id.is_empty() || !ids.insert(&prompt.id) {
                return Err("Prompt IDs must be unique and nonempty.".into());
            }
            if prompt.enabled && !prompt.shortcut.is_empty() {
                let shortcut = prompt
                    .shortcut
                    .parse::<Shortcut>()
                    .map_err(|_| "Invalid shortcut")?;
                if !shortcuts.insert(shortcut.id()) {
                    return Err(format!(
                        "{}: another prompt already uses that shortcut.",
                        prompt.name
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let prompts = [
            ("correct", "Fix Writing", "W", "Correct spelling, punctuation and grammar. Preserve the original language, tone, meaning and paragraph structure. Make minimal changes. Return only the corrected text."),
            ("translate", "Translate to English", "T", "Translate the text into natural US English. Preserve meaning, tone and paragraph structure. Return only the translation."),
            ("shorten", "Make It Shorter", "K", "Make the text more concise without losing essential information. Preserve its language, meaning and tone. Return only the revised text."),
        ].into_iter().map(|(id, name, key, instructions)| Prompt { id: id.into(), name: name.into(), shortcut: format!("Ctrl+Alt+Shift+Super+{key}"), instructions: instructions.into(), model: String::new(), enabled: true, output: "replace".into() }).collect();
        Self {
            version: 1,
            default_model: DEFAULT_MODEL.into(),
            launch_at_login: false,
            prompts,
        }
    }
}

fn location(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("config.json"))
        .map_err(|e| e.to_string())
}

pub fn load(app: &tauri::AppHandle) -> (Config, Option<String>) {
    let read = || -> Result<Config, String> {
        let file = location(app)?;
        if !file.exists() {
            return Ok(Config::default());
        }
        let metadata = fs::metadata(&file).map_err(|e| e.to_string())?;
        if metadata.len() > 4_000_000 {
            return Err("Settings file is too large.".into());
        }
        let config: Config = serde_json::from_slice(&fs::read(&file).map_err(|e| e.to_string())?)
            .map_err(|_| {
            "Could not read settings. Your file has been left untouched.".to_string()
        })?;
        config.validate()?;
        Ok(config)
    };
    match read() {
        Ok(config) => (config, None),
        Err(e) => (Config::default(), Some(e)),
    }
}

pub fn save(app: &tauri::AppHandle, config: &Config) -> Result<(), String> {
    let path = location(app)?;
    let directory = path.parent().ok_or("Invalid settings directory")?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let temporary = directory.join(format!("config-{}.tmp", uuid::Uuid::new_v4()));
    let write = || -> Result<(), String> {
        fs::write(
            &temporary,
            serde_json::to_vec_pretty(config).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        #[cfg(windows)]
        {
            // std::fs::rename replaces an existing file on supported Windows targets.
        }
        fs::rename(&temporary, &path).map_err(|e| format!("Could not save settings: {e}"))?;
        Ok(())
    };
    let result = write();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aliases_cannot_assign_same_shortcut_twice() {
        let mut config = Config::default();
        config.prompts[1].shortcut = "Control+Alt+Shift+Super+KeyW".into();
        assert!(config.validate().is_err());
    }
    #[test]
    fn disabled_prompts_do_not_reserve_shortcuts() {
        let mut config = Config::default();
        config.prompts[1].shortcut = config.prompts[0].shortcut.clone();
        config.prompts[1].enabled = false;
        assert!(config.validate().is_ok());
    }
    #[test]
    fn defaults_round_trip_without_any_secret() {
        let config = Config::default();
        config.validate().unwrap();
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("apiKey"));
        serde_json::from_str::<Config>(&json)
            .unwrap()
            .validate()
            .unwrap();
    }
    #[test]
    fn reject_plain_keys_invalid_models_and_unknown_output() {
        let mut config = Config::default();
        config.prompts[0].shortcut = "E".into();
        assert!(config.validate().is_err());
        config.prompts[0].shortcut.clear();
        config.default_model = "invalid model".into();
        assert!(config.validate().is_err());
        config.default_model = DEFAULT_MODEL.into();
        config.prompts[0].output = "surprise".into();
        assert!(config.validate().is_err());
    }
}
