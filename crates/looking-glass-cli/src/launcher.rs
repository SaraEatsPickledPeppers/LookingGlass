//! Launch logic with mod injection for lg-launch

use anyhow::{Context, Result};
use looking_glass_mod::{ModDiscovery, InstalledMod};
use looking_glass_config::{Config, paths};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;

use crate::profiles::{ModProfile, ProfileManager};

/// Launch options
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub game_path: PathBuf,
    pub mod_directory: PathBuf,
    pub reshade_path: Option<PathBuf>,
    pub data_directory: Option<PathBuf>,
    pub profile: ModProfile,
    pub mods: Vec<InstalledMod>,
    pub extra_args: Vec<String>,
    pub extra_env: HashMap<String, String>,
    pub no_mods: bool,
    pub use_reshade: bool,
}

/// Game launcher with mod injection support
pub struct GameLauncher {
    config: Config,
    config_path_override: Option<PathBuf>,
    profile_manager: ProfileManager,
}

impl GameLauncher {
    pub fn new(config: Config, config_path_override: Option<PathBuf>) -> Self {
        let config_dir = config_path_override.as_ref().map(|p| p.clone()).unwrap_or_else(|| paths::config_dir());
        let profile_manager = ProfileManager::new(config_dir).unwrap_or_else(|_| ProfileManager::new(paths::config_dir()).unwrap());
        Self { config, config_path_override, profile_manager }
    }

    /// Build launch options from profile and CLI args
    pub fn build_options(
        &self,
        profile_name: &str,
        extra_args: Vec<String>,
        extra_env: HashMap<String, String>,
        no_mods: bool,
        use_reshade: bool,
        reshade_path_override: Option<PathBuf>,
    ) -> Result<LaunchOptions> {
        // Determine game path
        let game_path = self.config.game.data_path.clone()
            .or_else(|| {
                // Try to find the looking-glass binary in target/release
                let candidates = [
                    PathBuf::from("target/release/looking-glass"),
                    PathBuf::from("../target/release/looking-glass"),
                    PathBuf::from("../../target/release/looking-glass"),
                    PathBuf::from("/usr/local/bin/looking-glass"),
                    PathBuf::from("/usr/bin/looking-glass"),
                ];
                // Convert to absolute paths and find the first that exists
                let cwd = std::env::current_dir().ok()?;
                candidates.into_iter()
                    .map(|p| {
                        if p.is_absolute() {
                            p
                        } else {
                            cwd.join(p)
                        }
                    })
                    .find(|p| p.exists())
            })
            .context("Game binary not found. Build with 'cargo build --release' or set game.data_path in config")?;

        // Determine mod directory (use first mod dir from config, or default)
        let mod_directory = self.config.mods.mod_dirs.first().cloned()
            .unwrap_or_else(|| {
                paths::data_dir().join("mods")
            });

        // Determine ReShade path
        let reshade_path = reshade_path_override
            .or_else(|| {
                // Check common ReShade locations
                let candidates = [
                    PathBuf::from("reshade/ReShade64.dll"),
                    PathBuf::from("/usr/lib/reshade/ReShade64.dll"),
                    PathBuf::from("/usr/local/lib/reshade/ReShade64.dll"),
                ];
                candidates.into_iter().find(|p| p.exists())
            });

        // Determine data directory
        let data_directory = self.config.game.data_path.clone();

        // Discover mods
        // ModDiscovery expects the base directory (parent of "mods"), not the game-specific mods dir
        let base_dir = mod_directory.parent().unwrap_or(&mod_directory).to_path_buf();
        let discovery = ModDiscovery::new(base_dir, "Alice".to_string());
        let all_mods = if no_mods {
            Vec::new()
        } else {
            discovery.discover().context("Failed to discover mods")?
        };

        // Get profile from profile manager
        let profile = if profile_name == "vanilla" || no_mods {
            ModProfile::vanilla()
        } else if let Some(p) = self.profile_manager.get_profile(profile_name) {
            p.clone()
        } else {
            // Try to find by case-insensitive match
            self.profile_manager.list_profiles()
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case(profile_name))
                .map(|p| (*p).clone())
                .unwrap_or_else(|| ModProfile::new(profile_name.to_string(), vec![]))
        };

        // Filter mods based on profile
        let selected_mods = if no_mods || profile_name == "vanilla" {
            Vec::new()
        } else {
            all_mods.into_iter()
                .filter(|m| profile.mod_ids.contains(&m.manifest.id) && m.enabled)
                .collect()
        };

        // no_mods should be true if explicitly requested OR if profile is vanilla
        let effective_no_mods = no_mods || profile_name == "vanilla";

