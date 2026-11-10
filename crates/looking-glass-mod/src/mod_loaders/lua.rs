use crate::loader::{Mod, ModContext, ModLoader};
use crate::manifest::{ModManifest, ModEntryPoints, GameTarget, AssetType, AssetMapping, ModDependency, LoadOrder, Permission, PermissionSet, Permissions};
use anyhow::{Context, Result};
use mlua::{Lua, Table, Value, Function, StdLib, MetaMethod};
use std::path::PathBuf;
use std::sync::Arc;

pub struct LuaModLoader;

impl LuaModLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn find_lua_script(mod_dir: &PathBuf) -> Option<PathBuf> {
        let candidates = [
            mod_dir.join("mod.lua"),
            mod_dir.join("init.lua"),
            mod_dir.join("main.lua"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Some(candidate.clone());
            }
        }
        None
    }

    /// Create a sandboxed Lua state based on the mod's permissions
    fn create_sandboxed_lua(&self, permissions: &PermissionSet) -> Result<Lua> {
        let lua = Lua::new();

        // Load standard libraries manually (mlua 0.9 doesn't have StdLib::ALL working)
        // Load standard libraries manually (mlua 0.9 doesn't have chunk! macro)
        lua.load(r#"
            require("coroutine")
            require("table")
            require("string")
            require("math")
            require("io")
            require("os")
            require("package")
            require("debug")
            require("utf8")
        "#).exec()?;

        // Remove or wrap dangerous global functions based on permissions
        self.restrict_stdlib(&lua, permissions)?;

        Ok(lua)
    }

    /// Load the mod_api module (our custom API for mods)
    fn load_mod_api(lua: &Lua) -> Result<Table> {
        let mod_api = lua.create_table()?;

        // Add safe functions here that mods can use
        // These would be implemented to communicate with the host

        // Example: logging
        mod_api.set("log", lua.create_function(|_, (level, msg): (String, String)| {
            match level.as_str() {
                "info" => tracing::info!("[Mod] {}", msg),
                "warn" => tracing::warn!("[Mod] {}", msg),
                "error" => tracing::error!("[Mod] {}", msg),
                "debug" => tracing::debug!("[Mod] {}", msg),
                _ => tracing::info!("[Mod] {}", msg),
            }
            Ok(())
        })?)?;

        // Version info
        mod_api.set("version", env!("CARGO_PKG_VERSION"))?;

        Ok(mod_api)
    }

    /// Remove or wrap dangerous stdlib functions based on permissions
    fn restrict_stdlib(&self, lua: &Lua, permissions: &PermissionSet) -> Result<()> {
        let globals = lua.globals();

        // Always remove os.execute and os.spawn (process_spawn permission)
        if !permissions.is_granted(&Permission::ProcessSpawn) {
            let os: Table = globals.get("os")?;
            os.set("execute", Value::Nil)?;
            os.set("spawn", Value::Nil)?;
            // Also remove os.remove, os.rename, os.tmpname for filesystem_write
            if !permissions.is_granted(&Permission::FilesystemWrite) {
                os.set("remove", Value::Nil)?;
                os.set("rename", Value::Nil)?;
                os.set("tmpname", Value::Nil)?;
            }
        }

        // Restrict io library for filesystem access
        if !permissions.is_granted(&Permission::FilesystemRead) && !permissions.is_granted(&Permission::FilesystemWrite) {
            let io: Table = globals.get("io")?;
            // Remove file operations
            io.set("open", Value::Nil)?;
            io.set("close", Value::Nil)?;
            io.set("read", Value::Nil)?;
            io.set("write", Value::Nil)?;
            io.set("lines", Value::Nil)?;
            io.set("input", Value::Nil)?;
            io.set("output", Value::Nil)?;
            io.set("popen", Value::Nil)?;
            io.set("tmpfile", Value::Nil)?;
            io.set("type", Value::Nil)?;
            io.set("flush", Value::Nil)?;
        } else if permissions.is_granted(&Permission::FilesystemRead) && !permissions.is_granted(&Permission::FilesystemWrite) {
            // Allow read-only operations
            let io: Table = globals.get("io")?;
            io.set("write", Value::Nil)?;
            io.set("popen", Value::Nil)?;
            io.set("tmpfile", Value::Nil)?;
            io.set("output", Value::Nil)?;
        }

        // Restrict debug library (can be used to bypass sandbox)
        let debug: Table = globals.get("debug")?;
        debug.set("debug", Value::Nil)?;
        debug.set("gethook", Value::Nil)?;
        debug.set("sethook", Value::Nil)?;
        debug.set("getlocal", Value::Nil)?;
        debug.set("setlocal", Value::Nil)?;
        debug.set("getupvalue", Value::Nil)?;
        debug.set("setupvalue", Value::Nil)?;
        debug.set("getregistry", Value::Nil)?;

        // Restrict package library (module loading)
        if !permissions.is_granted(&Permission::NativeCodeExecution) {
            let package: Table = globals.get("package")?;
            package.set("loadlib", Value::Nil)?;
            package.set("cpath", Value::Nil)?;
        }

        // Restrict os library for system access
        if !permissions.is_granted(&Permission::ProcessSpawn) {
            let os: Table = globals.get("os")?;
            os.set("execute", Value::Nil)?;
            os.set("exit", Value::Nil)?;
            os.set("getenv", Value::Nil)?; // Could leak env vars
            os.set("setlocale", Value::Nil)?;
        }

        // Network access would require socket library which isn't in stdlib by default
        // If a mod has network_access, they'd need a custom module

        Ok(())
    }
}

