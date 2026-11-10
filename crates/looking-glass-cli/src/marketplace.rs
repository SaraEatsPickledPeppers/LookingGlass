//! Mod marketplace integration for Mod.io and NexusMods

mod modio;
mod nexusmods;
mod cache;

pub use modio::{ModioClient, ModioMod, ModioModfile, ModioSearchParams};
pub use nexusmods::{NexusModsClient, NexusMod, NexusModFile, NexusSearchParams};
pub use cache::MarketplaceCache;

use anyhow::{Context, Result};
use looking_glass_config::ConfigManager;
use looking_glass_mod::discovery::ModDiscovery;
use looking_glass_mod::manifest::{InstalledMod, ModManifest};
use reqwest::blocking::Client;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Unified mod representation from any marketplace
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteMod {
    pub source: MarketplaceSource,
    pub id: String,
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub downloads: u64,
    pub rating: Option<f32>,
    pub tags: Vec<String>,
    pub logo_url: Option<String>,
    pub images: Vec<String>,
    pub homepage_url: Option<String>,
    pub download_url: Option<String>,
    pub file_size: Option<u64>,
    pub file_hash: Option<String>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub dependencies: Vec<String>,
    pub game_versions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MarketplaceSource {
    Modio,
    NexusMods,
}

impl std::fmt::Display for MarketplaceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketplaceSource::Modio => write!(f, "Mod.io"),
            MarketplaceSource::NexusMods => write!(f, "NexusMods"),
        }
    }
}

/// Search parameters for marketplace queries
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub query: Option<String>,
    pub tags: Vec<String>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    #[default]
    Relevance,
    Name,
    Updated,
    Created,
    Downloads,
    Rating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Descending,
    Ascending,
}

impl SearchParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_sort(mut self, sort_by: SortBy, sort_order: SortOrder) -> Self {
        self.sort_by = sort_by;
        self.sort_order = sort_order;
        self
    }

    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = limit;
        self.offset = offset;
        self
    }
}

/// Marketplace manager that handles both Mod.io and NexusMods
pub struct MarketplaceManager {
    modio_client: Option<ModioClient>,
    nexusmods_client: Option<NexusModsClient>,
    cache: MarketplaceCache,
    http_client: Client,
    config: ConfigManager,
}

impl MarketplaceManager {
    pub fn new(config: ConfigManager) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("LookingGlass/0.1.0")
            .build()?;

        let cache = MarketplaceCache::new()?;

        // Initialize Mod.io client if API key is configured
        let modio_client = if let Some(api_key) = config.get("mods.modio_api_key").and_then(|v| v.as_str()) {
            if let Some(game_id) = config.get("mods.modio_game_id").and_then(|v| v.as_i64()) {
                Some(ModioClient::new(http_client.clone(), api_key.to_string(), game_id as u32))
            } else {
                log::warn!("Mod.io game_id not configured, Mod.io client disabled");
                None
            }
        } else {
            log::debug!("Mod.io API key not configured, Mod.io client disabled");
            None
        };

        // Initialize NexusMods client if API key is configured
        let nexusmods_client = if let Some(api_key) = config.get("mods.nexusmods_api_key").and_then(|v| v.as_str()) {
            Some(NexusModsClient::new(http_client.clone(), api_key.to_string()))
        } else {
            log::debug!("NexusMods API key not configured, NexusMods client disabled");
            None
        };

