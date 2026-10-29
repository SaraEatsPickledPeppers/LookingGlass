//! looking-glass-platform - Platform integrations for Looking Glass launcher

pub mod steam;
pub mod heroic;
pub mod flatpak;
pub mod wine;
pub mod lutris;

pub use steam::SteamIntegration;
pub use heroic::HeroicIntegration;
pub use flatpak::FlatpakIntegration;
pub use wine::WineIntegration;
pub use lutris::LutrisIntegration;

mod steam {
    use super::*;
    use std::path::PathBuf;
    use looking_glass_core::{GameEntry, Platform, GameMetadata};

    pub struct SteamIntegration {
        steam_path: Option<PathBuf>,
    }

    impl SteamIntegration {
        pub fn new(steam_path: Option<PathBuf>) -> Self {
            Self { steam_path }
        }

        pub fn discover_games(&self) -> anyhow::Result<Vec<GameEntry>> {
            // TODO: Parse libraryfolders.vdf, appmanifest_*.acf
            Ok(vec![])
        }

        pub fn get_proton_versions(&self) -> anyhow::Result<Vec<String>> {
            // TODO: List Proton versions in compatibilitytools.d
            Ok(vec![])
        }

        pub fn launch_game(&self, app_id: u32, proton_version: Option<&str>) -> anyhow::Result<std::process::Child> {
            // TODO: Launch via steam://rungameid or steam -applaunch
            todo!()
        }
    }
}

mod heroic {
    use super::*;
    use std::path::PathBuf;
    use looking_glass_core::{GameEntry, Platform, GameMetadata};

    pub struct HeroicIntegration {
        heroic_path: Option<PathBuf>,
    }

    impl HeroicIntegration {
        pub fn new(heroic_path: Option<PathBuf>) -> Self {
            Self { heroic_path }
        }

        pub fn discover_games(&self) -> anyhow::Result<Vec<GameEntry>> {
            // TODO: Parse Heroic config
            Ok(vec![])
        }

        pub fn get_wine_prefixes(&self) -> anyhow::Result<Vec<PathBuf>> {
            // TODO: List Wine prefixes
            Ok(vec![])
        }

        pub fn launch_game(&self, game_id: &str) -> anyhow::Result<std::process::Child> {
            // TODO: Launch via Heroic CLI or Wine
            todo!()
        }
    }
}

mod flatpak {
    use super::*;
    use looking_glass_core::{GameEntry, Platform, GameMetadata};

    pub struct FlatpakIntegration;

    impl FlatpakIntegration {
        pub fn new() -> Self {
            Self
        }

        pub fn discover_games(&self) -> anyhow::Result<Vec<GameEntry>> {
            // TODO: Run `flatpak list --app` and parse
            Ok(vec![])
        }

        pub fn get_permissions(&self, app_id: &str) -> anyhow::Result<Vec<String>> {
            // TODO: Run `flatpak info --show-permissions`
            Ok(vec![])
        }

        pub fn launch_game(&self, app_id: &str, args: &[String]) -> anyhow::Result<std::process::Child> {
            // TODO: Launch via flatpak run
            todo!()
        }
    }
}

mod wine {
    use super::*;
    use std::path::PathBuf;
    use looking_glass_core::{GameEntry, Platform, GameMetadata};

    pub struct WineIntegration {
        wine_prefix: Option<PathBuf>,
        proton_version: Option<String>,
    }

    impl WineIntegration {
        pub fn new(wine_prefix: Option<PathBuf>, proton_version: Option<String>) -> Self {
            Self { wine_prefix, proton_version }
        }

        pub fn setup_prefix(&self, prefix: &PathBuf) -> anyhow::Result<()> {
            // TODO: Initialize Wine prefix
            Ok(())
        }

        pub fn install_proton(&self, version: &str) -> anyhow::Result<PathBuf> {
            // TODO: Download and install Proton-GE
            todo!()
        }

        pub fn launch_game(&self, exe_path: &PathBuf, args: &[String]) -> anyhow::Result<std::process::Child> {
            // TODO: Launch with Wine/Proton
            todo!()
        }
    }
}

mod lutris {
    use super::*;
    use looking_glass_core::{GameEntry, Platform, GameMetadata};

    pub struct LutrisIntegration;

    impl LutrisIntegration {
        pub fn new() -> Self {
            Self
        }

        pub fn discover_games(&self) -> anyhow::Result<Vec<GameEntry>> {
            // TODO: Parse Lutris database
            Ok(vec![])
        }

        pub fn launch_game(&self, game_slug: &str) -> anyhow::Result<std::process::Child> {
            // TODO: Launch via lutris cli
            todo!()
        }
    }
}