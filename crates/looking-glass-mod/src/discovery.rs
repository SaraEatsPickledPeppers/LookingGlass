use crate::manifest::{InstalledMod, ModManifest, Permission, PermissionSet, Permissions};
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use jsonschema::validator_for;
use semver::VersionReq;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tar::Archive;
use tempfile::tempdir;
use walkdir::WalkDir;
use zip::ZipArchive;

pub struct ModDiscovery {
    pub base_dir: PathBuf,
    pub game_id: String,
}

impl ModDiscovery {
    pub fn new(base_dir: PathBuf, game_id: String) -> Self {
        Self { base_dir, game_id }
    }

    pub fn mods_dir(&self) -> PathBuf {
        self.base_dir.join("mods").join(&self.game_id)
    }

    pub fn discover(&self) -> Result<Vec<InstalledMod>> {
        let mods_dir = self.mods_dir();
        if !mods_dir.exists() {
            return Ok(Vec::new());
        }

        let mut mods = Vec::new();

        for entry in WalkDir::new(&mods_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            let mod_dir = entry.path();
            match InstalledMod::load(mod_dir) {
                Ok(installed_mod) => {
                    // Verify checksum
                    if installed_mod.verify_checksum().unwrap_or(false) {
                        mods.push(installed_mod);
                    } else {
                        log::warn!("Mod checksum mismatch: {}", mod_dir.display());
                    }
                }
                Err(e) => {
                    log::warn!("Failed to load mod at {}: {}", mod_dir.display(), e);
                }
            }
        }

        Ok(mods)
    }

    pub fn install_from_archive(&self, archive_path: &Path) -> Result<InstalledMod> {
        let temp_dir = tempdir()?;
        let extract_dir = temp_dir.path();

        // Extract archive
        self.extract_archive(archive_path, extract_dir)?;

        // Find manifest
        let manifest_path = self.find_manifest(extract_dir)?;
        let manifest = ModManifest::load_from_file(&manifest_path)?;

        // Validate game target
        if manifest.game_target.as_str() != self.game_id {
            return Err(anyhow::anyhow!(
                "Mod targets {} but current game is {}",
                manifest.game_target,
                self.game_id
            ));
        }

        // Validate dependencies
        self.validate_dependencies(&manifest)?;

        // Validate asset files exist
        self.validate_assets(&manifest, extract_dir)?;

        // Validate entry point files exist
        self.validate_entry_points(&manifest, extract_dir)?;

        // Validate config schema and default_config
        self.validate_config(&manifest)?;

        // Validate load_order references
        self.validate_load_order(&manifest)?;

        // Validate permissions
        self.validate_permissions(&manifest)?;

        // Compute checksum
        let checksum = manifest.compute_checksum(extract_dir)?;

        // Check if mod already exists
        let target_dir = self.mods_dir().join(&manifest.id);
        if target_dir.exists() {
            // Check if same version
            let existing = InstalledMod::load(&target_dir)?;
            if existing.manifest.version == manifest.version
                && existing.checksum == checksum
            {
                return Err(anyhow::anyhow!(
                    "Mod {} v{} is already installed",
                    manifest.id,
                    manifest.version
                ));
            }
            // Remove old version
            fs::remove_dir_all(&target_dir)?;
        }

        // Move to mods directory
        fs::create_dir_all(&self.mods_dir())?;
        fs::rename(extract_dir, &target_dir)?;

        // Create permission set - by default, grant optional permissions and required permissions
        // In a real implementation, this would prompt the user for consent
        let permission_set = Self::create_permission_set(&manifest)?;

        let installed_mod = InstalledMod {
            manifest,
            path: target_dir.clone(),
            enabled: true,
            config: None,
            installed_at: chrono::Utc::now(),
            checksum,
            permissions: permission_set,
        };

        Ok(installed_mod)
    }

    /// Validate permissions in manifest
    fn validate_permissions(&self, manifest: &ModManifest) -> Result<()> {
        // Validate the permissions struct
        manifest.permissions.validate()?;

        // Check for potentially dangerous permissions and warn
        let dangerous_permissions = [
            Permission::ProcessSpawn,
            Permission::NativeCodeExecution,
            Permission::NetworkAccess,
        ];

        for perm in &dangerous_permissions {
            if manifest.permissions.all_requested().contains(perm) {
                tracing::warn!("Mod '{}' requests potentially dangerous permission: {}", manifest.id, perm);
            }
        }

        Ok(())
    }

