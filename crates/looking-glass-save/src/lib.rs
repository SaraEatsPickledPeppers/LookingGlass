//! looking-glass-save - Save management for Looking Glass launcher

pub mod backup;
pub mod discovery;
pub mod sync;

pub use backup::{SaveBackup, BackupManager};
pub use discovery::{SaveDiscovery, SaveLocation};
pub use sync::{SyncProvider, SyncManager};

mod backup {
    use super::*;
    use std::path::PathBuf;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct SaveBackup {
        pub id: String,
        pub game_id: String,
        pub path: PathBuf,
        pub created_at: DateTime<Utc>,
        pub size: u64,
        pub description: Option<String>,
    }

    pub struct BackupManager {
        backup_dir: PathBuf,
    }

    impl BackupManager {
        pub fn new(backup_dir: PathBuf) -> Self {
            Self { backup_dir }
        }

        pub fn create_backup(&self, game_id: &str, save_path: &PathBuf) -> anyhow::Result<SaveBackup> {
            // TODO: Implement backup creation
            todo!()
        }

        pub fn list_backups(&self, game_id: &str) -> anyhow::Result<Vec<SaveBackup>> {
            // TODO: Implement backup listing
            Ok(vec![])
        }

        pub fn restore_backup(&self, backup: &SaveBackup, target: &PathBuf) -> anyhow::Result<()> {
            // TODO: Implement backup restoration
            todo!()
        }

        pub fn delete_backup(&self, backup_id: &str) -> anyhow::Result<()> {
            // TODO: Implement backup deletion
            todo!()
        }
    }
}

mod discovery {
    use super::*;
    use std::path::PathBuf;
    use looking_glass_config::PlatformConfig;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct SaveLocation {
        pub game_id: String,
        pub platform: String,
        pub path: PathBuf,
        pub is_cloud: bool,
    }

    pub struct SaveDiscovery;

    impl SaveDiscovery {
        pub fn discover_for_game(game_id: &str, platform_config: &PlatformConfig) -> anyhow::Result<Vec<SaveLocation>> {
            // TODO: Implement save discovery per platform
            Ok(vec![])
        }

        pub fn discover_all(platform_config: &PlatformConfig) -> anyhow::Result<Vec<SaveLocation>> {
            // TODO: Implement discovery for all games
            Ok(vec![])
        }
    }
}

mod sync {
    use super::*;
    use async_trait::async_trait;

    #[async_trait]
    pub trait SyncProvider: Send + Sync {
        async fn push(&self, game_id: &str, backup_path: &std::path::Path) -> anyhow::Result<()>;
        async fn pull(&self, game_id: &str, target_dir: &std::path::Path) -> anyhow::Result<()>;
        async fn list_remote(&self, game_id: &str) -> anyhow::Result<Vec<String>>;
        fn name(&self) -> &'static str;
    }

    pub struct LocalSyncProvider {
        base_dir: std::path::PathBuf,
    }

    impl LocalSyncProvider {
        pub fn new(base_dir: std::path::PathBuf) -> Self {
            Self { base_dir }
        }
    }

    #[async_trait]
    impl SyncProvider for LocalSyncProvider {
        async fn push(&self, _game_id: &str, _backup_path: &std::path::Path) -> anyhow::Result<()> {
            // TODO: Implement local push
            Ok(())
        }

        async fn pull(&self, _game_id: &str, _target_dir: &std::path::Path) -> anyhow::Result<()> {
            // TODO: Implement local pull
            Ok(())
        }

        async fn list_remote(&self, _game_id: &str) -> anyhow::Result<Vec<String>> {
            Ok(vec![])
        }

        fn name(&self) -> &'static str {
            "local"
        }
    }

    pub struct SyncManager {
        providers: std::collections::HashMap<String, Box<dyn SyncProvider>>,
    }

    impl SyncManager {
        pub fn new() -> Self {
            Self { providers: std::collections::HashMap::new() }
        }

        pub fn register(&mut self, provider: Box<dyn SyncProvider>) {
            self.providers.insert(provider.name().to_string(), provider);
        }

        pub fn get(&self, name: &str) -> Option<&dyn SyncProvider> {
            self.providers.get(name).map(|p| p.as_ref())
        }
    }
}