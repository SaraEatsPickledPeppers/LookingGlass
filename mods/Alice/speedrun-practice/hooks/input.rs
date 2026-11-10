// Speedrun Practice Input Hook
// Handles input capture for input display and hotkeys

use looking_glass_mod::loader::{ModContext, ModApi, InputHook, ButtonOption, ButtonState, GamepadButton, KeyCode, KeyModifiers};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use anyhow::Result;
use serde_json::Value;

/// Input state for display
#[derive(Debug, Clone, Default)]
struct InputState {
    keyboard: HashMap<String, ButtonState>,
    gamepad_buttons: HashMap<GamepadButton, ButtonState>,
    gamepad_axes: HashMap<String, (f32, f32)>,
    last_keys: Vec<String>,
}

/// Speedrun practice input hook
pub struct SpeedrunInputHook {
    input_state: Arc<Mutex<InputState>>,
    config: SpeedrunInputConfig,
}

/// Input hook configuration
#[derive(Debug, Clone)]
struct SpeedrunInputConfig {
    enable_input_display: bool,
    input_display_position: String,
    show_keyboard: bool,
    show_gamepad: bool,
    show_axes: bool,
    fade_time: f32,
    scale: f32,
}

impl Default for SpeedrunInputConfig {
    fn default() -> Self {
        Self {
            enable_input_display: false,
            input_display_position: "bottom-left".to_string(),
            show_keyboard: true,
            show_gamepad: true,
            show_axes: true,
            fade_time: 0.5,
            scale: 1.0,
        }
    }
}

impl SpeedrunInputHook {
    pub fn new() -> Self {
        Self {
            input_state: Arc::new(Mutex::new(InputState::default())),
            config: SpeedrunInputConfig::default(),
        }
    }

    /// Initialize with config
    pub fn init(&mut self, config: Value) {
        if let Some(enable) = config.get("enable_input_display").and_then(|v| v.as_bool()) {
            self.config.enable_input_display = enable;
        }
        if let Some(pos) = config.get("input_display_position").and_then(|v| v.as_str()) {
            self.config.input_display_position = pos.to_string();
        }
        if let Some(show_kb) = config.get("show_keyboard").and_then(|v| v.as_bool()) {
            self.config.show_keyboard = show_kb;
        }
        if let Some(show_gp) = config.get("show_gamepad").and_then(|v| v.as_bool()) {
            self.config.show_gamepad = show_gp;
        }
        if let Some(show_axes) = config.get("show_axes").and_then(|v| v.as_bool()) {
            self.config.show_axes = show_axes;
        }
        if let Some(fade) = config.get("fade_time").and_then(|v| v.as_f64()) {
            self.config.fade_time = fade as f32;
        }
        if let Some(scale) = config.get("scale").and_then(|v| v.as_f64()) {
            self.config.scale = scale as f32;
        }
    }

    /// Update keyboard key state
    pub fn set_key_state(&self, key: &str, state: ButtonState) {
        if self.config.enable_input_display && self.config.show_keyboard {
            let mut input = self.input_state.lock().unwrap();
            input.keyboard.insert(key.to_string(), state);
            if state == ButtonState::Pressed {
                input.last_keys.push(key.to_string());
                if input.last_keys.len() > 10 {
                    input.last_keys.remove(0);
                }
            }
        }
    }

    /// Update gamepad button state
    pub fn set_gamepad_button_state(&self, button: GamepadButton, state: ButtonState) {
        if self.config.enable_input_display && self.config.show_gamepad {
            self.input_state.lock().unwrap().gamepad_buttons.insert(button, state);
        }
    }

    /// Update gamepad axis state
    pub fn set_gamepad_axis_state(&self, axis: &str, x: f32, y: f32) {
        if self.config.enable_input_display && self.config.show_axes {
            self.input_state.lock().unwrap().gamepad_axes.insert(axis.to_string(), (x, y));
        }
    }

    /// Get current input state for rendering
    pub fn get_input_state(&self) -> InputState {
        self.input_state.lock().unwrap().clone()
    }

    /// Check if input display is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enable_input_display
    }

    /// Get display position
    pub fn get_position(&self) -> &str {
        &self.config.input_display_position
    }

    /// Get scale
    pub fn get_scale(&self) -> f32 {
        self.config.scale
    }
}

impl InputHook for SpeedrunInputHook {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        // Register config validator
        let config = context.config.get("speedrun-practice").cloned().unwrap_or_default();
        self.init(config);

        // Register input override for pause buffering
        context.mod_api.register_input_override("speedrun_pause_buffer", |state| {
            // Allow inputs during pause for frame-perfect buffering
            Ok(state)
        })?;

        Ok(())
    }

    async fn on_button_prompt(&mut self, prompt_id: &str, options: &[ButtonOption]) -> Result<Option<usize>> {
        // Could customize button prompts for speedrun features
        Ok(None)
    }

    async fn process_keyboard(&mut self, key: KeyCode, modifiers: KeyModifiers, state: ButtonState) -> Result<bool> {
        let key_name = format!("{:?}", key);
        self.set_key_state(&key_name, state);

        // Pass through to game
        Ok(false)
    }

    async fn process_mouse_button(&mut self, button: u8, state: ButtonState) -> Result<bool> {
        let button_name = match button {
            0 => "MouseLeft",
            1 => "MouseRight",
            2 => "MouseMiddle",
            3 => "MouseX1",
            4 => "MouseX2",
            _ => "MouseUnknown",
        };
        self.set_key_state(button_name, state);
        Ok(false)
    }

    async fn process_mouse_motion(&mut self, x: f32, y: f32) -> Result<bool> {
        self.set_gamepad_axis_state("Mouse", x, y);
        Ok(false)
    }

    async fn process_gamepad_button(&mut self, button: GamepadButton, state: ButtonState) -> Result<bool> {
        self.set_gamepad_button_state(button, state);
        Ok(false)
    }

    async fn process_gamepad_axis(&mut self, axis: &str, x: f32, y: f32) -> Result<bool> {
        self.set_gamepad_axis_state(axis, x, y);
        Ok(false)
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

// Export the hook for the mod system
#[no_mangle]
pub extern "C" fn create_speedrun_input_hook() -> Box<dyn InputHook> {
    Box::new(SpeedrunInputHook::new())
}