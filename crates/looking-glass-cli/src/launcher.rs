//! Launch logic with mod injection for lg-launch

use anyhow::{Context, Result};
use looking_glass_mod::{ModDiscovery, InstalledMod};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;

use crate::profiles::{ModProfile, LauncherConfig};

/// Launch options
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub game_path: PathBuf,
    pub mod_directory: PathBuf,
    pub reshade_path: Option<PathBuf>,
    pub profile: ModProfile,
    pub mods: Vec<InstalledMod>,
    pub extra_args: Vec<String>,
    pub extra_env: HashMap<String, String>,
    pub no_mods: bool,
    pub use_reshade: bool,
}

/// Game launcher with mod injection support
pub struct GameLauncher {
    config: LauncherConfig,
}

impl GameLauncher {
    pub fn new(config: LauncherConfig) -> Self {
        Self { config }
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
        let game_path = self.config.game_path.clone()
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
            .context("Game binary not found. Build with 'cargo build --release' or set game_path in config")?;

        // Determine mod directory
        let mod_directory = self.config.mod_directory.clone()
            .unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("looking-glass")
                    .join("mods")
            });

        // Determine ReShade path
        let reshade_path = reshade_path_override
            .or(self.config.reshade_path.clone())
            .or_else(|| {
                // Check common ReShade locations
                let candidates = [
                    PathBuf::from("reshade/ReShade64.dll"),
                    PathBuf::from("/usr/lib/reshade/ReShade64.dll"),
                    PathBuf::from("/usr/local/lib/reshade/ReShade64.dll"),
                ];
                candidates.into_iter().find(|p| p.exists())
            });

        // Discover mods
        let discovery = ModDiscovery::new(mod_directory.clone(), "Alice".to_string());
        let all_mods = if no_mods {
            Vec::new()
        } else {
            discovery.discover().context("Failed to discover mods")?
        };

        // Get profile
        let profile = if profile_name == "vanilla" || no_mods {
            ModProfile::vanilla()
        } else if let Some(p) = self.config.profiles.get(profile_name) {
            p.clone()
        } else {
            // Try to find by case-insensitive match
            self.config.profiles.iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(profile_name))
                .map(|(_, v)| v.clone())
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

        Ok(LaunchOptions {
            game_path,
            mod_directory,
            reshade_path,
            profile,
            mods: selected_mods,
            extra_args,
            extra_env,
            no_mods,
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
    pub fn install_mod(&self, archive_path: &Path, game_id: &str) -> Result<InstalledMod> {
        let mod_directory = self.config.mod_directory.clone()
            .unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("looking-glass")
                    .join("mods")
            });

        let discovery = ModDiscovery::new(mod_directory, game_id.to_string());
        let installed_mod = discovery.install_from_archive(archive_path)
            .context("Failed to install mod")?;

        tracing::info!("Installed mod: {} v{}", installed_mod.manifest.id, installed_mod.manifest.version);
        Ok(installed_mod)
    }

    /// List available profiles
    pub fn list_profiles(&self) -> Vec<&ModProfile> {
        self.config.profiles.values().collect()
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