        Ok(Self {
            modio_client,
            nexusmods_client,
            cache,
            http_client,
            config,
        })
    }

    /// Search mods across all configured marketplaces
    pub fn search(&self, params: SearchParams) -> Result<Vec<RemoteMod>> {
        let mut results = Vec::new();

        // Search Mod.io
        if let Some(ref client) = self.modio_client {
            match client.search(&params) {
                Ok(mods) => {
                    for m in mods {
                        results.push(self.convert_modio_mod(m));
                    }
                }
                Err(e) => log::warn!("Mod.io search failed: {}", e),
            }
        }

        // Search NexusMods
        if let Some(ref client) = self.nexusmods_client {
            match client.search(&params) {
                Ok(mods) => {
                    for m in mods {
                        results.push(self.convert_nexusmods_mod(m));
                    }
                }
                Err(e) => log::warn!("NexusMods search failed: {}", e),
            }
        }

        // Sort combined results
        self.sort_results(&mut results, params.sort_by, params.sort_order);

        Ok(results)
    }

    /// Get mod details by ID from a specific marketplace
    pub fn get_mod(&self, source: MarketplaceSource, mod_id: &str) -> Result<RemoteMod> {
        match source {
            MarketplaceSource::Modio => {
                let client = self.modio_client.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Mod.io client not configured"))?;
                let modio_mod = client.get_mod(mod_id)?;
                Ok(self.convert_modio_mod(modio_mod))
            }
            MarketplaceSource::NexusMods => {
                let client = self.nexusmods_client.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("NexusMods client not configured"))?;
                let nexus_mod = client.get_mod(mod_id)?;
                Ok(self.convert_nexusmods_mod(nexus_mod))
            }
        }
    }

    /// Download and install a mod from a marketplace
    pub fn install_mod(
        &self,
        source: MarketplaceSource,
        mod_id: &str,
        discovery: &ModDiscovery,
    ) -> Result<InstalledMod> {
        let remote_mod = self.get_mod(source, mod_id)?;

        let download_url = remote_mod.download_url
            .ok_or_else(|| anyhow::anyhow!("No download URL available for mod {}", mod_id))?;

        // Download the mod archive
        let archive_path = self.download_mod_archive(&download_url, &remote_mod)?;

        // Install using the existing discovery module
        let installed_mod = discovery.install_from_archive(&archive_path)?;

        // Clean up downloaded archive
        let _ = std::fs::remove_file(&archive_path);

        Ok(installed_mod)
    }

    /// Check for updates for installed mods
    pub fn check_updates(&self, installed_mods: &[InstalledMod]) -> Result<Vec<ModUpdateInfo>> {
        let mut updates = Vec::new();

        for installed in installed_mods {
            // Try to find the mod in marketplace
            // First check cache for source info
            if let Some(cached) = self.cache.get_mod_source(&installed.manifest.id) {
                match self.get_mod(cached.source, &cached.remote_id) {
                    Ok(remote_mod) => {
                        let remote_version = semver::Version::parse(&remote_mod.version)?;
                        let local_version = semver::Version::parse(&installed.manifest.version)?;

                        if remote_version > local_version {
                            updates.push(ModUpdateInfo {
                                mod_id: installed.manifest.id.clone(),
                                current_version: installed.manifest.version.clone(),
                                latest_version: remote_mod.version,
                                source: cached.source,
                                remote_mod,
                            });
                        }
                    }
                    Err(e) => {
                        log::debug!("Failed to check update for {}: {}", installed.manifest.id, e);
                    }
                }
            }
        }

        Ok(updates)
    }

    /// Update a mod to the latest version
    pub fn update_mod(
        &self,
        mod_id: &str,
        discovery: &ModDiscovery,
    ) -> Result<InstalledMod> {
        let installed_mods = discovery.discover()?;
        let installed = installed_mods.iter()
            .find(|m| m.manifest.id == mod_id)
            .ok_or_else(|| anyhow::anyhow!("Mod not installed: {}", mod_id))?;

        let cached = self.cache.get_mod_source(mod_id)
            .ok_or_else(|| anyhow::anyhow!("No marketplace source info for mod: {}", mod_id))?;

        // Uninstall old version
        discovery.uninstall(mod_id)?;

        // Install new version
        let new_mod = self.install_mod(cached.source, &cached.remote_id, discovery)?;

        Ok(new_mod)
    }

    /// List all installed mods with their marketplace info
    pub fn list_installed_with_source(&self, discovery: &ModDiscovery) -> Result<Vec<InstalledModInfo>> {
        let installed_mods = discovery.discover()?;
        let mut result = Vec::new();

        for installed in installed_mods {
            let source_info = self.cache.get_mod_source(&installed.manifest.id);
            result.push(InstalledModInfo {
                installed_mod: installed,
                marketplace_source: source_info.map(|s| s.source),
                remote_id: source_info.map(|s| s.remote_id),
            });
        }

        Ok(result)
    }

    /// Download a mod archive from a URL
    fn download_mod_archive(&self, url: &str, remote_mod: &RemoteMod) -> Result<PathBuf> {
        let cache_dir = self.cache.cache_dir()?;
        let filename = url.split('/').last().unwrap_or("mod.zip");
        let safe_filename = format!("{}_{}", remote_mod.id.replace('/', "_"), filename);
        let archive_path = cache_dir.join(&safe_filename);

        // Check if already cached
        if archive_path.exists() {
            if let Some(expected_hash) = &remote_mod.file_hash {
                let actual_hash = self.compute_file_hash(&archive_path)?;
                if actual_hash == *expected_hash {
                    log::info!("Using cached mod archive: {}", archive_path.display());
                    return Ok(archive_path);
                }
            } else {
                log::info!("Using cached mod archive: {}", archive_path.display());
                return Ok(archive_path);
            }
        }

        log::info!("Downloading mod from: {}", url);
        let mut response = self.http_client.get(url).send()?;
        response.error_for_status_ref()?;

        let mut file = std::fs::File::create(&archive_path)?;
        std::io::copy(&mut response, &mut file)?;

        // Verify hash if provided
        if let Some(expected_hash) = &remote_mod.file_hash {
            let actual_hash = self.compute_file_hash(&archive_path)?;
            if actual_hash != *expected_hash {
                std::fs::remove_file(&archive_path)?;
                return Err(anyhow::anyhow!(
                    "File hash mismatch: expected {}, got {}",
                    expected_hash,
                    actual_hash
                ));
            }
        }

        Ok(archive_path)
    }

    fn compute_file_hash(&self, path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};
        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn convert_modio_mod(&self, modio_mod: ModioMod) -> RemoteMod {
        let download_url = modio_mod.modfile.as_ref().and_then(|mf| mf.download.binary_url.clone());
        let file_size = modio_mod.modfile.as_ref().map(|mf| mf.filesize as u64);
        let file_hash = modio_mod.modfile.as_ref().and_then(|mf| mf.filehash.md5.clone());
        let version = modio_mod.modfile.as_ref()
            .map(|mf| mf.version.clone())
            .unwrap_or_else(|| "1.0.0".to_string());

        let mut images = Vec::new();
        if let Some(logo) = modio_mod.logo.as_ref() {
            images.push(logo.original.clone());
            images.push(logo.thumb_640x360.clone());
        }
        if let Some(media) = modio_mod.media.as_ref() {
            for img in &media.images {
                images.push(img.original.clone());
            }
        }

        let mut tags = Vec::new();
        if let Some(tag_list) = modio_mod.tags.as_ref() {
            for tag in tag_list {
                tags.push(tag.name.clone());
            }
        }

        RemoteMod {
            source: MarketplaceSource::Modio,
            id: modio_mod.id.to_string(),
            name: modio_mod.name,
            summary: modio_mod.summary,
            description: modio_mod.description_plaintext.or(modio_mod.description),
            version,
            author: modio_mod.submitted_by.as_ref().map(|u| u.username.clone()),
            downloads: modio_mod.stats.as_ref().map(|s| s.downloads_total as u64).unwrap_or(0),
            rating: modio_mod.stats.as_ref().map(|s| s.ratings_weighted_aggregate as f32),
            tags,
            logo_url: modio_mod.logo.as_ref().map(|l| l.original.clone()),
            images,
            homepage_url: modio_mod.homepage_url,
            download_url,
            file_size,
            file_hash,
            updated_at: modio_mod.date_updated.map(|ts| chrono::DateTime::from_timestamp(ts as i64, 0)).flatten(),
            created_at: modio_mod.date_added.map(|ts| chrono::DateTime::from_timestamp(ts as i64, 0)).flatten(),
            dependencies: Vec::new(), // Would need to parse from modfile.dependencies
            game_versions: Vec::new(),
        }
    }

    fn convert_nexusmods_mod(&self, nexus_mod: NexusMod) -> RemoteMod {
        let (download_url, file_size, file_hash, version) = if let Some(file) = nexus_mod.files.first() {
            (
                file.download_url.clone(),
                Some(file.file_size as u64),
                file.file_hash.clone(),
                file.version.clone(),
            )
        } else {
            (None, None, None, "1.0.0".to_string())
        };

        let mut images = Vec::new();
        if let Some(picture) = nexus_mod.picture_url.as_ref() {
            images.push(picture.clone());
        }

        RemoteMod {
            source: MarketplaceSource::NexusMods,
            id: nexus_mod.mod_id.to_string(),
            name: nexus_mod.name,
            summary: nexus_mod.summary,
            description: nexus_mod.description,
            version,
            author: nexus_mod.author.as_ref().map(|a| a.username.clone()),
            downloads: nexus_mod.downloads as u64,
            rating: Some(nexus_mod.rating as f32),
            tags: nexus_mod.tags.unwrap_or_default(),
            logo_url: nexus_mod.picture_url,
            images,
            homepage_url: Some(format!("https://www.nexusmods.com/{}/mods/{}",
                nexus_mod.game_domain_name, nexus_mod.mod_id)),
            download_url,
            file_size,
            file_hash,
            updated_at: nexus_mod.updated_timestamp.map(|ts| chrono::DateTime::from_timestamp(ts as i64, 0)).flatten(),
            created_at: nexus_mod.uploaded_timestamp.map(|ts| chrono::DateTime::from_timestamp(ts as i64, 0)).flatten(),
            dependencies: nexus_mod.requirements.unwrap_or_default(),
            game_versions: Vec::new(),
        }
    }

    fn sort_results(&self, results: &mut [RemoteMod], sort_by: SortBy, sort_order: SortOrder) {
        results.sort_by(|a, b| {
            let cmp = match sort_by {
                SortBy::Relevance => {
                    // For relevance, we'd need search scores - fallback to downloads
                    b.downloads.cmp(&a.downloads)
                }
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Updated => {
                    let a_time = a.updated_at.unwrap_or(chrono::DateTime::UNIX_EPOCH);
                    let b_time = b.updated_at.unwrap_or(chrono::DateTime::UNIX_EPOCH);
                    b_time.cmp(&a_time)
                }
                SortBy::Created => {
                    let a_time = a.created_at.unwrap_or(chrono::DateTime::UNIX_EPOCH);
                    let b_time = b.created_at.unwrap_or(chrono::DateTime::UNIX_EPOCH);
                    b_time.cmp(&a_time)
                }
                SortBy::Downloads => b.downloads.cmp(&a.downloads),
                SortBy::Rating => {
                    let a_rating = a.rating.unwrap_or(0.0);
                    let b_rating = b.rating.unwrap_or(0.0);
                    b_rating.partial_cmp(&a_rating).unwrap_or(std::cmp::Ordering::Equal)
                }
            };

            match sort_order {
                SortOrder::Ascending => cmp.reverse(),
                SortOrder::Descending => cmp,
            }
        });
    }
}

