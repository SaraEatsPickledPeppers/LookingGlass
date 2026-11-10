//! looking-glass-mod - Core mod system types and functionality for Looking Glass

pub mod config;
pub mod discovery;
pub mod loader;
pub mod manifest;
pub mod migration;
pub mod mod_loaders;
pub mod profiling;

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
pub use {
    profiling::is_profiling_enabled,
    profiling::set_profiling_enabled,
    profiling::start_profiling_session,
    profiling::reset_profiling,
    profiling::get_mod_profiles,
    profiling::get_mod_profile,
    profiling::get_global_summary,
    profiling::GlobalProfileSummary,
    profiling::ModProfile,
    profiling::ProfilingGuard,
    profiling::Timer,
    profiling::record_ui_patch,
};
// Macros are exported at crate root via #[macro_export] in profiling.rs
// profile_mod_load, profile_mod_init, profile_asset_override, profile_input_hook, profile_runtime_hook, profile_frame