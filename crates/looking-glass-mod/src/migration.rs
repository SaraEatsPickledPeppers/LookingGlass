//! Migration utilities for mod manifest versions

use crate::manifest::{AssetMapping, AssetType, GameTarget, LoadOrder, ModDependency, ModEntryPoints, ModManifest};
use anyhow::{Context, Result};
use semver::VersionReq;
use serde_json::Value;

/// Migrate a v1 manifest to v2
pub fn migrate_manifest_v1_to_v2(mut manifest: ModManifest) -> Result<ModManifest> {
    // v1 didn't have manifest_version, so we assume any without it is v1
    // v1 used different field names and structure

    // Set manifest version to 2
    manifest.manifest_version = 2;

    // Migrate game_target if it was a string
    // This is handled by serde deserialization

    // Ensure dependencies have optional field defaulted to false
    for dep in &mut manifest.dependencies {
        // VersionReq is already validated
    }

    // Ensure load_order exists with empty arrays
    // Default trait handles this

    // Ensure entry_points exists
    // Default trait handles this

    // Migrate assets if they used old format
    for asset in &mut manifest.assets {
        // Old format might not have asset_type or condition
        // serde will default asset_type to Texture and condition to None
    }

    // Ensure tags exists
    // Default trait handles this

    // Migrate config_schema and default_config if they existed in v1 under different names
    // For now, we just keep them as-is if present

    Ok(manifest)
}

/// Migrate a v1 manifest from a raw JSON value
pub fn migrate_manifest_v1_to_v2_from_value(mut value: Value) -> Result<Value> {
    // Add manifest_version if missing
    if value.get("manifest_version").is_none() {
        value["manifest_version"] = Value::Number(2.into());
    }

    // Ensure game_target is properly formatted
    if let Some(gt) = value.get("game_target") {
        if let Some(s) = gt.as_str() {
            // Convert to PascalCase if needed
            let pascal = match s.to_lowercase().as_str() {
                "alice" => "Alice",
                "alicedemo" => "AliceDemo",
                _ => s,
            };
            value["game_target"] = Value::String(pascal.to_string());
        }
    }

    // Ensure dependencies have optional field
    if let Some(deps) = value.get_mut("dependencies") {
        if let Some(arr) = deps.as_array_mut() {
            for dep in arr {
                if dep.get("optional").is_none() {
                    dep["optional"] = Value::Bool(false);
                }
            }
        }
    }

    // Ensure load_order exists
    if value.get("load_order").is_none() {
        value["load_order"] = serde_json::json!({"before": [], "after": []});
    }

    // Ensure entry_points exists
    if value.get("entry_points").is_none() {
        value["entry_points"] = serde_json::json!({});
    }

    // Ensure tags exists
    if value.get("tags").is_none() {
        value["tags"] = Value::Array(vec![]);
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ModManifest, GameTarget};
    use serde_json::json;

    #[test]
    fn test_migrate_v1_to_v2() {
        let v1_manifest = ModManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            manifest_version: 1,
            game_target: GameTarget::Alice,
            ..Default::default()
        };

        let v2 = migrate_manifest_v1_to_v2(v1_manifest).unwrap();
        assert_eq!(v2.manifest_version, 2);
    }

    #[test]
    fn test_migrate_from_value() {
        let v1 = json!({
            "id": "test",
            "name": "Test",
            "version": "1.0.0",
            "game_target": "alice"
        });

        let v2 = migrate_manifest_v1_to_v2_from_value(v1).unwrap();
        assert_eq!(v2["manifest_version"], 2);
        assert_eq!(v2["game_target"], "Alice");
        assert!(v2["load_order"].is_object());
        assert!(v2["entry_points"].is_object());
        assert!(v2["tags"].is_array());
    }
}