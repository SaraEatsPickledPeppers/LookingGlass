use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Controller glyph style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum GlyphStyle {
    #[default]
    PlayStation,
    Xbox,
    Switch,
    Generic,
}

impl GlyphStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            GlyphStyle::PlayStation => "PlayStation",
            GlyphStyle::Xbox => "Xbox",
            GlyphStyle::Switch => "Switch",
            GlyphStyle::Generic => "Generic",
        }
    }

    pub fn asset_dir(&self) -> &'static str {
        match self {
            GlyphStyle::PlayStation => "playstation",
            GlyphStyle::Xbox => "xbox",
            GlyphStyle::Switch => "switch",
            GlyphStyle::Generic => "generic",
        }
    }
}

impl std::fmt::Display for GlyphStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for GlyphStyle {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PlayStation" | "playstation" | "ps" => Ok(GlyphStyle::PlayStation),
            "Xbox" | "xbox" => Ok(GlyphStyle::Xbox),
            "Switch" | "switch" => Ok(GlyphStyle::Switch),
            "Generic" | "generic" | "text" => Ok(GlyphStyle::Generic),
            _ => Err(anyhow::anyhow!("Unknown glyph style: {}", s)),
        }
    }
}

/// Complete set of glyph mappings for a controller style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphSet {
    pub style: GlyphStyle,
    pub face_south: String,   // Cross / A / B (Switch)
    pub face_east: String,    // Circle / B / A (Switch)
    pub face_west: String,    // Square / X / Y
    pub face_north: String,   // Triangle / Y / X
    pub shoulder_l1: String,  // L1 / LB / L
    pub shoulder_l2: String,  // L2 / LT / ZL
    pub shoulder_r1: String,  // R1 / RB / R
    pub shoulder_r2: String,  // R2 / RT / ZR
    pub stick_l3: String,     // L3 / LS / LS
    pub stick_r3: String,     // R3 / RS / RS
    pub dpad_up: String,
    pub dpad_down: String,
    pub dpad_left: String,
    pub dpad_right: String,
    pub start: String,        // Options / Menu / +
    pub select: String,       // Create / View / -
    pub home: String,         // PS / Xbox / Home
    pub touchpad: String,     // Touchpad / Share / Capture
    pub paddle1: String,
    pub paddle2: String,
    pub paddle3: String,
    pub paddle4: String,
}

impl GlyphSet {
    /// Create the default PlayStation glyph set
    pub fn playstation() -> Self {
        Self {
            style: GlyphStyle::PlayStation,
            face_south: "✕".to_string(),      // Cross
            face_east: "◯".to_string(),       // Circle
            face_west: "□".to_string(),       // Square
            face_north: "△".to_string(),      // Triangle
            shoulder_l1: "L1".to_string(),
            shoulder_l2: "L2".to_string(),
            shoulder_r1: "R1".to_string(),
            shoulder_r2: "R2".to_string(),
            stick_l3: "L3".to_string(),
            stick_r3: "R3".to_string(),
            dpad_up: "↑".to_string(),
            dpad_down: "↓".to_string(),
            dpad_left: "←".to_string(),
            dpad_right: "→".to_string(),
            start: "Options".to_string(),
            select: "Create".to_string(),
            home: "PS".to_string(),
            touchpad: "Touchpad".to_string(),
            paddle1: "P1".to_string(),
            paddle2: "P2".to_string(),
            paddle3: "P3".to_string(),
            paddle4: "P4".to_string(),
        }
    }

    /// Create the Xbox glyph set
    pub fn xbox() -> Self {
        Self {
            style: GlyphStyle::Xbox,
            face_south: "A".to_string(),
            face_east: "B".to_string(),
            face_west: "X".to_string(),
            face_north: "Y".to_string(),
            shoulder_l1: "LB".to_string(),
            shoulder_l2: "LT".to_string(),
            shoulder_r1: "RB".to_string(),
            shoulder_r2: "RT".to_string(),
            stick_l3: "LS".to_string(),
            stick_r3: "RS".to_string(),
            dpad_up: "↑".to_string(),
            dpad_down: "↓".to_string(),
            dpad_left: "←".to_string(),
            dpad_right: "→".to_string(),
            start: "Menu".to_string(),
            select: "View".to_string(),
            home: "Xbox".to_string(),
            touchpad: "Share".to_string(),
            paddle1: "P1".to_string(),
            paddle2: "P2".to_string(),
            paddle3: "P3".to_string(),
            paddle4: "P4".to_string(),
        }
    }