    /// Create a permission set for the installed mod
    /// In a real implementation, this would prompt the user for consent
    fn create_permission_set(manifest: &ModManifest) -> Result<PermissionSet> {
        let mut permission_set = PermissionSet::default();

        // Grant all required permissions by default
        for perm in &manifest.permissions.required {
            permission_set.grant(perm.clone());
        }

        // Grant all optional permissions by default (in a real implementation, this would be user-configurable)
        for perm in &manifest.permissions.optional {
            permission_set.grant(perm.clone());
        }

        // Explicitly deny not_needed permissions
        for perm in &manifest.permissions.not_needed {
            permission_set.deny(perm.clone());
        }

        Ok(permission_set)
    }

    fn validate_assets(&self, manifest: &ModManifest, extract_dir: &Path) -> Result<()> {
        for asset in &manifest.assets {
            let source_path = extract_dir.join(&asset.source);
            if !source_path.exists() {
                return Err(anyhow::anyhow!(
                    "Asset file not found: {} (referenced in manifest)",
                    asset.source
                ));
            }
            if !source_path.is_file() {
                return Err(anyhow::anyhow!(
                    "Asset path is not a file: {}",
                    asset.source
                ));
            }
        }
        Ok(())
    }

    fn validate_entry_points(&self, manifest: &ModManifest, extract_dir: &Path) -> Result<()> {
        let entry_points = &manifest.entry_points;

        if let Some(asset_override) = &entry_points.asset_override {
            let path = extract_dir.join(asset_override);
            if !path.exists() {
                return Err(anyhow::anyhow!(
                    "Asset override entry point not found: {}",
                    asset_override
                ));
            }
        }

        if let Some(config_inject) = &entry_points.config_inject {
            let path = extract_dir.join(config_inject);
            if !path.exists() {
                return Err(anyhow::anyhow!(
                    "Config inject entry point not found: {}",
                    config_inject
                ));
            }
        }

        if let Some(runtime_hook) = &entry_points.runtime_hook {
            let path = extract_dir.join(runtime_hook);
            if !path.exists() {
                return Err(anyhow::anyhow!(
                    "Runtime hook entry point not found: {}",
                    runtime_hook
                ));
            }
        }

        if let Some(ui_patch) = &entry_points.ui_patch {
            let path = extract_dir.join(ui_patch);
            if !path.exists() {
                return Err(anyhow::anyhow!(
                    "UI patch entry point not found: {}",
                    ui_patch
                ));
            }
        }

        if let Some(input_hook) = &entry_points.input_hook {
            let path = extract_dir.join(input_hook);
            if !path.exists() {
                return Err(anyhow::anyhow!(
                    "Input hook entry point not found: {}",
                    input_hook
                ));
            }
        }

        Ok(())
    }

