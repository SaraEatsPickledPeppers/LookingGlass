//! NexusMods API client

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{SearchParams, SortBy, SortOrder};

/// NexusMods client for API interactions
pub struct NexusModsClient {
    client: Client,
    api_key: String,
    base_url: String,
    game_domain_name: String,
}

impl NexusModsClient {
    pub fn new(client: Client, api_key: String) -> Self {
        // Default to American McGee's Alice
        Self {
            client,
            api_key,
            base_url: "https://api.nexusmods.com/v1".to_string(),
            game_domain_name: "american-mcgees-alice".to_string(),
        }
    }

    /// Set the game domain name (e.g., "skyrim", "fallout4", "american-mcgees-alice")
    pub fn with_game_domain(mut self, game_domain_name: String) -> Self {
        self.game_domain_name = game_domain_name;
        self
    }

    /// Search for mods using the "latest added" endpoint with filters
    /// Note: NexusMods API doesn't have a direct search endpoint, so we use latest/trending/updated
    /// and filter client-side, or use the md5 search for specific files
    pub fn search(&self, params: &SearchParams) -> Result<Vec<NexusMod>> {
        let mut all_mods = Vec::new();

        // Try multiple endpoints to get a good set of mods
        let endpoints = vec![
            ("latest_added", format!("{}/games/{}/mods/latest_added.json", self.base_url, self.game_domain_name)),
            ("trending", format!("{}/games/{}/mods/trending.json", self.base_url, self.game_domain_name)),
            ("latest_updated", format!("{}/games/{}/mods/latest_updated.json", self.base_url, self.game_domain_name)),
        ];

        for (endpoint_name, url) in endpoints {
            log::debug!("Fetching mods from NexusMods endpoint: {}", endpoint_name);
            match self.fetch_mods_from_endpoint(&url) {
                Ok(mods) => {
                    all_mods.extend(mods);
                }
                Err(e) => {
                    log::warn!("Failed to fetch from {}: {}", endpoint_name, e);
                }
            }
        }

        // Deduplicate by mod_id
        let mut seen = std::collections::HashSet::new();
        all_mods.retain(|m| seen.insert(m.mod_id));

        // Apply client-side filtering
        let mut filtered = self.filter_mods(all_mods, params);

        // Sort results
        self.sort_mods(&mut filtered, params.sort_by, params.sort_order);

        // Apply pagination
        let start = params.offset.min(filtered.len());
        let end = (start + params.limit).min(filtered.len());
        Ok(filtered[start..end].to_vec())
    }