/// Information about an installed mod with marketplace source
#[derive(Debug, Clone)]
pub struct InstalledModInfo {
    pub installed_mod: InstalledMod,
    pub marketplace_source: Option<MarketplaceSource>,
    pub remote_id: Option<String>,
}

/// Information about an available mod update
#[derive(Debug, Clone)]
pub struct ModUpdateInfo {
    pub mod_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub source: MarketplaceSource,
    pub remote_mod: RemoteMod,
}

/// Marketplace cache for storing mod source information and downloaded archives
mod cache {
    use super::*;
    use anyhow::{Context, Result};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct CachedModSource {
        pub mod_id: String,
        pub source: MarketplaceSource,
        pub remote_id: String,
        pub cached_at: chrono::DateTime<chrono::Utc>,
    }

    /// Cache for marketplace mod source information and downloaded archives
    pub struct MarketplaceCache {
        cache_dir: PathBuf,
        sources_file: PathBuf,
        sources: HashMap<String, CachedModSource>,
    }

    impl MarketplaceCache {
        /// Create a new marketplace cache
        pub fn new() -> Result<Self> {
            let cache_dir = Self::default_cache_dir()?;
            let sources_file = cache_dir.join("mod_sources.json");

            let sources = if sources_file.exists() {
                let content = fs::read_to_string(&sources_file)?;
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                HashMap::new()
            };

            // Ensure cache directory exists
            fs::create_dir_all(&cache_dir)?;

            Ok(Self {
                cache_dir,
                sources_file,
                sources,
            })
        }

