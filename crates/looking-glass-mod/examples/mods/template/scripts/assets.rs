//! Template Mod - Asset Override Script
//! This is a placeholder for asset overrides. Copy and modify for your mod.

use looking_glass_mod::{AssetType, ModApi, ModContext, Result};

/// Register asset overrides for this mod
pub fn register_asset_overrides(api: &ModApi, mod_dir: &std::path::Path) {
    // Example: Override a texture
    // let source = mod_dir.join("assets/textures/my_custom_texture.png");
    // if source.exists() {
    //     api.register_asset_override(
    //         "textures/original_texture.png".to_string(),
    //         source,
    //         AssetType::Texture,
    //         None, // No condition
    //         50,   // Priority
    //     );
    // }

    // Example: Override with condition (e.g., only for specific platform)
    // api.register_asset_override(
    //     "textures/ui/button_a.png".to_string(),
    //     mod_dir.join("assets/textures/ui/button_a_ps.png"),
    //     AssetType::Texture,
    //     Some("platform == \"ps4\"".to_string()),
    //     100,
    // );
}

/// Entry point called by the mod loader
#[no_mangle]
pub extern "C" fn register_asset_overrides_entry(api: &ModApi, context: &ModContext) {
    register_asset_overrides(api, &context.mod_dir);
}