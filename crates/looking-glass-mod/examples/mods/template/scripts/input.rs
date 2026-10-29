//! Template Mod - Input Hook Script
//! This is a placeholder for input hooks. Copy and modify for your mod.

use looking_glass_mod::{
    ModApi, ModContext, InputHook, KeyCode, KeyModifiers, GamepadButton, ButtonState,
    ButtonOption, Result, async_trait,
};

/// Example input hook for handling custom key bindings
pub struct TemplateInputHook {
    // Add your state here
}

impl TemplateInputHook {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl InputHook for TemplateInputHook {
    async fn on_button_prompt(
        &self,
        prompt_id: &str,
        options: &[ButtonOption],
    ) -> Option<usize> {
        // Called when the game shows a button prompt (e.g., "Press A to jump")
        // Return Some(index) to auto-select an option, None to let player choose
        // println!("Button prompt: {} with {} options", prompt_id, options.len());
        None
    }

    async fn on_key_down(&self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        // Called when a keyboard key is pressed
        // Return true to consume the input (prevent game from handling it)

        // Example: Toggle mod with Ctrl+M
        // if modifiers.control && key == KeyCode::M {
        //     println!("Mod toggle key pressed!");
        //     return true;
        // }

        // Example: Custom hotkey
        // if key == KeyCode::F1 {
        //     println!("F1 pressed - custom action!");
        //     return true;
        // }

        false
    }

    async fn on_key_up(&self, key: KeyCode, modifiers: KeyModifiers) -> bool {
        // Called when a keyboard key is released
        // Return true to consume the input
        false
    }

    async fn on_gamepad_button(
        &self,
        button: GamepadButton,
        state: ButtonState,
    ) -> bool {
        // Called when a gamepad button is pressed/released
        // Return true to consume the input

        // Example: Handle gamepad button combo
        // if state == ButtonState::Pressed {
        //     match button {
        //         GamepadButton::Start => println!("Start pressed"),
        //         GamepadButton::Select => println!("Select/Back pressed"),
        //         _ => {}
        //     }
        // }

        false
    }
}

/// Register the input hook
pub fn register_input_hook(api: &ModApi) {
    api.register_input_hook(Box::new(TemplateInputHook::new()));
}

/// Entry point called by the mod loader
#[no_mangle]
pub extern "C" fn register_input_hook_entry(api: &ModApi, _context: &ModContext) {
    register_input_hook(api);
}