use crate::loader::{Mod, ModContext, ModLoader};
use crate::manifest::{ModManifest, ModEntryPoints, GameTarget, PermissionSet};
use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use std::path::PathBuf;
use std::sync::Arc;

pub struct NativeModLoader;

impl NativeModLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn find_library(mod_dir: &PathBuf) -> Option<PathBuf> {
        let candidates = [
            mod_dir.join("libmod.so"),
            mod_dir.join("libmod.dylib"),
            mod_dir.join("mod.dll"),
            mod_dir.join("mod.so"),
            mod_dir.join("mod.dylib"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Some(candidate.clone());
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl ModLoader for NativeModLoader {
    async fn load_mod_with_permissions(
        &self,
        mod_dir: &PathBuf,
        _permissions: &PermissionSet,
    ) -> Result<Box<dyn Mod + Send + Sync + 'static>> {
        // For native mods, we load the library and use permissions from manifest
        // The permissions parameter is provided by the user, but native mods
        // typically declare their permissions in the manifest
        self.load_mod(mod_dir).await
    }

    async fn load_mod(&self, mod_dir: &PathBuf) -> Result<Box<dyn Mod + Send + Sync + 'static>> {
        let library_path = Self::find_library(mod_dir)
            .context("No native library found in mod directory")?;

        let library = unsafe { Library::new(&library_path) }
            .with_context(|| format!("Failed to load library: {}", library_path.display()))?;

        // Get the mod manifest from the library
        let get_manifest: Symbol<unsafe extern "C" fn() -> *const ModManifest> = unsafe {
            library.get(b"get_manifest\0")?
        };
        let manifest_ptr = unsafe { get_manifest() };
        let manifest = unsafe { &*manifest_ptr }.clone();

        // Create the mod instance
        let create_mod: Symbol<unsafe extern "C" fn() -> *mut dyn Mod> = unsafe {
            library.get(b"create_mod\0")?
        };
        let mod_ptr = unsafe { create_mod() };
        let mod_instance = unsafe { Box::from_raw(mod_ptr) };

        // Store the library to keep it alive
        let native_mod = NativeMod {
            manifest,
            library: Arc::new(library),
            inner: mod_instance,
        };

        Ok(Box::new(native_mod))
    }

    async fn unload_mod(&self, _mod_id: &str) -> Result<()> {
        // The library will be dropped when NativeMod is dropped
        Ok(())
    }

    fn supported_game_targets(&self) -> Vec<GameTarget> {
        vec![GameTarget::Alice]
    }
}

struct NativeMod {
    manifest: ModManifest,
    library: Arc<Library>,
    inner: Box<dyn Mod>,
}

impl std::ops::Deref for NativeMod {
    type Target = dyn Mod;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl std::ops::DerefMut for NativeMod {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

#[async_trait::async_trait]
impl Mod for NativeMod {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        self.inner.initialize(context).await
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown().await
    }

    async fn on_config_changed(&mut self, key: &str, value: &serde_json::Value) -> Result<()> {
        self.inner.on_config_changed(key, value).await
    }

    fn manifest(&self) -> &ModManifest {
        &self.manifest
    }

    fn entry_points(&self) -> &ModEntryPoints {
        self.inner.entry_points()
    }
}