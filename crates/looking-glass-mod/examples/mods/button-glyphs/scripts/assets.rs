//! Button Glyphs Mod - Asset Override Script
//! This script registers platform-specific button glyph overrides.

use looking_glass_mod::{
    AssetType, ModApi, ModContext, ModManifest, ModEntryPoints,
    Result,
};

/// Platform-specific button glyph asset override module
pub struct ButtonGlyphsAssetOverride;

impl ButtonGlyphsAssetOverride {
    /// Register all platform-specific button glyph overrides
    pub fn register_overrides(api: &ModApi, mod_dir: &std::path::Path) {
        let platforms = ["ps4", "xbox", "switch"];
        let buttons = [
            ("south", "button_south"),
            ("east", "button_east"),
            ("west", "button_west"),
            ("north", "button_north"),
            ("l1", "button_l1"),
            ("r1", "button_r1"),
            ("l2", "button_l2"),
            ("r2", "button_r2"),
            ("select", "button_select"),
            ("start", "button_start"),
            ("l3", "button_l3"),
            ("r3", "button_r3"),
            ("dpad", "button_dpad"),
        ];

        for platform in &platforms {
            for (button_name, target_name) in &buttons {
                let source_path = mod_dir.join(format!(
                    "assets/textures/ui/button_{}_{}.png",
                    button_name, platform
                ));
                let target_path = format!("textures/ui/{}.png", target_name);

                if source_path.exists() {
                    let condition = Some(format!("platform == \"{}\"", platform));
                    api.register_asset_override(
                        target_path,
                        source_path,
                        AssetType::Texture,
                        condition,
                        100, // High priority for platform-specific overrides
                    );
                }
            }
        }
    }
}

/// Entry point for the asset override system
#[no_mangle]
pub extern "C" fn register_asset_overrides(api: &ModApi, context: &ModContext) {
    ButtonGlyphsAssetOverride::register_overrides(api, &context.mod_dir);
}

/// Mod implementation for the Button Glyphs mod
pub struct ButtonGlyphsMod {
    manifest: ModManifest,
    entry_points: ModEntryPoints,
}

impl ButtonGlyphsMod {
    pub fn new(mod_dir: &std::path::Path) -> Result<Self> {
        let manifest_path = mod_dir.join("mod.toml");
        let manifest = ModManifest::load_from_file(&manifest_path)?;
        let entry_points = manifest.entry_points.clone();
        Ok(Self { manifest, entry_points })
    }
}

#[async_trait::async_trait]
impl looking_glass_mod::Mod for ButtonGlyphsMod {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        // Register asset overrides for button glyphs
        ButtonGlyphsAssetOverride::register_overrides(&context.mod_api, &context.mod_dir);
        context.mod_api.log(
            log::Level::Info,
            "Button Glyphs mod initialized - platform-specific button glyphs registered",
        );
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        context.mod_api.log(log::Level::Info, "Button Glyphs mod shutdown");
        Ok(())
    }

    async fn on_config_changed(&mut self, _key: &str, _value: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    fn manifest(&self) -> &ModManifest {
        &self.manifest
    }

    fn entry_points(&self) -> &ModEntryPoints {
        &self.entry_points
    }
}