    /// Create the Switch glyph set (A/B swapped compared to Xbox)
    pub fn switch() -> Self {
        Self {
            style: GlyphStyle::Switch,
            face_south: "B".to_string(),  // Swapped: B is bottom on Switch
            face_east: "A".to_string(),   // Swapped: A is right on Switch
            face_west: "Y".to_string(),
            face_north: "X".to_string(),
            shoulder_l1: "L".to_string(),
            shoulder_l2: "ZL".to_string(),
            shoulder_r1: "R".to_string(),
            shoulder_r2: "ZR".to_string(),
            stick_l3: "LS".to_string(),
            stick_r3: "RS".to_string(),
            dpad_up: "↑".to_string(),
            dpad_down: "↓".to_string(),
            dpad_left: "←".to_string(),
            dpad_right: "→".to_string(),
            start: "+".to_string(),
            select: "-".to_string(),
            home: "Home".to_string(),
            touchpad: "Capture".to_string(),
            paddle1: "P1".to_string(),
            paddle2: "P2".to_string(),
            paddle3: "P3".to_string(),
            paddle4: "P4".to_string(),
        }
    }

    /// Create the Generic text fallback glyph set
    pub fn generic() -> Self {
        Self {
            style: GlyphStyle::Generic,
            face_south: "[A]".to_string(),
            face_east: "[B]".to_string(),
            face_west: "[X]".to_string(),
            face_north: "[Y]".to_string(),
            shoulder_l1: "[L1]".to_string(),
            shoulder_l2: "[L2]".to_string(),
            shoulder_r1: "[R1]".to_string(),
            shoulder_r2: "[R2]".to_string(),
            stick_l3: "[L3]".to_string(),
            stick_r3: "[R3]".to_string(),
            dpad_up: "[↑]".to_string(),
            dpad_down: "[↓]".to_string(),
            dpad_left: "[←]".to_string(),
            dpad_right: "[→]".to_string(),
            start: "[Start]".to_string(),
            select: "[Select]".to_string(),
            home: "[Home]".to_string(),
            touchpad: "[Touchpad]".to_string(),
            paddle1: "[P1]".to_string(),
            paddle2: "[P2]".to_string(),
            paddle3: "[P3]".to_string(),
            paddle4: "[P4]".to_string(),
        }
    }

    /// Get the default glyph set for a style
    pub fn default_for(style: GlyphStyle) -> Self {
        match style {
            GlyphStyle::PlayStation => Self::playstation(),
            GlyphStyle::Xbox => Self::xbox(),
            GlyphStyle::Switch => Self::switch(),
            GlyphStyle::Generic => Self::generic(),
        }
    }

    /// Get glyph for a button identifier
    pub fn get_glyph(&self, button: &str) -> Option<&str> {
        let map = self.to_map();
        map.get(button).map(|s| s.as_str())
    }

    /// Get all glyphs as a map for lookup
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("South".to_string(), self.face_south.clone());
        map.insert("Cross".to_string(), self.face_south.clone());
        map.insert("A".to_string(), self.face_south.clone());
        map.insert("East".to_string(), self.face_east.clone());
        map.insert("Circle".to_string(), self.face_east.clone());
        map.insert("B".to_string(), self.face_east.clone());
        map.insert("West".to_string(), self.face_west.clone());
        map.insert("Square".to_string(), self.face_west.clone());
        map.insert("X".to_string(), self.face_west.clone());
        map.insert("North".to_string(), self.face_north.clone());
        map.insert("Triangle".to_string(), self.face_north.clone());
        map.insert("Y".to_string(), self.face_north.clone());
        map.insert("L1".to_string(), self.shoulder_l1.clone());
        map.insert("LB".to_string(), self.shoulder_l1.clone());
        map.insert("L".to_string(), self.shoulder_l1.clone());
        map.insert("L2".to_string(), self.shoulder_l2.clone());
        map.insert("LT".to_string(), self.shoulder_l2.clone());
        map.insert("ZL".to_string(), self.shoulder_l2.clone());
        map.insert("R1".to_string(), self.shoulder_r1.clone());
        map.insert("RB".to_string(), self.shoulder_r1.clone());
        map.insert("R".to_string(), self.shoulder_r1.clone());
        map.insert("R2".to_string(), self.shoulder_r2.clone());
        map.insert("RT".to_string(), self.shoulder_r2.clone());
        map.insert("ZR".to_string(), self.shoulder_r2.clone());
        map.insert("L3".to_string(), self.stick_l3.clone());
        map.insert("LS".to_string(), self.stick_l3.clone());
        map.insert("R3".to_string(), self.stick_r3.clone());
        map.insert("RS".to_string(), self.stick_r3.clone());
        map.insert("DPadUp".to_string(), self.dpad_up.clone());
        map.insert("Up".to_string(), self.dpad_up.clone());
        map.insert("DPadDown".to_string(), self.dpad_down.clone());
        map.insert("Down".to_string(), self.dpad_down.clone());
        map.insert("DPadLeft".to_string(), self.dpad_left.clone());
        map.insert("Left".to_string(), self.dpad_left.clone());
        map.insert("DPadRight".to_string(), self.dpad_right.clone());
        map.insert("Right".to_string(), self.dpad_right.clone());
        map.insert("Start".to_string(), self.start.clone());
        map.insert("Options".to_string(), self.start.clone());
        map.insert("Menu".to_string(), self.start.clone());
        map.insert("Plus".to_string(), self.start.clone());
        map.insert("Select".to_string(), self.select.clone());
        map.insert("Create".to_string(), self.select.clone());
        map.insert("View".to_string(), self.select.clone());
        map.insert("Minus".to_string(), self.select.clone());
        map.insert("Home".to_string(), self.home.clone());
        map.insert("PS".to_string(), self.home.clone());
        map.insert("Xbox".to_string(), self.home.clone());
        map.insert("Touchpad".to_string(), self.touchpad.clone());
        map.insert("Share".to_string(), self.touchpad.clone());
        map.insert("Capture".to_string(), self.touchpad.clone());
        map.insert("Paddle1".to_string(), self.paddle1.clone());
        map.insert("Paddle2".to_string(), self.paddle2.clone());
        map.insert("Paddle3".to_string(), self.paddle3.clone());
        map.insert("Paddle4".to_string(), self.paddle4.clone());
        map
    }
}

