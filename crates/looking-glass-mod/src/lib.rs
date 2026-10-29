//! looking-glass-mod - Core mod system types and functionality for Looking Glass

pub mod config;
pub mod discovery;
pub mod loader;
pub mod manifest;
pub mod migration;

pub use config::{ConfigManager, GlobalConfig, GameConfig, Settings, LayeredConfig, ModConfig, XdgPaths, resolve_xdg_paths};
pub use discovery::{ModDiscovery, compute_directory_checksum};
pub use loader::{
    ModRegistry, ModContext, ModApi, AssetRegistry, MenuRegistry, UiRegistry, ConfigRegistry,
    AssetOverrideEntry, MenuEntry, MenuCallback, UiPatch, UiPatchOp, WidgetDef,
    InputHook, KeyCode, KeyModifiers, GamepadButton, ButtonState, ButtonOption,
    AssetOverride, AssetOverrideResult, ConfigValidator, RuntimeHook, Mod, ModLoader,
};
pub use manifest::{
    GameTarget, ModDependency, ModEntryPoints, AssetType, AssetMapping,
    LoadOrder, ModManifest, InstalledMod,
};
pub use migration::{migrate_manifest_v1_to_v2, migrate_manifest_v1_to_v2_from_value};