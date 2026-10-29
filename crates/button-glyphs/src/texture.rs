use crate::glyphs::GlyphStyle;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Manages loading and caching of glyph textures
pub struct GlyphTextureManager {
    textures: HashMap<String, Texture2D>,
    asset_dir: PathBuf,
    fallback_glyphs: HashMap<String, String>,
    scale: f32,
    color_tint: Option<Color>,
}

impl GlyphTextureManager {
    /// Create a new texture manager
    pub fn new(mod_dir: &Path, style: GlyphStyle, scale: f32, color_tint: Option<[f32; 4]>) -> Self {
        let asset_dir = mod_dir.join("assets").join("glyphs").join(style.asset_dir());
        let color = color_tint.map(|c| Color::new(c[0], c[1], c[2], c[3]));

        let mut manager = Self {
            textures: HashMap::new(),
            asset_dir,
            fallback_glyphs: Self::builtin_fallback_glyphs(style),
            scale,
            color_tint: color,
        };

        // Pre-load built-in procedural glyphs as fallbacks
        manager.load_builtin_fallbacks();

        manager
    }

    /// Get built-in procedural glyph text representations for each style
    fn builtin_fallback_glyphs(style: GlyphStyle) -> HashMap<String, String> {
        let mut map = HashMap::new();
        match style {
            GlyphStyle::PlayStation => {
                map.insert("South".to_string(), "✕".to_string());
                map.insert("East".to_string(), "◯".to_string());
                map.insert("West".to_string(), "□".to_string());
                map.insert("North".to_string(), "△".to_string());
                map.insert("L1".to_string(), "L1".to_string());
                map.insert("L2".to_string(), "L2".to_string());
                map.insert("R1".to_string(), "R1".to_string());
                map.insert("R2".to_string(), "R2".to_string());
                map.insert("L3".to_string(), "L3".to_string());
                map.insert("R3".to_string(), "R3".to_string());
                map.insert("DPadUp".to_string(), "↑".to_string());
                map.insert("DPadDown".to_string(), "↓".to_string());
                map.insert("DPadLeft".to_string(), "←".to_string());
                map.insert("DPadRight".to_string(), "→".to_string());
                map.insert("Start".to_string(), "Options".to_string());
                map.insert("Select".to_string(), "Create".to_string());
                map.insert("Home".to_string(), "PS".to_string());
                map.insert("Touchpad".to_string(), "Touchpad".to_string());
            }
            GlyphStyle::Xbox => {
                map.insert("South".to_string(), "A".to_string());
                map.insert("East".to_string(), "B".to_string());
                map.insert("West".to_string(), "X".to_string());
                map.insert("North".to_string(), "Y".to_string());
                map.insert("L1".to_string(), "LB".to_string());
                map.insert("L2".to_string(), "LT".to_string());
                map.insert("R1".to_string(), "RB".to_string());
                map.insert("R2".to_string(), "RT".to_string());
                map.insert("L3".to_string(), "LS".to_string());
                map.insert("R3".to_string(), "RS".to_string());
                map.insert("DPadUp".to_string(), "↑".to_string());
                map.insert("DPadDown".to_string(), "↓".to_string());
                map.insert("DPadLeft".to_string(), "←".to_string());
                map.insert("DPadRight".to_string(), "→".to_string());
                map.insert("Start".to_string(), "Menu".to_string());
                map.insert("Select".to_string(), "View".to_string());
                map.insert("Home".to_string(), "Xbox".to_string());
                map.insert("Touchpad".to_string(), "Share".to_string());
            }
            GlyphStyle::Switch => {
                map.insert("South".to_string(), "B".to_string());
                map.insert("East".to_string(), "A".to_string());
                map.insert("West".to_string(), "Y".to_string());
                map.insert("North".to_string(), "X".to_string());
                map.insert("L1".to_string(), "L".to_string());
                map.insert("L2".to_string(), "ZL".to_string());
                map.insert("R1".to_string(), "R".to_string());
                map.insert("R2".to_string(), "ZR".to_string());
                map.insert("L3".to_string(), "LS".to_string());
                map.insert("R3".to_string(), "RS".to_string());
                map.insert("DPadUp".to_string(), "↑".to_string());
                map.insert("DPadDown".to_string(), "↓".to_string());
                map.insert("DPadLeft".to_string(), "←".to_string());
                map.insert("DPadRight".to_string(), "→".to_string());
                map.insert("Start".to_string(), "+".to_string());
                map.insert("Select".to_string(), "-".to_string());
                map.insert("Home".to_string(), "Home".to_string());
                map.insert("Touchpad".to_string(), "Capture".to_string());
            }
            GlyphStyle::Generic => {
                map.insert("South".to_string(), "[A]".to_string());
                map.insert("East".to_string(), "[B]".to_string());
                map.insert("West".to_string(), "[X]".to_string());
                map.insert("North".to_string(), "[Y]".to_string());
                map.insert("L1".to_string(), "[L1]".to_string());
                map.insert("L2".to_string(), "[L2]".to_string());
                map.insert("R1".to_string(), "[R1]".to_string());
                map.insert("R2".to_string(), "[R2]".to_string());
                map.insert("L3".to_string(), "[L3]".to_string());
                map.insert("R3".to_string(), "[R3]".to_string());
                map.insert("DPadUp".to_string(), "[↑]".to_string());
                map.insert("DPadDown".to_string(), "[↓]".to_string());
                map.insert("DPadLeft".to_string(), "[←]".to_string());
                map.insert("DPadRight".to_string(), "[→]".to_string());
                map.insert("Start".to_string(), "[Start]".to_string());
                map.insert("Select".to_string(), "[Select]".to_string());
                map.insert("Home".to_string(), "[Home]".to_string());
                map.insert("Touchpad".to_string(), "[Touchpad]".to_string());
            }
        }
        map
    }