impl Default for GlyphSet {
    fn default() -> Self {
        Self::playstation()
    }
}

/// Configuration for the button glyphs mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphConfig {
    pub glyph_style: GlyphStyle,
    pub scale: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_tint: Option<[f32; 4]>, // RGBA
}

impl Default for GlyphConfig {
    fn default() -> Self {
        Self {
            glyph_style: GlyphStyle::PlayStation,
            scale: 1.0,
            color_tint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_style_parsing() {
        assert_eq!("PlayStation".parse::<GlyphStyle>().unwrap(), GlyphStyle::PlayStation);
        assert_eq!("xbox".parse::<GlyphStyle>().unwrap(), GlyphStyle::Xbox);
        assert_eq!("Switch".parse::<GlyphStyle>().unwrap(), GlyphStyle::Switch);
        assert_eq!("generic".parse::<GlyphStyle>().unwrap(), GlyphStyle::Generic);
        assert!("Invalid".parse::<GlyphStyle>().is_err());
    }

    #[test]
    fn test_playstation_glyphs() {
        let glyphs = GlyphSet::playstation();
        assert_eq!(glyphs.get_glyph("Cross"), Some("✕"));
        assert_eq!(glyphs.get_glyph("Circle"), Some("◯"));
        assert_eq!(glyphs.get_glyph("Square"), Some("□"));
        assert_eq!(glyphs.get_glyph("Triangle"), Some("△"));
        assert_eq!(glyphs.get_glyph("L1"), Some("L1"));
        assert_eq!(glyphs.get_glyph("R1"), Some("R1"));
    }

    #[test]
    fn test_xbox_glyphs() {
        let glyphs = GlyphSet::xbox();
        assert_eq!(glyphs.get_glyph("A"), Some("A"));
        assert_eq!(glyphs.get_glyph("B"), Some("B"));
        assert_eq!(glyphs.get_glyph("X"), Some("X"));
        assert_eq!(glyphs.get_glyph("Y"), Some("Y"));
        assert_eq!(glyphs.get_glyph("LB"), Some("LB"));
        assert_eq!(glyphs.get_glyph("RB"), Some("RB"));
    }

    #[test]
    fn test_switch_glyphs_swapped_ab() {
        let glyphs = GlyphSet::switch();
        // Switch has A/B swapped compared to Xbox
        assert_eq!(glyphs.get_glyph("A"), Some("B"));  // A is on the right (East)
        assert_eq!(glyphs.get_glyph("B"), Some("A"));  // B is on the bottom (South)
    }

    #[test]
    fn test_generic_glyphs() {
        let glyphs = GlyphSet::generic();
        assert_eq!(glyphs.get_glyph("A"), Some("[A]"));
        assert_eq!(glyphs.get_glyph("B"), Some("[B]"));
        assert_eq!(glyphs.get_glyph("L1"), Some("[L1]"));
    }

    #[test]
    fn test_glyph_set_default_for() {
        assert_eq!(GlyphSet::default_for(GlyphStyle::PlayStation).style, GlyphStyle::PlayStation);
        assert_eq!(GlyphSet::default_for(GlyphStyle::Xbox).style, GlyphStyle::Xbox);
        assert_eq!(GlyphSet::default_for(GlyphStyle::Switch).style, GlyphStyle::Switch);
        assert_eq!(GlyphSet::default_for(GlyphStyle::Generic).style, GlyphStyle::Generic);
    }

    #[test]
    fn test_glyph_config_default() {
        let config = GlyphConfig::default();
        assert_eq!(config.glyph_style, GlyphStyle::PlayStation);
        assert_eq!(config.scale, 1.0);
        assert_eq!(config.color_tint, None);
    }
}