#[async_trait::async_trait]
impl ModLoader for LuaModLoader {
    async fn load_mod_with_permissions(
        &self,
        mod_dir: &PathBuf,
        permissions: &PermissionSet,
    ) -> Result<Box<dyn Mod + Send + Sync + 'static>> {
        let script_path = Self::find_lua_script(mod_dir)
            .context("No Lua script found in mod directory")?;

        // First, load the manifest from a temporary unrestricted Lua state
        let temp_lua = Lua::new();
        // Load standard libraries manually (mlua 0.9 doesn't have chunk! macro)
        temp_lua.load(r#"
            require("coroutine")
            require("table")
            require("string")
            require("math")
            require("io")
            require("os")
            require("package")
            require("debug")
            require("utf8")
        "#).exec()?;

        let script_content = std::fs::read_to_string(&script_path)
            .with_context(|| format!("Failed to read Lua script: {}", script_path.display()))?;

        // Load manifest
        let manifest = self.load_manifest(&temp_lua, &script_content, &script_path)?;

        // Now create the sandboxed Lua state with the provided permissions
        let lua = self.create_sandboxed_lua(permissions)?;

        // Create the mod instance with the provided permission set
        let lua_mod = LuaMod {
            manifest,
            lua: Arc::new(lua),
            script_path: script_path.clone(),
            permission_set: permissions.clone(),
        };

        Ok(Box::new(lua_mod))
    }

    async fn unload_mod(&self, _mod_id: &str) -> Result<()> {
        Ok(())
    }

    fn supported_game_targets(&self) -> Vec<GameTarget> {
        vec![GameTarget::Alice]
    }
}

