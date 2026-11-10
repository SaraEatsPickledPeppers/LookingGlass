//! Mod.io API client

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{MarketplaceSource, SearchParams, SortBy, SortOrder};

/// Mod.io client for API interactions
pub struct ModioClient {
    client: Client,
    api_key: String,
    game_id: u32,
    base_url: String,
}

impl ModioClient {
    pub fn new(client: Client, api_key: String, game_id: u32) -> Self {
        Self {
            client,
            api_key,
            game_id,
            base_url: "https://api.mod.io/v1".to_string(),
        }
    }

    /// Search for mods
    pub fn search(&self, params: &SearchParams) -> Result<Vec<ModioMod>> {
        let url = format!("{}/games/{}/mods", self.base_url, self.game_id);

        let mut query_params = HashMap::new();

        if let Some(q) = &params.query {
            query_params.insert("_q".to_string(), q.clone());
        }

        // Add tags filter
        for tag in &params.tags {
            query_params.entry("tags".to_string())
                .or_insert_with(String::new)
                .push_str(&format!("{},", tag));
        }

        // Add sorting
        let sort_field = match params.sort_by {
            SortBy::Name => "name",
            SortBy::Updated => "date_updated",
            SortBy::Created => "date_added",
            SortBy::Downloads => "downloads_total",
            SortBy::Rating => "ratings_weighted_aggregate",
            SortBy::Relevance => "_q", // Relevance is implicit with _q
        };

        let sort_prefix = match params.sort_order {
            SortOrder::Ascending => "",
            SortOrder::Descending => "-",
        };
        query_params.insert("_sort".to_string(), format!("{}{}", sort_prefix, sort_field));

        // Add pagination
        query_params.insert("_limit".to_string(), params.limit.to_string());
        query_params.insert("_offset".to_string(), params.offset.to_string());

        let mut request = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key));

        for (key, value) in query_params {
            request = request.query(&[(key, value)]);
        }

        let response = request.send()?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("Mod.io API error {}: {}", status, error_text));
        }

        let search_result: ModioSearchResult = response.json()?;
        Ok(search_result.data)
    }

    /// Get a specific mod by ID
    pub fn get_mod(&self, mod_id: &str) -> Result<ModioMod> {
        let url = format!("{}/games/{}/mods/{}", self.base_url, self.game_id, mod_id);

        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("Mod.io API error {}: {}", status, error_text));
        }

        let mod_data: ModioMod = response.json()?;
        Ok(mod_data)
    }

    /// Get mod files for a specific mod
    pub fn get_mod_files(&self, mod_id: &str) -> Result<Vec<ModioModfile>> {
        let url = format!("{}/games/{}/mods/{}/files", self.base_url, self.game_id, mod_id);

        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("Mod.io API error {}: {}", status, error_text));
        }

        let search_result: ModioSearchResult<ModioModfile> = response.json()?;
        Ok(search_result.data)
    }
}

/// Mod.io search result wrapper
#[derive(Debug, Deserialize)]
struct ModioSearchResult<T> {
    data: Vec<T>,
    #[allow(dead_code)]
    result_count: u32,
    #[allow(dead_code)]
    result_limit: u32,
    #[allow(dead_code)]
    result_offset: u32,
    #[allow(dead_code)]
    result_total: u32,
}

/// Mod.io Mod object (subset of fields we need)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioMod {
    pub id: u32,
    pub game_id: u32,
    pub status: i32,
    pub visible: i32,
    #[serde(default)]
    pub submitted_by: Option<ModioUser>,
    pub date_added: u32,
    pub date_updated: u32,
    pub date_live: Option<u32>,
    #[serde(default)]
    pub maturity_option: i32,
    #[serde(default)]
    pub community_options: i32,
    #[serde(default)]
    pub monetization_options: i32,
    #[serde(default)]
    pub credit_options: i32,
    #[serde(default)]
    pub stock: i32,
    #[serde(default)]
    pub price: i32,
    #[serde(default)]
    pub tax: i32,
    #[serde(default)]
    pub logo: Option<ModioLogo>,
    #[serde(default)]
    pub homepage_url: Option<String>,
    pub name: String,
    pub name_id: String,
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub description_plaintext: Option<String>,
    #[serde(default)]
    pub metadata_blob: Option<String>,
    #[serde(default)]
    pub profile_url: Option<String>,
    #[serde(default)]
    pub media: Option<ModioMedia>,
    #[serde(default)]
    pub modfile: Option<ModioModfile>,
    #[serde(default)]
    pub dependencies: bool,
    #[serde(default)]
    pub platforms: Vec<ModioPlatform>,
    #[serde(default)]
    pub metadata_kvp: Vec<ModioMetadataKVP>,
    #[serde(default)]
    pub tags: Vec<ModioTag>,
    #[serde(default)]
    pub stats: Option<ModioStats>,
    #[serde(default)]
    pub skus: Vec<ModioSku>,
}