        /// Get the cache directory
        pub fn cache_dir(&self) -> Result<PathBuf> {
            Ok(self.cache_dir.clone())
        }

        /// Get the default cache directory
        fn default_cache_dir() -> Result<PathBuf> {
            let base = dirs::cache_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
                .join("looking-glass")
                .join("marketplace");
            Ok(base)
        }

        /// Store mod source information
        pub fn set_mod_source(&mut self, mod_id: &str, source: MarketplaceSource, remote_id: &str) -> Result<()> {
            let cached = CachedModSource {
                mod_id: mod_id.to_string(),
                source,
                remote_id: remote_id.to_string(),
                cached_at: chrono::Utc::now(),
            };
            self.sources.insert(mod_id.to_string(), cached);
            self.save_sources()?;
            Ok(())
        }

        /// Get mod source information
        pub fn get_mod_source(&self, mod_id: &str) -> Option<CachedModSource> {
            self.sources.get(mod_id).cloned()
        }

        /// Remove mod source information
        pub fn remove_mod_source(&mut self, mod_id: &str) -> Result<()> {
            self.sources.remove(mod_id);
            self.save_sources()?;
            Ok(())
        }

        /// Save sources to disk
        fn save_sources(&self) -> Result<()> {
            let content = serde_json::to_string_pretty(&self.sources)?;
            fs::write(&self.sources_file, content)?;
            Ok(())
        }
    }

    impl Default for MarketplaceCache {
        fn default() -> Self {
            Self::new().expect("Failed to create marketplace cache")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_builder() {
        let params = SearchParams::new()
            .with_query("test".to_string())
            .with_tags(vec!["gameplay".to_string()])
            .with_sort(SortBy::Downloads, SortOrder::Descending)
            .with_pagination(20, 0);

        assert_eq!(params.query, Some("test".to_string()));
        assert_eq!(params.tags, vec!["gameplay"]);
        assert_eq!(params.sort_by, SortBy::Downloads);
        assert_eq!(params.sort_order, SortOrder::Descending);
        assert_eq!(params.limit, 20);
        assert_eq!(params.offset, 0);
    }
}