use crate::migration::migrate_manifest_v1_to_v2;
use anyhow::{Context, Result};
use jsonschema::validator_for;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Helper for serializing/deserializing VersionReq
mod version_req_serde {
    use semver::VersionReq;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(version_req: &VersionReq, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&version_req.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<VersionReq, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        VersionReq::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum GameTarget {
    #[default]
    Alice,
    AliceDemo,
}

impl GameTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameTarget::Alice => "Alice",
            GameTarget::AliceDemo => "AliceDemo",
        }
    }
}

impl std::fmt::Display for GameTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for GameTarget {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Alice" => Ok(GameTarget::Alice),
            "AliceDemo" => Ok(GameTarget::AliceDemo),
            _ => Err(anyhow::anyhow!("Unknown game target: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub id: String,
    #[serde(with = "crate::manifest::version_req_serde")]
    pub version: VersionReq,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ModEntryPoints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_inject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_hook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_patch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_hook: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AssetType {
    Texture,
    Model,
    Sound,
    Font,
    Ui,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMapping {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub asset_type: AssetType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LoadOrder {
    #[serde(default)]
    pub before: Vec<String>,
    #[serde(default)]
    pub after: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub manifest_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    pub game_target: GameTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_game_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_game_version: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<ModDependency>,
    #[serde(default)]
    pub load_order: LoadOrder,
    #[serde(default)]
    pub entry_points: ModEntryPoints,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_config: Option<serde_json::Value>,
    #[serde(default)]
    pub assets: Vec<AssetMapping>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for ModManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: String::new(),
            manifest_version: 2,
            author: None,
            description: None,
            homepage: None,
            repository: None,
            license: None,
            game_target: GameTarget::default(),
            min_game_version: None,
            max_game_version: None,
            dependencies: Vec::new(),
            load_order: LoadOrder::default(),
            entry_points: ModEntryPoints::default(),
            config_schema: None,
            default_config: None,
            assets: Vec::new(),
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMod {
    pub manifest: ModManifest,
    pub path: PathBuf,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub checksum: String,
}

impl Default for InstalledMod {
    fn default() -> Self {
        Self {
            manifest: ModManifest::default(),
            path: PathBuf::new(),
            enabled: true,
            config: None,
            installed_at: chrono::Utc::now(),
            checksum: String::new(),
        }
    }
}

impl ModManifest {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest file: {}", path.display()))?;

        let mut manifest: ModManifest = if path.extension().map_or(false, |e| e == "toml") {
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse TOML manifest: {}", path.display()))?
        } else {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON manifest: {}", path.display()))?
        };

        // Migrate if needed
        if manifest.manifest_version == 1 {
            manifest = migrate_manifest_v1_to_v2(manifest)?;
        }

        // Validate against schema
        manifest.validate()?;

        Ok(manifest)
    }

    pub fn validate(&self) -> Result<()> {
        // Load and compile schema
        let schema_content = include_str!("../mod.schema.json");
        let schema: serde_json::Value = serde_json::from_str(schema_content)?;

        // Create validator using new jsonschema 0.60 API
        let validator = validator_for(&schema)
            .map_err(|e| anyhow::anyhow!("Failed to compile JSON schema: {}", e))?;

        // Validate
        let value = serde_json::to_value(self)?;
        if let Err(_) = validator.validate(&value) {
            // Collect all validation errors using iter_errors
            let error_msgs: Vec<String> = validator.iter_errors(&value)
                .map(|e| e.to_string())
                .collect();
            return Err(anyhow::anyhow!(
                "Manifest validation failed:\n{}",
                error_msgs.join("\n")
            ));
        }

        // Additional semantic validation
        self.validate_version()?;
        self.validate_dependencies()?;
        self.validate_game_version_constraints()?;

        Ok(())
    }

    fn validate_version(&self) -> Result<()> {
        semver::Version::parse(&self.version)
            .with_context(|| format!("Invalid version format: {}", self.version))?;
        Ok(())
    }

    fn validate_dependencies(&self) -> Result<()> {
        for dep in &self.dependencies {
            if dep.id == self.id {
                return Err(anyhow::anyhow!("Mod cannot depend on itself"));
            }
            // VersionReq is already validated by semver during deserialization
        }
        Ok(())
    }

    fn validate_game_version_constraints(&self) -> Result<()> {
        if let (Some(min), Some(max)) = (&self.min_game_version, &self.max_game_version) {
            let min_req = VersionReq::parse(&format!(">={}", min))?;
            let max_req = VersionReq::parse(&format!("<={}", max))?;

            // Check if there's any version that satisfies both
            // This is a simplified check - in practice you'd want more sophisticated logic
            if min > max {
                return Err(anyhow::anyhow!(
                    "min_game_version ({}) cannot be greater than max_game_version ({})",
                    min,
                    max
                ));
            }
        }
        Ok(())
    }

    pub fn compute_checksum(&self, mod_dir: &Path) -> Result<String> {
        let mut hasher = Sha256::new();
        let mut files: Vec<PathBuf> = Vec::new();

        for entry in WalkDir::new(mod_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            files.push(entry.path().to_path_buf());
        }

        files.sort();

        for file in files {
            let content = fs::read(&file)?;
            hasher.update(&content);
            // Include relative path in hash to catch file moves
            let rel_path = file.strip_prefix(mod_dir).unwrap_or(&file);
            hasher.update(rel_path.to_string_lossy().as_bytes());
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn satisfies_game_version(&self, game_version: &str) -> Result<bool> {
        let game_ver = semver::Version::parse(game_version)?;

        if let Some(min) = &self.min_game_version {
            let min_req = VersionReq::parse(&format!(">={}", min))?;
            if !min_req.matches(&game_ver) {
                return Ok(false);
            }
        }

        if let Some(max) = &self.max_game_version {
            let max_req = VersionReq::parse(&format!("<={}", max))?;
            if !max_req.matches(&game_ver) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl InstalledMod {
    pub fn load(mod_dir: &Path) -> Result<Self> {
        let manifest_path = mod_dir.join("mod.toml");
        if !manifest_path.exists() {
            let manifest_path = mod_dir.join("mod.json");
            if !manifest_path.exists() {
                return Err(anyhow::anyhow!("No manifest found in {}", mod_dir.display()));
            }
        }

        let manifest = ModManifest::load_from_file(&manifest_path)?;
        let checksum = manifest.compute_checksum(mod_dir)?;

        Ok(InstalledMod {
            manifest,
            path: mod_dir.to_path_buf(),
            enabled: true,
            config: None,
            installed_at: chrono::Utc::now(),
            checksum,
        })
    }

    pub fn verify_checksum(&self) -> Result<bool> {
        let current_checksum = self.manifest.compute_checksum(&self.path)?;
        Ok(current_checksum == self.checksum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_manifest_parsing() {
        let manifest_json = r#"
        {
            "id": "test_mod",
            "name": "Test Mod",
            "version": "1.0.0",
            "manifest_version": 2,
            "author": "Test Author",
            "game_target": "Alice",
            "dependencies": [
                {"id": "core", "version": ">=1.0.0", "optional": false}
            ],
            "load_order": {
                "before": ["other_mod"],
                "after": ["base_mod"]
            },
            "entry_points": {
                "asset_override": "assets.rs",
                "runtime_hook": "hooks.rs"
            },
            "assets": [
                {"source": "textures/test.png", "target": "textures/test.png", "type": "Texture"}
            ],
            "tags": ["gameplay", "visual"]
        }
        "#;

        let manifest: ModManifest = serde_json::from_str(manifest_json).unwrap();
        assert_eq!(manifest.id, "test_mod");
        assert_eq!(manifest.name, "Test Mod");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.game_target, GameTarget::Alice);
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(manifest.load_order.before, vec!["other_mod"]);
        assert_eq!(manifest.load_order.after, vec!["base_mod"]);
        assert!(manifest.entry_points.asset_override.is_some());
        assert_eq!(manifest.assets.len(), 1);
        assert_eq!(manifest.tags, vec!["gameplay", "visual"]);
    }

    #[test]
    fn test_manifest_toml_parsing() {
        let manifest_toml = r#"
        id = "test_mod"
        name = "Test Mod"
        version = "1.0.0"
        manifest_version = 2
        author = "Test Author"
        game_target = "AliceDemo"
        "#;

        let manifest: ModManifest = toml::from_str(manifest_toml).unwrap();
        assert_eq!(manifest.id, "test_mod");
        assert_eq!(manifest.game_target, GameTarget::AliceDemo);
    }

    #[test]
    fn test_version_constraints() {
        let manifest = ModManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            manifest_version: 2,
            game_target: GameTarget::Alice,
            min_game_version: Some("1.0.0".to_string()),
            max_game_version: Some("2.0.0".to_string()),
            ..Default::default()
        };

        assert!(manifest.satisfies_game_version("1.5.0").unwrap());
        assert!(manifest.satisfies_game_version("1.0.0").unwrap());
        assert!(manifest.satisfies_game_version("2.0.0").unwrap());
        assert!(!manifest.satisfies_game_version("0.9.0").unwrap());
        assert!(!manifest.satisfies_game_version("2.1.0").unwrap());
    }

    #[test]
    fn test_checksum_computation() {
        let dir = tempdir().unwrap();
        let mod_dir = dir.path().join("test_mod");
        fs::create_dir_all(&mod_dir).unwrap();

        fs::write(mod_dir.join("mod.toml"), "id = \"test\"\nname = \"Test\"\nversion = \"1.0.0\"\nmanifest_version = 2\ngame_target = \"Alice\"").unwrap();
        fs::write(mod_dir.join("file1.txt"), "content1").unwrap();
        fs::create_dir_all(mod_dir.join("assets")).unwrap();
        fs::write(mod_dir.join("assets").join("texture.png"), "fake png data").unwrap();

        let manifest = ModManifest::load_from_file(&mod_dir.join("mod.toml")).unwrap();
        let checksum1 = manifest.compute_checksum(&mod_dir).unwrap();
        let checksum2 = manifest.compute_checksum(&mod_dir).unwrap();
        assert_eq!(checksum1, checksum2);

        // Modify a file
        fs::write(mod_dir.join("file1.txt"), "content2").unwrap();
        let checksum3 = manifest.compute_checksum(&mod_dir).unwrap();
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_dependency_validation() {
        let mut manifest = ModManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            manifest_version: 2,
            game_target: GameTarget::Alice,
            ..Default::default()
        };

        // Self-dependency should fail
        manifest.dependencies.push(ModDependency {
            id: "test".to_string(),
            version: VersionReq::parse(">=1.0.0").unwrap(),
            optional: false,
        });

        assert!(manifest.validate_dependencies().is_err());
    }
}