impl LuaModLoader {
    fn load_manifest(&self, lua: &Lua, script_content: &str, script_path: &PathBuf) -> Result<ModManifest> {
        // Execute the script to get the manifest table
        let globals = lua.globals();

        // Create a module table for the mod
        let mod_table = lua.create_table()?;
        globals.set("mod", mod_table.clone())?;

        // Execute the script
        lua.load(script_content)
            .set_name(script_path.to_string_lossy().to_string())
            .exec()
            .with_context(|| format!("Failed to execute Lua script: {}", script_path.display()))?;

        // Extract manifest from the mod table
        let manifest_table: Table = globals.get("manifest")?;

        let id: String = manifest_table.get("id")?;
        let name: String = manifest_table.get("name")?;
        let version: String = manifest_table.get("version")?;
        let manifest_version: u32 = manifest_table.get("manifest_version").unwrap_or(2);

        let author: Option<String> = manifest_table.get("author").ok();
        let description: Option<String> = manifest_table.get("description").ok();
        let homepage: Option<String> = manifest_table.get("homepage").ok();
        let repository: Option<String> = manifest_table.get("repository").ok();
        let license: Option<String> = manifest_table.get("license").ok();

        let game_target_str: String = manifest_table.get("game_target").unwrap_or_else(|_| "Alice".to_string());
        let game_target = game_target_str.parse()?;

        let min_game_version: Option<String> = manifest_table.get("min_game_version").ok();
        let max_game_version: Option<String> = manifest_table.get("max_game_version").ok();

        // Load dependencies
        let mut dependencies = Vec::new();
        if let Ok(deps_table) = manifest_table.get::<_, Table>("dependencies") {
            for pair in deps_table.pairs::<String, Table>() {
                let (_, dep_table) = pair?;
                let dep_id: String = dep_table.get("id")?;
                let dep_version: String = dep_table.get("version")?;
                let dep_optional: bool = dep_table.get("optional").unwrap_or(false);

                dependencies.push(ModDependency {
                    id: dep_id,
                    version: dep_version.parse()?,
                    optional: dep_optional,
                });
            }
        }

        // Load load_order
        let mut load_order = LoadOrder::default();
        if let Ok(lo_table) = manifest_table.get::<_, Table>("load_order") {
            if let Ok(before) = lo_table.get::<_, Vec<String>>("before") {
                load_order.before = before;
            }
            if let Ok(after) = lo_table.get::<_, Vec<String>>("after") {
                load_order.after = after;
            }
        }

        // Load entry_points
        let mut entry_points = ModEntryPoints::default();
        if let Ok(ep_table) = manifest_table.get::<_, Table>("entry_points") {
            entry_points.asset_override = ep_table.get("asset_override").ok();
            entry_points.config_inject = ep_table.get("config_inject").ok();
            entry_points.runtime_hook = ep_table.get("runtime_hook").ok();
            entry_points.ui_patch = ep_table.get("ui_patch").ok();
            entry_points.input_hook = ep_table.get("input_hook").ok();
        }

        // Load config_schema and default_config - need custom deserialization since serde_json::Value doesn't implement FromLua
        let config_schema: Option<serde_json::Value> = Self::get_json_value(&manifest_table, "config_schema").ok();
        let default_config: Option<serde_json::Value> = Self::get_json_value(&manifest_table, "default_config").ok();

        // Load assets
        let mut assets = Vec::new();
        if let Ok(assets_table) = manifest_table.get::<_, Table>("assets") {
            for pair in assets_table.pairs::<u32, Table>() {
                let (_, asset_table) = pair?;
                let source: String = asset_table.get("source")?;
                let target: String = asset_table.get("target")?;
                let asset_type_str: String = asset_table.get("type")?;
                let asset_type = asset_type_str.parse::<AssetType>()?;
                let condition: Option<String> = asset_table.get("condition").ok();

                assets.push(AssetMapping {
                    source,
                    target,
                    asset_type,
                    condition,
                });
            }
        }

        let tags: Vec<String> = manifest_table.get("tags").unwrap_or_default();

        // Load permissions - manually deserialize since Permissions doesn't implement FromLua
        let permissions = self.load_permissions(&manifest_table)?;

        Ok(ModManifest {
            id,
            name,
            version,
            manifest_version,
            author,
            description,
            homepage,
            repository,
            license,
            game_target,
            min_game_version,
            max_game_version,
            dependencies,
            load_order,
            entry_points,
            config_schema,
            default_config,
            assets,
            tags,
            permissions,
        })
    }

    fn get_json_value(table: &Table, key: &str) -> Result<serde_json::Value> {
        let value: Value = table.get(key)?;
        Self::lua_value_to_json(value)
    }

    fn lua_value_to_json(value: Value) -> Result<serde_json::Value> {
        match value {
            Value::Nil => Ok(serde_json::Value::Null),
            Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
            Value::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(i))),
            Value::Number(n) => {
                if let Some(f) = serde_json::Number::from_f64(n) {
                    Ok(serde_json::Value::Number(f))
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
            Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
            Value::Table(t) => {
                // Check if it's an array or object
                let len: i64 = t.len()?;
                let mut is_array = true;
                let mut max_idx: i64 = 0;

                // Clone the table for the first iteration since pairs() takes ownership
                let t_for_check = t.clone();
                for pair in t_for_check.pairs::<Value, Value>() {
                    let (k, _) = pair?;
                    match k {
                        Value::Integer(i) if i > 0 && i <= len => {
                            max_idx = max_idx.max(i);
                        }
                        _ => {
                            is_array = false;
                            break;
                        }
                    }
                }

                if is_array && max_idx == len {
                    let len_usize = len as usize;
                    let mut arr = Vec::with_capacity(len_usize);
                    for i in 1..=len_usize {
                        arr.push(Self::lua_value_to_json(t.get(i)?)?);
                    }
                    Ok(serde_json::Value::Array(arr))
                } else {
                    let mut obj = serde_json::Map::new();
                    for pair in t.pairs::<String, Value>() {
                        let (k, v) = pair?;
                        obj.insert(k, Self::lua_value_to_json(v)?);
                    }
                    Ok(serde_json::Value::Object(obj))
                }
            }
            _ => Ok(serde_json::Value::Null),
        }
    }

