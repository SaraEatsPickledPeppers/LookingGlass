//! looking-glass-core - Core launcher functionality: game registry, launch abstraction, config

pub use game_registry::{GameEntry, GameRegistry, Platform, GameMetadata};
pub use launch::{LaunchContext, PlatformLauncher};
pub use config::LauncherConfig;

mod game_registry {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
    pub enum Platform {
        Steam,
        Heroic,
        Flatpak,
        Lutris,
        Wine,
        Proton,
        Native,
        AppImage,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct GameMetadata {
        pub name: String,
        pub description: Option<String>,
        pub install_path: PathBuf,
        pub executable: Option<String>,
        pub launch_args: Vec<String>,
        pub environment: HashMap<String, String>,
        pub prefix: Option<PathBuf>,
        pub tags: Vec<String>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct GameEntry {
        pub id: String,
        pub platform: Platform,
        pub metadata: GameMetadata,
        pub enabled: bool,
    }

    #[derive(Debug, Default)]
    pub struct GameRegistry {
        games: HashMap<String, GameEntry>,
    }

    impl GameRegistry {
        pub fn new() -> Self {
            Self { games: HashMap::new() }
        }

        pub fn register(&mut self, game: GameEntry) {
            self.games.insert(game.id.clone(), game);
        }

        pub fn get(&self, id: &str) -> Option<&GameEntry> {
            self.games.get(id)
        }

        pub fn list(&self) -> Vec<&GameEntry> {
            self.games.values().collect()
        }

        pub fn scan_steam(&mut self) -> anyhow::Result<()> {
            // TODO: Implement Steam library scanning
            Ok(())
        }

        pub fn scan_flatpak(&mut self) -> anyhow::Result<()> {
            // TODO: Implement Flatpak scanning
            Ok(())
        }

        pub fn scan_heroic(&mut self) -> anyhow::Result<()> {
            // TODO: Implement Heroic scanning
            Ok(())
        }
    }
}
mod launch {
    use super::*;
    use std::process::Command;
    use std::path::PathBuf;

    #[derive(Debug, Clone)]
    pub struct LaunchContext {
        pub game_id: String,
        pub executable: PathBuf,
        pub args: Vec<String>,
        pub env: std::collections::HashMap<String, String>,
        pub working_dir: Option<PathBuf>,
    }

    pub trait PlatformLauncher: Send + Sync {
        fn build_command(&self, ctx: &LaunchContext) -> Command;
        fn platform_name(&self) -> &'static str;
    }

    pub struct SteamLauncher;
    impl PlatformLauncher for SteamLauncher {
        fn build_command(&self, ctx: &LaunchContext) -> Command {
            let mut cmd = Command::new("steam");
            cmd.arg(format!("steam://rungameid/{}", ctx.game_id));
            cmd
        }
        fn platform_name(&self) -> &'static str { "Steam" }
    }

    pub struct FlatpakLauncher;
    impl PlatformLauncher for FlatpakLauncher {
        fn build_command(&self, ctx: &LaunchContext) -> Command {
            let mut cmd = Command::new("flatpak");
            cmd.arg("run").arg(&ctx.game_id);
            cmd.args(&ctx.args);
            cmd
        }
        fn platform_name(&self) -> &'static str { "Flatpak" }
    }

    pub struct NativeLauncher;
    impl PlatformLauncher for NativeLauncher {
        fn build_command(&self, ctx: &LaunchContext) -> Command {
            let mut cmd = Command::new(&ctx.executable);
            cmd.args(&ctx.args);
            if let Some(dir) = &ctx.working_dir {
                cmd.current_dir(dir);
            }
            for (k, v) in &ctx.env {
                cmd.env(k, v);
            }
            cmd
        }
        fn platform_name(&self) -> &'static str { "Native" }
    }

    pub struct WineLauncher;
    impl PlatformLauncher for WineLauncher {
        fn build_command(&self, ctx: &LaunchContext) -> Command {
            let mut cmd = Command::new("wine");
            if let Some(prefix) = &ctx.working_dir {
                cmd.env("WINEPREFIX", prefix);
            }
            cmd.arg(&ctx.executable);
            cmd.args(&ctx.args);
            cmd
        }
        fn platform_name(&self) -> &'static str { "Wine" }
    }
}
mod config {
    use super::*;
    use looking_glass_config::{ConfigManager, ConfigLayer};

    pub struct LauncherConfig {
        manager: ConfigManager,
    }

    impl LauncherConfig {
        pub fn new() -> anyhow::Result<Self> {
            Ok(Self { manager: ConfigManager::new()? })
        }

        pub fn get_game_path(&self, game_id: &str) -> Option<std::path::PathBuf> {
            self.manager.get(&format!("games.{}.metadata.install_path", game_id))
                .and_then(|v| v.as_str().map(std::path::PathBuf::from))
        }
    }
}