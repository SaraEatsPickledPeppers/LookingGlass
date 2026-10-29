//! Mod profile management for lg-launch

use anyhow::{Context, Result};
use looking_glass_mod::{ModDiscovery, InstalledMod};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Predefined mod profiles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Profile {
    /// Vanilla - no mods
    Vanilla,
    /// Cheats mod profile
    Cheats,
    /// Button glyphs mod profile
    ButtonGlyphs,
    /// Custom profile with user-defined mod list
    Custom(String),
}

impl Profile {
    pub fn name(&self) -> String {
        match self {
            Profile::Vanilla => "vanilla".to_string(),
            Profile::Cheats => "cheats".to_string(),
            Profile::ButtonGlyphs => "button-glyphs".to_string(),
            Profile::Custom(name) => name.clone(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "vanilla" => Profile::Vanilla,
            "cheats" => Profile::Cheats,
            "button-glyphs" | "buttonglyphs" => Profile::ButtonGlyphs,
            name => Profile::Custom(name.to_string()),
        }
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, Profile::Vanilla | Profile::Cheats | Profile::ButtonGlyphs)
    }
}

/// Mod profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModProfile {
    pub name: String,
    pub description: Option<String>,
    pub mod_ids: Vec<String>,
    pub enabled: bool,
}

impl ModProfile {
    pub fn new(name: String, mod_ids: Vec<String>) -> Self {
        Self {
            name,
            description: None,
            mod_ids,
            enabled: true,
        }
    }

    pub fn vanilla() -> Self {
        Self::new("vanilla".to_string(), vec![])
    }

    pub fn cheats() -> Self {
        Self::new("cheats".to_string(), vec!["cheats".to_string()])
    }

    pub fn button_glyphs() -> Self {
        Self::new("button-glyphs".to_string(), vec!["button-glyphs".to_string()])
    }
}

/// Profile manager for handling mod profiles
pub struct ProfileManager {
    config_dir: PathBuf,
    profiles: HashMap<String, ModProfile>,
    default_profile: String,
}

impl ProfileManager {
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            config_dir: config_dir.clone(),
            profiles: HashMap::new(),
            default_profile: "default".to_string(),
        };
        manager.load()?;
        manager.ensure_builtin_profiles();
        Ok(manager)
    }

    fn profiles_file(&self) -> PathBuf {
        self.config_dir.join("profiles.toml")
    }

    fn load(&mut self) -> Result<()> {
        let path = self.profiles_file();
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .context("Failed to read profiles file")?;
            let data: HashMap<String, ModProfile> = toml::from_str(&content)
                .context("Failed to parse profiles file")?;
            self.profiles = data;
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let path = self.profiles_file();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        let content = toml::to_string_pretty(&self.profiles)
            .context("Failed to serialize profiles")?;
        std::fs::write(&path, content)
            .context("Failed to write profiles file")?;
        Ok(())
    }

    fn ensure_builtin_profiles(&mut self) {
        // Ensure vanilla profile exists
        if !self.profiles.contains_key("vanilla") {
            self.profiles.insert("vanilla".to_string(), ModProfile::vanilla());
        }
        // Ensure cheats profile exists
        if !self.profiles.contains_key("cheats") {
            self.profiles.insert("cheats".to_string(), ModProfile::cheats());
        }
        // Ensure button-glyphs profile exists
        if !self.profiles.contains_key("button-glyphs") {
            self.profiles.insert("button-glyphs".to_string(), ModProfile::button_glyphs());
        }
    }

    pub fn list_profiles(&self) -> Vec<&ModProfile> {
        self.profiles.values().collect()
    }

    pub fn get_profile(&self, name: &str) -> Option<&ModProfile> {
        self.profiles.get(name)
    }

    pub fn get_profile_mut(&mut self, name: &str) -> Option<&mut ModProfile> {
        self.profiles.get_mut(name)
    }

    pub fn create_profile(&mut self, name: String, mod_ids: Vec<String>) -> Result<()> {
        if self.profiles.contains_key(&name) {
            return Err(anyhow::anyhow!("Profile '{}' already exists", name));
        }
        let profile = ModProfile::new(name.clone(), mod_ids);
        self.profiles.insert(name, profile);
        self.save()?;
        Ok(())
    }

    pub fn delete_profile(&mut self, name: &str) -> Result<()> {
        if name == "vanilla" || name == "cheats" || name == "button-glyphs" {
            return Err(anyhow::anyhow!("Cannot delete builtin profile '{}'", name));
        }
        self.profiles.remove(name);
        self.save()?;
        Ok(())
    }

    pub fn set_default_profile(&mut self, name: &str) -> Result<()> {
        if !self.profiles.contains_key(name) {
            return Err(anyhow::anyhow!("Profile '{}' does not exist", name));
        }
        self.default_profile = name.to_string();
        // Save to config (in a real implementation, this would be in the main config)
        Ok(())
    }

    pub fn get_default_profile(&self) -> &str {
        &self.default_profile
    }

    pub fn resolve_mods_for_profile(&self, profile_name: &str, discovery: &ModDiscovery) -> Result<Vec<InstalledMod>> {
        let profile = self.profiles.get(profile_name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", profile_name))?;

        let installed_mods = discovery.discover()?;
        let mut selected_mods = Vec::new();

        for mod_id in &profile.mod_ids {
            if let Some(mod_info) = installed_mods.iter().find(|m| m.manifest.id == *mod_id) {
                if mod_info.enabled {
                    selected_mods.push(mod_info.clone());
                }
            } else {
                tracing::warn!("Mod '{}' in profile '{}' not found or not enabled", mod_id, profile_name);
            }
        }

        Ok(selected_mods)
    }
}

/// Configuration for the launcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    pub game_path: Option<PathBuf>,
    pub mod_directory: Option<PathBuf>,
    pub reshade_path: Option<PathBuf>,
    pub data_directory: Option<PathBuf>,
    pub default_profile: String,
    pub profiles: HashMap<String, ModProfile>,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("vanilla".to_string(), ModProfile::vanilla());
        profiles.insert("cheats".to_string(), ModProfile::cheats());
        profiles.insert("button-glyphs".to_string(), ModProfile::button_glyphs());

        Self {
            game_path: None,
            mod_directory: None,
            reshade_path: None,
            data_directory: None,
            default_profile: "vanilla".to_string(),
            profiles,
        }
    }
}

impl LauncherConfig {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let path = config_dir.join("launcher.toml");
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .context("Failed to read launcher config")?;
            let mut config: LauncherConfig = toml::from_str(&content)
                .context("Failed to parse launcher config")?;
            // Ensure built-in profiles exist
            config.ensure_builtin_profiles();
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    fn ensure_builtin_profiles(&mut self) {
        if !self.profiles.contains_key("vanilla") {
            self.profiles.insert("vanilla".to_string(), ModProfile::vanilla());
        }
        if !self.profiles.contains_key("cheats") {
            self.profiles.insert("cheats".to_string(), ModProfile::cheats());
        }
        if !self.profiles.contains_key("button-glyphs") {
            self.profiles.insert("button-glyphs".to_string(), ModProfile::button_glyphs());
        }
    }

    pub fn save(&self, config_dir: &Path) -> Result<()> {
        let path = config_dir.join("launcher.toml");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize launcher config")?;
        std::fs::write(&path, content)
            .context("Failed to write launcher config")?;
        Ok(())
    }
}