        Ok(LaunchOptions {
            game_path,
            mod_directory,
            reshade_path,
            data_directory,
            profile,
            mods: selected_mods,
            extra_args,
            extra_env,
            no_mods: effective_no_mods,
            use_reshade,
        })
    }

    /// Launch the game with mod injection
    pub fn launch(&self, options: LaunchOptions) -> Result<()> {
        let mut cmd = Command::new(&options.game_path);

        // Set working directory to game directory
        if let Some(parent) = options.game_path.parent() {
            cmd.current_dir(parent);
        }

        // Add extra arguments
        cmd.args(&options.extra_args);

        // Pass --no-mods flag to game if no mods should be loaded
        if options.no_mods {
            cmd.arg("--no-mods");
        }

        // Pass --data argument to game
        if let Some(data_dir) = &options.data_directory {
            cmd.arg("--data").arg(data_dir);
        }

        // Set up environment
        let mut env = options.extra_env;

        // Add mod injection environment variables
        if !options.no_mods && !options.mods.is_empty() {
            // Build mod paths for LD_PRELOAD or game's mod API
            let mod_paths: Vec<String> = options.mods.iter()
                .map(|m| m.path.to_string_lossy().to_string())
                .collect();

            // Set mod paths - the game/mod system will use this
            env.insert("LOOKING_GLASS_MODS".to_string(), mod_paths.join(":"));

            // Also set individual mod paths for the mod loader
            for (i, m) in options.mods.iter().enumerate() {
                env.insert(format!("LOOKING_GLASS_MOD_{}", i), m.path.to_string_lossy().to_string());
            }

            // Set mod load order
            let load_order: Vec<String> = options.mods.iter()
                .map(|m| m.manifest.id.clone())
                .collect();
            env.insert("LOOKING_GLASS_MOD_ORDER".to_string(), load_order.join(","));

            tracing::info!("Injecting {} mods: {}", options.mods.len(), load_order.join(", "));
        }

        // ReShade injection
        if options.use_reshade {
            if let Some(reshade_path) = &options.reshade_path {
                if reshade_path.exists() {
                    // For Vulkan (VK_LAYER)
                    env.insert("VK_LAYER_PATH".to_string(),
                        reshade_path.parent()
                            .unwrap_or(Path::new("."))
                            .to_string_lossy().to_string());

                    // For DXGI (dxgi.dll injection on Wine)
                    env.insert("RESHADE_DLL_PATH".to_string(), reshade_path.to_string_lossy().to_string());

                    // For DXVK
                    env.insert("DXVK_HUD".to_string(), "1".to_string());

                    tracing::info!("ReShade enabled: {}", reshade_path.display());
                } else {
                    tracing::warn!("ReShade DLL not found at: {}", reshade_path.display());
                }
            } else {
                tracing::warn!("ReShade requested but no ReShade path configured");
            }
        }

        // Apply environment variables
        for (k, v) in env {
            cmd.env(k, v);
        }

        tracing::info!("Launching game: {}", options.game_path.display());
        tracing::debug!("Working directory: {:?}", options.game_path.parent());
        tracing::debug!("Args: {:?}", options.extra_args);

        // Execute the game
        let mut child = cmd.spawn()
            .context("Failed to spawn game process")?;

        let status = child.wait()
            .context("Failed to wait for game process")?;

        if !status.success() {
            tracing::error!("Game exited with status: {}", status);
            return Err(anyhow::anyhow!("Game exited with status: {}", status));
        }

        tracing::info!("Game exited successfully");
        Ok(())
    }

    /// Install a mod from an archive file
    pub fn install_mod(&self, archive_path: &Path) -> Result<InstalledMod> {
        let mod_directory = self.config.mods.mod_dirs.first().cloned()
            .unwrap_or_else(|| {
                paths::data_dir().join("mods")
            });

        // ModDiscovery expects the base directory (parent of "mods"), not the game-specific mods dir
        let base_dir = mod_directory.parent().unwrap_or(&mod_directory).to_path_buf();
        let discovery = ModDiscovery::new(base_dir, "Alice".to_string());
        let installed_mod = discovery.install_from_archive(archive_path)
            .context("Failed to install mod")?;

        tracing::info!("Installed mod: {} v{}", installed_mod.manifest.id, installed_mod.manifest.version);
        Ok(installed_mod)
    }

    /// List available profiles
    pub fn list_profiles(&self) -> Vec<&ModProfile> {
        self.profile_manager.list_profiles()
    }
}

/// Helper to find the game binary
pub fn find_game_binary() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("target/release/looking-glass"),
        PathBuf::from("../target/release/looking-glass"),
        PathBuf::from("../../target/release/looking-glass"),
        PathBuf::from("/usr/local/bin/looking-glass"),
        PathBuf::from("/usr/bin/looking-glass"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

/// Helper to find ReShade DLL
pub fn find_reshade_dll() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("reshade/ReShade64.dll"),
        PathBuf::from("/usr/lib/reshade/ReShade64.dll"),
        PathBuf::from("/usr/local/lib/reshade/ReShade64.dll"),
        PathBuf::from("/opt/reshade/ReShade64.dll"),
    ];
    candidates.into_iter().find(|p| p.exists())
}