    /// Load built-in fallback glyphs as simple text-based textures
    fn load_builtin_fallbacks(&mut self) {
        // For now, we don't create actual textures for built-in glyphs
        // They'll be rendered as text when no texture is available
    }

    /// Load a texture for a specific glyph from the assets directory
    pub async fn load_glyph(&mut self, button: &str) -> Option<Texture2D> {
        // Check if already cached
        if let Some(texture) = self.textures.get(button) {
            return Some(*texture);
        }

        // Try to load from assets directory
        let extensions = ["png", "svg", "jpg", "jpeg", "tga", "bmp"];
        for ext in &extensions {
            let path = self.asset_dir.join(format!("{}.{}", button.to_lowercase(), ext));
            if path.exists() {
                match load_texture(path.to_str().unwrap()).await {
                    Ok(texture) => {
                        texture.set_filter(FilterMode::Linear);
                        self.textures.insert(button.to_string(), texture);
                        return Some(texture);
                    }
                    Err(e) => {
                        log::warn!("Failed to load glyph texture {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Try common alternative names
        let alternatives = self.get_alternative_names(button);
        for alt in alternatives {
            for ext in &extensions {
                let path = self.asset_dir.join(format!("{}.{}", alt.to_lowercase(), ext));
                if path.exists() {
                    match load_texture(path.to_str().unwrap()).await {
                        Ok(texture) => {
                            texture.set_filter(FilterMode::Linear);
                            self.textures.insert(button.to_string(), texture);
                            return Some(texture);
                        }
                        Err(e) => {
                            log::warn!("Failed to load glyph texture {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        None
    }

    /// Get alternative names for a button (e.g., "Cross" -> "South", "A")
    fn get_alternative_names(&self, button: &str) -> Vec<String> {
        let mut alternatives = Vec::new();

        match button {
            "South" => alternatives.extend(["Cross", "A", "FaceSouth"]),
            "East" => alternatives.extend(["Circle", "B", "FaceEast"]),
            "West" => alternatives.extend(["Square", "X", "FaceWest"]),
            "North" => alternatives.extend(["Triangle", "Y", "FaceNorth"]),
            "L1" => alternatives.extend(["LB", "L", "LeftShoulder", "LeftBumper"]),
            "L2" => alternatives.extend(["LT", "ZL", "LeftTrigger"]),
            "R1" => alternatives.extend(["RB", "R", "RightShoulder", "RightBumper"]),
            "R2" => alternatives.extend(["RT", "ZR", "RightTrigger"]),
            "L3" => alternatives.extend(["LS", "LeftStick", "LeftThumb"]),
            "R3" => alternatives.extend(["RS", "RightStick", "RightThumb"]),
            "DPadUp" => alternatives.extend(["Up", "DUp"]),
            "DPadDown" => alternatives.extend(["Down", "DDown"]),
            "DPadLeft" => alternatives.extend(["Left", "DLeft"]),
            "DPadRight" => alternatives.extend(["Right", "DRight"]),
            "Start" => alternatives.extend(["Options", "Menu", "Plus", "StartButton"]),
            "Select" => alternatives.extend(["Create", "View", "Minus", "SelectButton", "Back"]),
            "Home" => alternatives.extend(["PS", "Xbox", "Guide", "HomeButton"]),
            "Touchpad" => alternatives.extend(["Share", "Capture", "TouchPad"]),
            _ => {}
        }

        alternatives.into_iter().map(|s| s.to_string()).collect()
    }

    /// Get a texture for a glyph, loading it if necessary
    pub async fn get_or_load(&mut self, button: &str) -> Option<Texture2D> {
        self.load_glyph(button).await
    }

    /// Draw a glyph texture at the specified position
    pub fn draw_glyph(&self, button: &str, x: f32, y: f32, scale: Option<f32>) {
        let scale = scale.unwrap_or(self.scale);

        if let Some(texture) = self.textures.get(button) {
            let color = self.color_tint.unwrap_or(WHITE);
            draw_texture_ex(
                *texture,
                x,
                y,
                color,
                DrawTextureParams {
                    dest_size: Some(vec2(texture.width() * scale, texture.height() * scale)),
                    ..Default::default()
                },
            );
        } else if let Some(text) = self.fallback_glyphs.get(button) {
            // Draw fallback text
            let color = self.color_tint.unwrap_or(WHITE);
            let font_size = (16.0 * scale) as u16;
            draw_text(text, x, y + font_size as f32, font_size as f32, color);
        }
    }

    /// Get the size of a glyph texture (or estimated size for fallback)
    pub fn glyph_size(&self, button: &str) -> Vec2 {
        if let Some(texture) = self.textures.get(button) {
            vec2(texture.width() * self.scale, texture.height() * self.scale)
        } else if let Some(text) = self.fallback_glyphs.get(button) {
            let font_size = (16.0 * self.scale) as f32;
            // Approximate text width
            vec2(text.len() as f32 * font_size * 0.6, font_size)
        } else {
            vec2(16.0 * self.scale, 16.0 * self.scale)
        }
    }

    /// Set the scale for all glyphs
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.1);
    }

    /// Set the color tint for all glyphs
    pub fn set_color_tint(&mut self, color: Option<[f32; 4]>) {
        self.color_tint = color.map(|c| Color::new(c[0], c[1], c[2], c[3]));
    }

    /// Reload all textures (useful after config change)
    pub async fn reload_all(&mut self) {
        self.textures.clear();
        self.load_builtin_fallbacks();
    }

    /// Check if a glyph texture exists in assets
    pub fn has_asset(&self, button: &str) -> bool {
        let extensions = ["png", "svg", "jpg", "jpeg", "tga", "bmp"];
        for ext in &extensions {
            let path = self.asset_dir.join(format!("{}.{}", button.to_lowercase(), ext));
            if path.exists() {
                return true;
            }
        }

        // Check alternatives
        for alt in self.get_alternative_names(button) {
            for ext in &extensions {
                let path = self.asset_dir.join(format!("{}.{}", alt.to_lowercase(), ext));
                if path.exists() {
                    return true;
                }
            }
        }

        false
    }
}

impl Default for GlyphTextureManager {
    fn default() -> Self {
        Self::new(Path::new("."), GlyphStyle::default(), 1.0, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_texture_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = GlyphTextureManager::new(dir.path(), GlyphStyle::PlayStation, 1.0, None);
        assert_eq!(manager.scale, 1.0);
        assert_eq!(manager.color_tint, None);
    }

    #[test]
    fn test_builtin_fallbacks_playstation() {
        let dir = tempdir().unwrap();
        let manager = GlyphTextureManager::new(dir.path(), GlyphStyle::PlayStation, 1.0, None);

        assert_eq!(manager.fallback_glyphs.get("South"), Some(&"✕".to_string()));
        assert_eq!(manager.fallback_glyphs.get("East"), Some(&"◯".to_string()));
        assert_eq!(manager.fallback_glyphs.get("West"), Some(&"□".to_string()));
        assert_eq!(manager.fallback_glyphs.get("North"), Some(&"△".to_string()));
    }

    #[test]
    fn test_builtin_fallbacks_xbox() {
        let dir = tempdir().unwrap();
        let manager = GlyphTextureManager::new(dir.path(), GlyphStyle::Xbox, 1.0, None);

        assert_eq!(manager.fallback_glyphs.get("South"), Some(&"A".to_string()));
        assert_eq!(manager.fallback_glyphs.get("East"), Some(&"B".to_string()));
        assert_eq!(manager.fallback_glyphs.get("West"), Some(&"X".to_string()));
        assert_eq!(manager.fallback_glyphs.get("North"), Some(&"Y".to_string()));
    }

    #[test]
    fn test_builtin_fallbacks_switch() {
        let dir = tempdir().unwrap();
        let manager = GlyphTextureManager::new(dir.path(), GlyphStyle::Switch, 1.0, None);

        // Switch has A/B swapped
        assert_eq!(manager.fallback_glyphs.get("South"), Some(&"B".to_string()));
        assert_eq!(manager.fallback_glyphs.get("East"), Some(&"A".to_string()));
    }

    #[test]
    fn test_builtin_fallbacks_generic() {
        let dir = tempdir().unwrap();
        let manager = GlyphTextureManager::new(dir.path(), GlyphStyle::Generic, 1.0, None);

        assert_eq!(manager.fallback_glyphs.get("South"), Some(&"[A]".to_string()));
        assert_eq!(manager.fallback_glyphs.get("East"), Some(&"[B]".to_string()));
    }

    #[test]
    fn test_alternative_names() {
        let dir = tempdir().unwrap();
        let manager = GlyphTextureManager::new(dir.path(), GlyphStyle::PlayStation, 1.0, None);

        let alts = manager.get_alternative_names("South");
        assert!(alts.contains(&"Cross".to_string()));
        assert!(alts.contains(&"A".to_string()));

        let alts = manager.get_alternative_names("L1");
        assert!(alts.contains(&"LB".to_string()));
        assert!(alts.contains(&"L".to_string()));
    }

    #[test]
    fn test_glyph_size_fallback() {
        let dir = tempdir().unwrap();
        let manager = GlyphTextureManager::new(dir.path(), GlyphStyle::PlayStation, 2.0, None);

        let size = manager.glyph_size("South");
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
    }
}