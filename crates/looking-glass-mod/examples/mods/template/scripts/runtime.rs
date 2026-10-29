//! Template Mod - Runtime Hook Script
//! This is a placeholder for runtime hooks. Copy and modify for your mod.

use looking_glass_mod::{ModApi, ModContext, RuntimeHook, Result, async_trait};

/// Example runtime hook that runs every frame
pub struct TemplateRuntimeHook {
    // Add your state here
    counter: u64,
}

impl TemplateRuntimeHook {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

#[async_trait]
impl RuntimeHook for TemplateRuntimeHook {
    async fn on_frame(&self, delta_time: f32) {
        // Called every frame
        // self.counter += 1;
        // if self.counter % 60 == 0 {
        //     println!("Template mod: {} frames elapsed", self.counter);
        // }
    }

    async fn on_menu_open(&self, menu_id: &str) {
        // Called when a menu opens
        // println!("Menu opened: {}", menu_id);
    }

    async fn on_menu_close(&self, menu_id: &str) {
        // Called when a menu closes
        // println!("Menu closed: {}", menu_id);
    }

    async fn on_level_load(&self, level_id: &str) {
        // Called when a level loads
        // println!("Level loaded: {}", level_id);
    }

    async fn on_level_unload(&self, level_id: &str) {
        // Called when a level unloads
        // println!("Level unloaded: {}", level_id);
    }
}

/// Register the runtime hook
pub fn register_runtime_hook(api: &ModApi) {
    api.register_runtime_hook(Box::new(TemplateRuntimeHook::new()));
}

/// Entry point called by the mod loader
#[no_mangle]
pub extern "C" fn register_runtime_hook_entry(api: &ModApi, _context: &ModContext) {
    register_runtime_hook(api);
}