    fn load_permissions(&self, manifest_table: &Table) -> Result<Permissions> {
        let mut permissions = Permissions::default();

        if let Ok(perms_table) = manifest_table.get::<_, Table>("permissions") {
            // Load required permissions
            if let Ok(required_table) = perms_table.get::<_, Table>("required") {
                for pair in required_table.pairs::<u32, String>() {
                    let (_, perm_str) = pair?;
                    if let Ok(perm) = Permission::from_str(&perm_str) {
                        permissions.required.push(perm);
                    }
                }
            }

            // Load optional permissions
            if let Ok(optional_table) = perms_table.get::<_, Table>("optional") {
                for pair in optional_table.pairs::<u32, String>() {
                    let (_, perm_str) = pair?;
                    if let Ok(perm) = Permission::from_str(&perm_str) {
                        permissions.optional.push(perm);
                    }
                }
            }

            // Load not_needed permissions
            if let Ok(not_needed_table) = perms_table.get::<_, Table>("not_needed") {
                for pair in not_needed_table.pairs::<u32, String>() {
                    let (_, perm_str) = pair?;
                    if let Ok(perm) = Permission::from_str(&perm_str) {
                        permissions.not_needed.push(perm);
                    }
                }
            }
        }

        Ok(permissions)
    }
}

struct LuaMod {
    manifest: ModManifest,
    lua: Arc<Lua>,
    script_path: PathBuf,
    permission_set: PermissionSet,
}

#[async_trait::async_trait]
impl Mod for LuaMod {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        // Expose permissions to the mod's environment so it can check its own permissions
        self.expose_permissions_to_mod()?;

        // Call the mod's initialize function if it exists
        let globals = self.lua.globals();
        let init_func: Result<Function<'_>, mlua::Error> = globals.get("initialize");
        if let Ok(func) = init_func {
            func.call::<(), ()>(())
                .context("Failed to call Lua initialize function")?;
        }

        // Register asset overrides from manifest
        for asset in &self.manifest.assets {
            context.mod_api.register_asset_override(
                asset.target.clone(),
                self.script_path.parent().unwrap_or(&PathBuf::new()).join(&asset.source),
                asset.asset_type.clone(),
                asset.condition.clone(),
                100,
            );
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        let globals = self.lua.globals();
        let shutdown_func: Result<Function<'_>, mlua::Error> = globals.get("shutdown");
        if let Ok(func) = shutdown_func {
            func.call::<(), ()>(())
                .context("Failed to call Lua shutdown function")?;
        }
        Ok(())
    }

    async fn on_config_changed(&mut self, key: &str, value: &serde_json::Value) -> Result<()> {
        let globals = self.lua.globals();
        let config_func: Result<Function<'_>, mlua::Error> = globals.get("on_config_changed");
        if let Ok(func) = config_func {
            // Convert serde_json::Value to mlua::Value
            let lua_config = json_to_lua(&self.lua, value)?;
            // Pass key as &str (will be converted to Lua string) and lua_config as Value
            // Specify both type parameters: (Args, Return) = ((&str, mlua::Value), ())
            func.call::<(&str, mlua::Value), ()>((key, lua_config))
                .context("Failed to call Lua on_config_changed function")?;
        }
        Ok(())
    }

    fn manifest(&self) -> &ModManifest {
        &self.manifest
    }

    fn entry_points(&self) -> &ModEntryPoints {
        &self.manifest.entry_points
    }
}

impl LuaMod {
    fn expose_permissions_to_mod(&self) -> Result<()> {
        let globals = self.lua.globals();

        // Create a permission check function for the mod
        let permission_set = self.permission_set.clone();
        let check_perm = self.lua.create_function(move |_, perm_name: String| {
            let perm = Permission::from_str(&perm_name).ok();
            if let Some(p) = perm {
                Ok(permission_set.is_granted(&p))
            } else {
                Ok(false)
            }
        })?;

        // Create a permissions table
        let perms_table = self.lua.create_table()?;
        perms_table.set("check", check_perm)?;

        // Add list of granted permissions
        let granted: Vec<String> = self.permission_set.granted.iter()
            .map(|p| p.as_str().to_string())
            .collect();
        perms_table.set("granted", granted)?;

        // Add list of denied permissions
        let denied: Vec<String> = self.permission_set.denied.iter()
            .map(|p| p.as_str().to_string())
            .collect();
        perms_table.set("denied", denied)?;

        globals.set("permissions", perms_table)?;

        Ok(())
    }
}

// Helper function to convert serde_json::Value to mlua::Value
fn json_to_lua<'lua>(lua: &'lua Lua, value: &serde_json::Value) -> Result<Value<'lua>> {
    match value {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else {
                Ok(Value::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(lua.create_string(s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj {
                // Convert string key to owned String for set
                table.set(k.as_str(), json_to_lua(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

// Need to make LuaMod Send + Sync for the Mod trait
// Since mlua with "send" feature makes Lua Send, we need to ensure our wrapper is too
unsafe impl Send for LuaMod {}
unsafe impl Sync for LuaMod {}