    /// Fetch mods from a specific endpoint
    fn fetch_mods_from_endpoint(&self, url: &str) -> Result<Vec<NexusMod>> {
        let response = self.client.get(url)
            .header("apikey", &self.api_key)
            .header("User-Agent", "LookingGlass/0.1.0")
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("NexusMods API error {}: {}", status, error_text));
        }

        let result: NexusModListResult = response.json()?;
        Ok(result.mods)
    }

    /// Get a specific mod by ID
    pub fn get_mod(&self, mod_id: &str) -> Result<NexusMod> {
        let url = format!("{}/games/{}/mods/{}.json", self.base_url, self.game_domain_name, mod_id);

        let response = self.client.get(&url)
            .header("apikey", &self.api_key)
            .header("User-Agent", "LookingGlass/0.1.0")
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("NexusMods API error {}: {}", status, error_text));
        }

        let mod_data: NexusMod = response.json()?;
        Ok(mod_data)
    }

    /// Get mod files for a specific mod
    pub fn get_mod_files(&self, mod_id: &str) -> Result<Vec<NexusModFile>> {
        let url = format!("{}/games/{}/mods/{}/files.json", self.base_url, self.game_domain_name, mod_id);

        let response = self.client.get(&url)
            .header("apikey", &self.api_key)
            .header("User-Agent", "LookingGlass/0.1.0")
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("NexusMods API error {}: {}", status, error_text));
        }

        let result: NexusModFilesResult = response.json()?;
        Ok(result.files)
    }

    /// Get download URL for a specific file
    pub fn get_download_url(&self, mod_id: &str, file_id: &str) -> Result<String> {
        let url = format!("{}/games/{}/mods/{}/files/{}/download_link.json",
            self.base_url, self.game_domain_name, mod_id, file_id);

        let response = self.client.get(&url)
            .header("apikey", &self.api_key)
            .header("User-Agent", "LookingGlass/0.1.0")
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow::anyhow!("NexusMods API error {}: {}", status, error_text));
        }

        let result: NexusDownloadLink = response.json()?;
        Ok(result.download_link)
    }

    /// Client-side filtering of mods
    fn filter_mods(&self, mods: Vec<NexusMod>, params: &SearchParams) -> Vec<NexusMod> {
        mods.into_iter().filter(|m| {
            // Filter by query (search in name and summary)
            if let Some(q) = &params.query {
                let q_lower = q.to_lowercase();
                let name_match = m.name.to_lowercase().contains(&q_lower);
                let summary_match = m.summary.as_ref()
                    .map(|s| s.to_lowercase().contains(&q_lower))
                    .unwrap_or(false);
                if !name_match && !summary_match {
                    return false;
                }
            }

            // Filter by tags
            if !params.tags.is_empty() {
                if let Some(mod_tags) = &m.tags {
                    let has_tag = params.tags.iter().any(|t|
                        mod_tags.iter().any(|mt| mt.eq_ignore_ascii_case(t))
                    );
                    if !has_tag {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        }).collect()
    }

    /// Sort mods
    fn sort_mods(&self, mods: &mut [NexusMod], sort_by: SortBy, sort_order: SortOrder) {
        mods.sort_by(|a, b| {
            let cmp = match sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Updated => {
                    let a_time = a.updated_timestamp.unwrap_or(0);
                    let b_time = b.updated_timestamp.unwrap_or(0);
                    b_time.cmp(&a_time)
                }
                SortBy::Created => {
                    let a_time = a.uploaded_timestamp.unwrap_or(0);
                    let b_time = b.uploaded_timestamp.unwrap_or(0);
                    b_time.cmp(&a_time)
                }
                SortBy::Downloads => b.downloads.cmp(&a.downloads),
                SortBy::Rating => {
                    let a_rating = a.rating as f32;
                    let b_rating = b.rating as f32;
                    b_rating.partial_cmp(&a_rating).unwrap_or(std::cmp::Ordering::Equal)
                }
                SortBy::Relevance => b.downloads.cmp(&a.downloads), // Fallback to downloads
            };

            match sort_order {
                SortOrder::Ascending => cmp.reverse(),
                SortOrder::Descending => cmp,
            }
        });
    }
}

/// NexusMods mod list result
#[derive(Debug, Deserialize)]
struct NexusModListResult {
    mods: Vec<NexusMod>,
}

/// NexusMods mod files result
#[derive(Debug, Deserialize)]
struct NexusModFilesResult {
    files: Vec<NexusModFile>,
}

/// NexusMods download link response
#[derive(Debug, Deserialize)]
struct NexusDownloadLink {
    #[serde(rename = "download_link")]
    download_link: String,
}

/// NexusMods Mod object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NexusMod {
    pub mod_id: u32,
    pub game_id: u32,
    pub game_domain_name: String,
    pub status: String,
    pub visibility: String,
    pub name: String,
    pub name_id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub description_html: Option<String>,
    pub version: Option<String>,
    pub category_id: Option<u32>,
    pub category_name: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub author: Option<NexusUser>,
    #[serde(default)]
    pub picture_url: Option<String>,
    #[serde(default)]
    pub uploaded_timestamp: Option<u32>,
    #[serde(default)]
    pub updated_timestamp: Option<u32>,
    #[serde(default)]
    pub downloads: u32,
    #[serde(default)]
    pub unique_downloads: u32,
    #[serde(default)]
    pub rating: f32,
    #[serde(default)]
    pub endorsement_count: u32,
    #[serde(default)]
    pub files: Vec<NexusModFile>,
    #[serde(default)]
    pub requirements: Option<Vec<String>>,
}

/// NexusMods User object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NexusUser {
    pub user_id: u32,
    pub username: String,
    #[serde(default)]
    pub profile_url: Option<String>,
}

/// NexusMods Mod File object
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NexusModFile {
    pub file_id: u32,
    pub file_name: String,
    pub file_size: u64,
    pub version: String,
    pub category_id: u32,
    pub category_name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub requirements: Option<Vec<String>>,
    pub uploaded_timestamp: u32,
    pub virus_scan_status: String,
    #[serde(default)]
    pub file_hash: Option<String>,
    #[serde(default)]
    pub download_url: Option<String>,
}

/// NexusMods search parameters (extends base SearchParams)
#[derive(Debug, Clone, Default)]
pub struct NexusSearchParams {
    pub query: Option<String>,
    pub category_id: Option<u32>,
    pub tags: Vec<String>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub limit: usize,
    pub offset: usize,
}

impl NexusSearchParams {
    pub fn new() -> Self {
        Self::default()
    }
}