/// Mod.io User object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioUser {
    pub id: u32,
    pub username: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub profile_url: Option<String>,
}

/// Mod.io Logo object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioLogo {
    pub filename: String,
    pub original: String,
    pub thumb_320x180: String,
    pub thumb_640x360: String,
    pub thumb_1280x720: String,
}

/// Mod.io Media object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioMedia {
    #[serde(default)]
    pub youtube: Vec<ModioYoutubeVideo>,
    #[serde(default)]
    pub sketchfab: Vec<ModioSketchfabModel>,
    #[serde(default)]
    pub images: Vec<ModioImage>,
}

/// Mod.io Youtube video
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioYoutubeVideo {
    pub video_id: String,
}

/// Mod.io Sketchfab model
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioSketchfabModel {
    pub uid: String,
}

/// Mod.io Image
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioImage {
    pub filename: String,
    pub original: String,
    pub thumb_320x180: String,
    pub thumb_1280x720: String,
}

/// Mod.io Modfile object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioModfile {
    pub id: u32,
    pub mod_id: u32,
    pub date_added: u32,
    pub date_updated: u32,
    pub date_scanned: u32,
    pub virus_status: i32,
    pub virus_positive: i32,
    pub filesize: u64,
    pub filesize_uncompressed: u64,
    pub filehash: ModioFileHash,
    pub filename: String,
    pub version: String,
    pub changelog: Option<String>,
    #[serde(default)]
    pub metadata_blob: Option<String>,
    #[serde(default)]
    pub download: ModioDownload,
    #[serde(default)]
    pub platforms: Vec<ModioModfilePlatform>,
    #[serde(default)]
    pub dependencies: Vec<ModioDependency>,
}

/// Mod.io File Hash
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioFileHash {
    pub md5: Option<String>,
}

/// Mod.io Download object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioDownload {
    pub binary_url: Option<String>,
    pub date_expires: u32,
}

/// Mod.io Modfile Platform
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioModfilePlatform {
    pub platform: String,
    pub status: i32,
}

/// Mod.io Dependency
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioDependency {
    pub mod_id: u32,
    pub date_added: u32,
}

/// Mod.io Platform
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioPlatform {
    pub platform: String,
    pub modfile_live: u32,
}

/// Mod.io Metadata Key-Value Pair
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioMetadataKVP {
    pub metakey: String,
    pub metavalue: String,
}

/// Mod.io Tag
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioTag {
    pub name: String,
    pub name_localized: String,
    pub date_added: u32,
}

/// Mod.io Stats
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioStats {
    pub mod_id: u32,
    pub popularity_rank_position: u32,
    pub popularity_rank_total_mods: u32,
    pub downloads_today: u32,
    pub downloads_total: u32,
    pub subscribers_total: u32,
    pub ratings_total: u32,
    pub ratings_positive: u32,
    pub ratings_negative: u32,
    pub ratings_percentage_positive: f32,
    pub ratings_weighted_aggregate: f32,
    pub ratings_display_text: String,
    pub date_expires: u32,
}

/// Mod.io SKU
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModioSku {
    pub id: u32,
    pub sku: String,
    pub portal: String,
}

/// Search parameters for Mod.io
#[derive(Debug, Clone, Default)]
pub struct ModioSearchParams {
    pub query: Option<String>,
    pub tags: Vec<String>,
    pub sort_by: String,
    pub sort_order: String,
    pub limit: usize,
    pub offset: usize,
}

impl ModioSearchParams {
    pub fn new() -> Self {
        Self::default()
    }
}