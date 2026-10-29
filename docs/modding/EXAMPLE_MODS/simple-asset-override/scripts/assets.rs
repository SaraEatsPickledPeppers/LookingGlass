// Simple Asset Override Example
// This module demonstrates how to override a single texture asset

use looking_glass_mod::loader::{AssetOverride, AssetOverrideResult, AssetMapping, AssetType, ModApi};
use std::path::PathBuf;
use std::sync::Arc;

pub struct SimpleAssetOverride {
    mod_dir: PathBuf,
}

impl SimpleAssetOverride {
    pub fn new(mod_dir: PathBuf) -> Self {
        Self { mod_dir }
    }
}

#[async_trait::async_trait]
impl AssetOverride for SimpleAssetOverride {
    /// Try to provide an override for the given target asset
    async fn try_override(&self, target: &str) -> Option<AssetOverrideResult> {
        // Only override the specific wall texture
        if target == "textures/wall.png" {
            let source_path = self.mod_dir.join("assets/textures/wall_replacement.png");

            // Read the replacement texture
            let data = std::fs::read(&source_path).ok()?;

            Some(AssetOverrideResult {
                data,
                mime_type: "image/png".to_string(),
            })
        } else {
            None
        }
    }

    /// List all overrides this module provides (for debugging)
    async fn list_overrides(&self) -> Vec<AssetMapping> {
        vec![AssetMapping {
            source: "assets/textures/wall_replacement.png".to_string(),
            target: "textures/wall.png".to_string(),
            asset_type: AssetType::Texture,
            condition: None,
        }]
    }
}

/// Register this asset override with the mod API
pub fn register(api: &ModApi, mod_dir: PathBuf) {
    let override_impl = Arc::new(SimpleAssetOverride::new(mod_dir));
    // Note: In practice, you'd register this through the ModRegistry
    // This is a simplified example showing the pattern
}