pub mod lua;
pub mod native;

use crate::loader::ModLoader;
use crate::manifest::GameTarget;

pub fn create_native_loader() -> Box<dyn ModLoader> {
    Box::new(native::NativeModLoader::new())
}

pub fn create_lua_loader() -> Box<dyn ModLoader> {
    Box::new(lua::LuaModLoader::new())
}

pub fn get_loader_for_mod(mod_dir: &std::path::Path) -> Option<Box<dyn ModLoader>> {
    let mod_dir_buf = mod_dir.to_path_buf();
    // Check for native library
    if native::NativeModLoader::find_library(&mod_dir_buf).is_some() {
        return Some(create_native_loader());
    }

    // Check for Lua script
    if lua::LuaModLoader::find_lua_script(&mod_dir_buf).is_some() {
        return Some(create_lua_loader());
    }

    None
}

pub fn get_supported_targets() -> Vec<GameTarget> {
    vec![GameTarget::Alice]
}