    fn validate_config(&self, manifest: &ModManifest) -> Result<()> {
        // If config_schema is provided, validate it's a valid JSON schema
        if let Some(config_schema) = &manifest.config_schema {
            validator_for(config_schema)
                .map_err(|e| anyhow::anyhow!("Invalid config_schema: {}", e))?;

            // If default_config is also provided, validate it against the schema
            if let Some(default_config) = &manifest.default_config {
                let validator = validator_for(config_schema)
                    .map_err(|e| anyhow::anyhow!("Failed to compile config schema: {}", e))?;

                if let Err(_) = validator.validate(default_config) {
                    let error_msgs: Vec<String> = validator.iter_errors(default_config)
                        .map(|e| e.to_string())
                        .collect();
                    return Err(anyhow::anyhow!(
                        "default_config does not match config_schema:\n{}",
                        error_msgs.join("\n")
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_load_order(&self, manifest: &ModManifest) -> Result<()> {
        let installed_mods = self.discover()?;
        let installed_ids: std::collections::HashSet<_> = installed_mods
            .iter()
            .map(|m| m.manifest.id.clone())
            .collect();

        // Validate 'before' references
        for before_id in &manifest.load_order.before {
            if before_id == &manifest.id {
                return Err(anyhow::anyhow!("Mod cannot specify itself in load_order.before"));
            }
            // Only warn if the referenced mod is not installed (not an error since it might be installed later)
            if !installed_ids.contains(before_id) {
                log::info!("load_order.before references mod '{}' which is not currently installed", before_id);
            }
        }

        // Validate 'after' references
        for after_id in &manifest.load_order.after {
            if after_id == &manifest.id {
                return Err(anyhow::anyhow!("Mod cannot specify itself in load_order.after"));
            }
            if !installed_ids.contains(after_id) {
                log::info!("load_order.after references mod '{}' which is not currently installed", after_id);
            }
        }

        // Check for circular dependencies in load_order (simplified check)
        // This is a basic check - full cycle detection would require graph analysis
        for before_id in &manifest.load_order.before {
            if manifest.load_order.after.contains(before_id) {
                return Err(anyhow::anyhow!(
                    "Circular load_order reference: mod '{}' specified in both before and after",
                    before_id
                ));
            }
        }

        Ok(())
    }

    pub fn uninstall(&self, mod_id: &str) -> Result<()> {
        let mod_dir = self.mods_dir().join(mod_id);
        if !mod_dir.exists() {
            return Err(anyhow::anyhow!("Mod not found: {}", mod_id));
        }

        fs::remove_dir_all(&mod_dir)?;
        Ok(())
    }

    /// Validate dependencies for a single manifest against all installed mods
    pub fn validate_dependencies(&self, manifest: &ModManifest) -> Result<()> {
        let installed_mods = self.discover()?;
        self.validate_dependencies_against(manifest, &installed_mods)
    }

    /// Validate dependencies for a single manifest against a specific set of mods
    /// This is used during loading to validate against the set of mods being loaded
    pub fn validate_dependencies_against(
        &self,
        manifest: &ModManifest,
        available_mods: &[InstalledMod],
    ) -> Result<()> {
        let available_map: HashMap<_, _> = available_mods
            .iter()
            .map(|m| (m.manifest.id.clone(), m))
            .collect();

        for dep in &manifest.dependencies {
            if let Some(installed) = available_map.get(&dep.id) {
                let installed_version = semver::Version::parse(&installed.manifest.version)?;
                if !dep.version.matches(&installed_version) {
                    return Err(anyhow::anyhow!(
                        "Dependency {} version {} does not satisfy requirement {}",
                        dep.id,
                        installed.manifest.version,
                        dep.version
                    ));
                }
            } else if dep.optional {
                // Optional dependency not found - warn but don't fail
                log::warn!("Optional dependency '{}' not found for mod '{}'", dep.id, manifest.id);
            } else {
                return Err(anyhow::anyhow!(
                    "Required dependency {} not found",
                    dep.id
                ));
            }
        }

        Ok(())
    }

    /// Validate dependencies for a complete set of mods that will be loaded together
    /// Checks that all required dependencies are satisfied within the set
    /// Returns warnings for optional dependencies that are missing
    pub fn validate_mod_set(&self, mods: &[InstalledMod]) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Build a map of all mods being loaded
        let mod_map: HashMap<_, _> = mods
            .iter()
            .map(|m| (m.manifest.id.clone(), m))
            .collect();

        // Validate each mod's dependencies against the set
        for installed_mod in mods {
            for dep in &installed_mod.manifest.dependencies {
                if let Some(dep_mod) = mod_map.get(&dep.id) {
                    let installed_version = semver::Version::parse(&dep_mod.manifest.version)?;
                    if !dep.version.matches(&installed_version) {
                        return Err(anyhow::anyhow!(
                            "Mod '{}' requires dependency '{}' version {} but found version {}",
                            installed_mod.manifest.id,
                            dep.id,
                            dep.version,
                            dep_mod.manifest.version
                        ));
                    }
                } else if dep.optional {
                    warnings.push(format!(
                        "Mod '{}' has optional dependency '{}' which is not loaded",
                        installed_mod.manifest.id, dep.id
                    ));
                } else {
                    return Err(anyhow::anyhow!(
                        "Mod '{}' requires dependency '{}' which is not loaded",
                        installed_mod.manifest.id, dep.id
                    ));
                }
            }
        }

        // Also validate game version constraints
        for installed_mod in mods {
            if let Err(e) = installed_mod.manifest.satisfies_game_version(&self.game_id) {
                warnings.push(format!(
                    "Mod '{}' game version check failed: {}",
                    installed_mod.manifest.id, e
                ));
            }
        }

        Ok(warnings)
    }

    fn extract_archive(&self, archive_path: &Path, dest: &Path) -> Result<()> {
        let extension = archive_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "zip" => self.extract_zip(archive_path, dest),
            "gz" | "tgz" => self.extract_tar_gz(archive_path, dest),
            "tar" => self.extract_tar(archive_path, dest),
            _ => {
                // Try to detect by content
                let file = fs::File::open(archive_path)?;
                let mut archive = ZipArchive::new(file)?;
                if archive.len() > 0 {
                    return self.extract_zip(archive_path, dest);
                }

                let file = fs::File::open(archive_path)?;
                let decoder = GzDecoder::new(file);
                let mut archive = Archive::new(decoder);
                if archive.entries()?.next().is_some() {
                    return self.extract_tar_gz(archive_path, dest);
                }

                Err(anyhow::anyhow!("Unknown archive format: {}", archive_path.display()))
            }
        }
    }

    fn extract_zip(&self, archive_path: &Path, dest: &Path) -> Result<()> {
        let file = fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = dest.join(file.mangled_name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            // Preserve permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }

        Ok(())
    }

    fn extract_tar_gz(&self, archive_path: &Path, dest: &Path) -> Result<()> {
        let file = fs::File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        archive.unpack(dest)?;
        Ok(())
    }

    fn extract_tar(&self, archive_path: &Path, dest: &Path) -> Result<()> {
        let file = fs::File::open(archive_path)?;
        let mut archive = Archive::new(file);
        archive.unpack(dest)?;
        Ok(())
    }

    fn find_manifest(&self, dir: &Path) -> Result<PathBuf> {
        let manifest_toml = dir.join("mod.toml");
        if manifest_toml.exists() {
            return Ok(manifest_toml);
        }

        let manifest_json = dir.join("mod.json");
        if manifest_json.exists() {
            return Ok(manifest_json);
        }

        Err(anyhow::anyhow!("No manifest found in {}", dir.display()))
    }

    pub fn compute_archive_checksum(&self, archive_path: &Path) -> Result<String> {
        let content = fs::read(archive_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn get_mod_info(&self, mod_id: &str) -> Result<InstalledMod> {
        let mod_dir = self.mods_dir().join(mod_id);
        InstalledMod::load(&mod_dir)
    }

    pub fn enable_mod(&self, mod_id: &str) -> Result<()> {
        let mod_dir = self.mods_dir().join(mod_id);
        let mut installed = InstalledMod::load(&mod_dir)?;
        installed.enabled = true;
        self.save_installed_mod(&installed)?;
        Ok(())
    }

    pub fn disable_mod(&self, mod_id: &str) -> Result<()> {
        let mod_dir = self.mods_dir().join(mod_id);
        let mut installed = InstalledMod::load(&mod_dir)?;
        installed.enabled = false;
        self.save_installed_mod(&installed)?;
        Ok(())
    }

    /// Get the permission set for a mod
    pub fn get_mod_permissions(&self, mod_id: &str) -> Result<PermissionSet> {
        let installed = self.get_mod_info(mod_id)?;
        Ok(installed.permissions)
    }

    /// Grant a permission to a mod
    pub fn grant_permission(&self, mod_id: &str, permission: Permission) -> Result<()> {
        let mod_dir = self.mods_dir().join(mod_id);
        let mut installed = InstalledMod::load(&mod_dir)?;
        installed.permissions.grant(permission);
        self.save_installed_mod(&installed)?;
        Ok(())
    }

    /// Deny a permission to a mod
    pub fn deny_permission(&self, mod_id: &str, permission: Permission) -> Result<()> {
        let mod_dir = self.mods_dir().join(mod_id);
        let mut installed = InstalledMod::load(&mod_dir)?;
        installed.permissions.deny(permission);
        self.save_installed_mod(&installed)?;
        Ok(())
    }

    /// Check if a mod has a specific permission granted
    pub fn check_permission(&self, mod_id: &str, permission: &Permission) -> Result<bool> {
        let installed = self.get_mod_info(mod_id)?;
        Ok(installed.permissions.is_granted(permission))
    }

    fn save_installed_mod(&self, installed: &InstalledMod) -> Result<()> {
        // Save config if present
        if let Some(config) = &installed.config {
            let config_path = installed.path.join("config.json");
            let content = serde_json::to_string_pretty(config)?;
            fs::write(config_path, content)?;
        }
        Ok(())
    }
}

pub fn compute_directory_checksum(dir: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(dir)
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
        let rel_path = file.strip_prefix(dir).unwrap_or(&file);
        hasher.update(rel_path.to_string_lossy().as_bytes());
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{GameTarget, ModManifest};
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_mod_discovery_creation() {
        let dir = tempdir().unwrap();
        let discovery = ModDiscovery::new(dir.path().to_path_buf(), "Alice".to_string());
        assert_eq!(discovery.game_id, "Alice");
    }

    #[test]
    fn test_discover_empty() {
        let dir = tempdir().unwrap();
        let discovery = ModDiscovery::new(dir.path().to_path_buf(), "Alice".to_string());
        let mods = discovery.discover().unwrap();
        assert!(mods.is_empty());
    }

    #[test]
    fn test_install_from_zip() {
        let dir = tempdir().unwrap();
        let discovery = ModDiscovery::new(dir.path().to_path_buf(), "Alice".to_string());

        // Create a test mod archive
        let mod_dir = tempdir().unwrap();
        let mod_path = mod_dir.path();

        fs::write(
            mod_path.join("mod.toml"),
            r#"
            id = "test_mod"
            name = "Test Mod"
            version = "1.0.0"
            manifest_version = 2
            game_target = "Alice"
            "#,
        ).unwrap();

        fs::create_dir_all(mod_path.join("assets")).unwrap();
        fs::write(mod_path.join("assets").join("test.txt"), "test content").unwrap();

        // Create zip
        let zip_path = dir.path().join("test_mod.zip");
        let zip_file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(zip_file);
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for entry in WalkDir::new(mod_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let rel_path = path.strip_prefix(mod_path).unwrap();
            let name = rel_path.to_string_lossy().to_string();

            if path.is_dir() {
                zip.add_directory(name, options).unwrap();
            } else {
                zip.start_file(name, options).unwrap();
                let content = fs::read(path).unwrap();
                zip.write_all(&content).unwrap();
            }
        }
        zip.finish().unwrap();

        // Install
        let installed = discovery.install_from_archive(&zip_path).unwrap();
        assert_eq!(installed.manifest.id, "test_mod");
        assert_eq!(installed.manifest.version, "1.0.0");
        assert!(installed.enabled);
        assert!(installed.path.exists());
    }

    #[test]
    fn test_dependency_validation() {
        let dir = tempdir().unwrap();
        let discovery = ModDiscovery::new(dir.path().to_path_buf(), "Alice".to_string());

        // Create base mod
        let base_dir = discovery.mods_dir().join("base_mod");
        fs::create_dir_all(&base_dir).unwrap();
        fs::write(
            base_dir.join("mod.toml"),
            r#"
            id = "base_mod"
            name = "Base Mod"
            version = "1.0.0"
            manifest_version = 2
            game_target = "Alice"
            "#,
        ).unwrap();

        // Create dependent mod manifest
        let manifest = ModManifest {
            id: "dependent".to_string(),
            name: "Dependent Mod".to_string(),
            version: "1.0.0".to_string(),
            manifest_version: 2,
            game_target: GameTarget::Alice,
            dependencies: vec![crate::manifest::ModDependency {
                id: "base_mod".to_string(),
                version: semver::VersionReq::parse(">=1.0.0").unwrap(),
                optional: false,
            }],
            ..Default::default()
        };

        assert!(discovery.validate_dependencies(&manifest).is_ok());

        // Test missing dependency
        let manifest_missing = ModManifest {
            id: "dependent".to_string(),
            name: "Dependent Mod".to_string(),
            version: "1.0.0".to_string(),
            manifest_version: 2,
            game_target: GameTarget::Alice,
            dependencies: vec![crate::manifest::ModDependency {
                id: "missing_mod".to_string(),
                version: semver::VersionReq::parse(">=1.0.0").unwrap(),
                optional: false,
            }],
            ..Default::default()
        };

        assert!(discovery.validate_dependencies(&manifest_missing).is_err());

        // Test version mismatch
        let manifest_wrong_version = ModManifest {
            id: "dependent".to_string(),
            name: "Dependent Mod".to_string(),
            version: "1.0.0".to_string(),
            manifest_version: 2,
            game_target: GameTarget::Alice,
            dependencies: vec![crate::manifest::ModDependency {
                id: "base_mod".to_string(),
                version: semver::VersionReq::parse(">=2.0.0").unwrap(),
                optional: false,
            }],
            ..Default::default()
        };

        assert!(discovery.validate_dependencies(&manifest_wrong_version).is_err());
    }

    #[test]
    fn test_checksum_computation() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("file1.txt"), "content1").unwrap();
        fs::create_dir_all(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("subdir").join("file2.txt"), "content2").unwrap();

        let checksum1 = compute_directory_checksum(dir.path()).unwrap();
        let checksum2 = compute_directory_checksum(dir.path()).unwrap();
        assert_eq!(checksum1, checksum2);

        // Modify a file
        fs::write(dir.path().join("file1.txt"), "modified").unwrap();
        let checksum3 = compute_directory_checksum(dir.path()).unwrap();
        assert_ne!(checksum1, checksum3);
    }
}