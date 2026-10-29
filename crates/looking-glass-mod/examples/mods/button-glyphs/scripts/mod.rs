//! Button Glyphs Mod - Main Mod Implementation
//! This mod replaces controller button prompts with proper glyphs (PS4/Xbox/Switch icons)

use looking_glass_mod::{
    ModApi, ModContext, ModManifest, ModEntryPoints, AssetType,
    Result, async_trait,
};

/// Main mod struct for Button Glyphs
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

    /// Register all platform-specific button glyph overrides
    fn register_overrides(&self, api: &ModApi, mod_dir: &std::path::Path) {
        let platforms = ["ps", "xbox", "switch"];
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
                    let condition = Some(format!("platform == \"{}\"",
                        if *platform == "ps" { "ps4" } else { platform }));
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

#[async_trait]
impl looking_glass_mod::Mod for ButtonGlyphsMod {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        // Register asset overrides for button glyphs
        self.register_overrides(&context.mod_api, &context.mod_dir);

        context.mod_api.log(
            log::Level::Info,
            "Button Glyphs mod initialized - platform-specific button glyphs registered",
        );
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // No cleanup needed for asset overrides
        Ok(())
    }

    async fn on_config_changed(&mut self, _key: &str, _value: &serde_json::Value) -> Result<()> {
        // Button glyphs mod doesn't have runtime config
        Ok(())
    }

    fn manifest(&self) -> &ModManifest {
        &self.manifest
    }

    fn entry_points(&self) -> &ModEntryPoints {
        &self.entry_points
    }
}

/// Module entry point - called by the mod loader
#[no_mangle]
pub extern "C" fn create_mod(mod_dir: &std::path::Path) -> Result<Box<dyn looking_glass_mod::Mod>> {
    Ok(Box::new(ButtonGlyphsMod::new(mod_dir)?))
}