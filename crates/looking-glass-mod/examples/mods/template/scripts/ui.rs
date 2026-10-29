//! Template Mod - UI Patch Script
//! This is a placeholder for UI patches. Copy and modify for your mod.

use looking_glass_mod::{ModApi, ModContext, UiPatch, UiPatchOp, WidgetDef, Result};

/// Register UI patches for this mod
pub fn register_ui_patches(api: &ModApi) {
    // Example: Add a custom widget to the main menu
    // let patch = UiPatch {
    //     target: "main_menu".to_string(),
    //     operations: vec![
    //         UiPatchOp::AddWidget {
    //             parent: "root".to_string(),
    //             widget: WidgetDef {
    //                 id: "my_custom_button".to_string(),
    //                 widget_type: "Button".to_string(),
    //                 properties: {
    //                     let mut props = std::collections::HashMap::new();
    //                     props.insert("text".to_string(), serde_json::Value::String("My Mod Settings".to_string()));
    //                     props.insert("on_click".to_string(), serde_json::Value::String("open_my_mod_menu".to_string()));
    //                     props
    //                 },
    //                 children: vec![],
    //             },
    //         },
    //     ],
    // };
    // api.register_ui_patch(patch);

    // Example: Modify existing widget
    // let patch = UiPatch {
    //     target: "pause_menu".to_string(),
    //     operations: vec![
    //         UiPatchOp::ModifyWidget {
    //             id: "resume_button".to_string(),
    //             changes: {
    //                 let mut changes = std::collections::HashMap::new();
    //                 changes.insert("text".to_string(), serde_json::Value::String("Continue".to_string()));
    //                 changes
    //             },
    //         },
    //     ],
    // };
    // api.register_ui_patch(patch);
}

/// Entry point called by the mod loader
#[no_mangle]
pub extern "C" fn register_ui_patches_entry(api: &ModApi, _context: &ModContext) {
    register_ui_patches(api);
}