//! Mod profiling and debugging utilities
//!
//! This module provides optional profiling capabilities for the mod system.
//! Profiling is disabled by default and can be enabled via environment variable
//! or configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Global profiling configuration
static PROFILING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check if profiling is enabled
pub fn is_profiling_enabled() -> bool {
    // Check environment variable first (only on first call, cached)
    if !PROFILING_ENABLED.load(Ordering::Relaxed) {
        if let Ok(val) = std::env::var("LOOKING_GLASS_MOD_PROFILE") {
            if val == "1" || val.eq_ignore_ascii_case("true") {
                PROFILING_ENABLED.store(true, Ordering::Relaxed);
                return true;
            }
        }
        // Could also check config file here
        false
    } else {
        true
    }
}

/// Enable/disable profiling programmatically
pub fn set_profiling_enabled(enabled: bool) {
    PROFILING_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Timer for measuring elapsed time
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// Profiling data for a single mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModProfile {
    pub mod_id: String,
    pub load_time_ms: f64,
    pub init_time_ms: f64,
    pub total_frame_time_ms: f64,
    pub frame_count: u64,
    pub asset_override_time_ms: f64,
    pub asset_override_count: u64,
    pub input_hook_calls: u64,
    pub input_hook_time_ms: f64,
    pub runtime_hook_calls: u64,
    pub runtime_hook_time_ms: f64,
    pub ui_patch_count: u64,
    pub memory_estimate_bytes: u64,
}

impl Default for ModProfile {
    fn default() -> Self {
        Self {
            mod_id: String::new(),
            load_time_ms: 0.0,
            init_time_ms: 0.0,
            total_frame_time_ms: 0.0,
            frame_count: 0,
            asset_override_time_ms: 0.0,
            asset_override_count: 0,
            input_hook_calls: 0,
            input_hook_time_ms: 0.0,
            runtime_hook_calls: 0,
            runtime_hook_time_ms: 0.0,
            ui_patch_count: 0,
            memory_estimate_bytes: 0,
        }
    }
}

/// Internal mutable profiling state
struct ProfilingState {
    mod_profiles: HashMap<String, ModProfile>,
    global_load_time: Duration,
    global_init_time: Duration,
    global_frame_time: Duration,
    frame_count: u64,
}

impl ProfilingState {
    fn new() -> Self {
        Self {
            mod_profiles: HashMap::new(),
            global_load_time: Duration::ZERO,
            global_init_time: Duration::ZERO,
            global_frame_time: Duration::ZERO,
            frame_count: 0,
        }
    }
}

/// Global profiling state
static PROFILING_STATE: OnceLock<Arc<Mutex<ProfilingState>>> = OnceLock::new();

fn get_profiling_state() -> Arc<Mutex<ProfilingState>> {
    PROFILING_STATE
        .get_or_init(|| Arc::new(Mutex::new(ProfilingState::new())))
        .clone()
}

/// Start a profiling session (called when mod loading begins)
pub fn start_profiling_session() {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    *state = ProfilingState::new();
}

/// Record mod load time
pub fn record_mod_load_time(mod_id: &str, duration: Duration) {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    state.global_load_time += duration;
    state
        .mod_profiles
        .entry(mod_id.to_string())
        .or_default()
        .load_time_ms += duration.as_secs_f64() * 1000.0;
}

/// Record mod initialization time
pub fn record_mod_init_time(mod_id: &str, duration: Duration) {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    state.global_init_time += duration;
    state
        .mod_profiles
        .entry(mod_id.to_string())
        .or_default()
        .init_time_ms += duration.as_secs_f64() * 1000.0;
}

/// Record asset override time
pub fn record_asset_override_time(mod_id: &str, duration: Duration) {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    let profile = state.mod_profiles.entry(mod_id.to_string()).or_default();
    profile.asset_override_time_ms += duration.as_secs_f64() * 1000.0;
    profile.asset_override_count += 1;
}

/// Record input hook call
pub fn record_input_hook_call(mod_id: &str, duration: Duration) {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    let profile = state.mod_profiles.entry(mod_id.to_string()).or_default();
    profile.input_hook_calls += 1;
    profile.input_hook_time_ms += duration.as_secs_f64() * 1000.0;
}

/// Record runtime hook call
pub fn record_runtime_hook_call(mod_id: &str, duration: Duration) {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    let profile = state.mod_profiles.entry(mod_id.to_string()).or_default();
    profile.runtime_hook_calls += 1;
    profile.runtime_hook_time_ms += duration.as_secs_f64() * 1000.0;
}

/// Record frame update time
pub fn record_frame_time(mod_id: &str, duration: Duration) {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    state.global_frame_time += duration;
    state.frame_count += 1;
    let profile = state.mod_profiles.entry(mod_id.to_string()).or_default();
    profile.total_frame_time_ms += duration.as_secs_f64() * 1000.0;
    profile.frame_count += 1;
}

/// Record UI patch registration
pub fn record_ui_patch(mod_id: &str) {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    let profile = state.mod_profiles.entry(mod_id.to_string()).or_default();
    profile.ui_patch_count += 1;
}

/// Update memory estimate for a mod
pub fn update_memory_estimate(mod_id: &str, bytes: u64) {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    let profile = state.mod_profiles.entry(mod_id.to_string()).or_default();
    profile.memory_estimate_bytes = bytes;
}

/// Get all mod profiles
pub fn get_mod_profiles() -> HashMap<String, ModProfile> {
    if !is_profiling_enabled() {
        return HashMap::new();
    }
    let state = get_profiling_state();
    let state = state.lock().unwrap();
    state.mod_profiles.clone()
}

/// Get a specific mod profile
pub fn get_mod_profile(mod_id: &str) -> Option<ModProfile> {
    if !is_profiling_enabled() {
        return None;
    }
    let state = get_profiling_state();
    let state = state.lock().unwrap();
    state.mod_profiles.get(mod_id).cloned()
}

/// Get global profiling summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProfileSummary {
    pub total_mods: usize,
    pub total_load_time_ms: f64,
    pub total_init_time_ms: f64,
    pub total_frame_time_ms: f64,
    pub frame_count: u64,
    pub average_frame_time_ms: f64,
    pub mods: HashMap<String, ModProfile>,
}

pub fn get_global_summary() -> GlobalProfileSummary {
    if !is_profiling_enabled() {
        return GlobalProfileSummary {
            total_mods: 0,
            total_load_time_ms: 0.0,
            total_init_time_ms: 0.0,
            total_frame_time_ms: 0.0,
            frame_count: 0,
            average_frame_time_ms: 0.0,
            mods: HashMap::new(),
        };
    }
    let state = get_profiling_state();
    let state = state.lock().unwrap();

    GlobalProfileSummary {
        total_mods: state.mod_profiles.len(),
        total_load_time_ms: state.global_load_time.as_secs_f64() * 1000.0,
        total_init_time_ms: state.global_init_time.as_secs_f64() * 1000.0,
        total_frame_time_ms: state.global_frame_time.as_secs_f64() * 1000.0,
        frame_count: state.frame_count,
        average_frame_time_ms: if state.frame_count > 0 {
            state.global_frame_time.as_secs_f64() * 1000.0 / state.frame_count as f64
        } else {
            0.0
        },
        mods: state.mod_profiles.clone(),
    }
}

/// Reset all profiling data
pub fn reset_profiling() {
    if !is_profiling_enabled() {
        return;
    }
    let state = get_profiling_state();
    let mut state = state.lock().unwrap();
    *state = ProfilingState::new();
}

/// Profiling guard for automatic timing
pub struct ProfilingGuard {
    mod_id: String,
    timer: Timer,
    kind: ProfilingKind,
}

pub enum ProfilingKind {
    Load,
    Init,
    AssetOverride,
    InputHook,
    RuntimeHook,
    Frame,
}

impl ProfilingGuard {
    pub fn new(mod_id: &str, kind: ProfilingKind) -> Self {
        Self {
            mod_id: mod_id.to_string(),
            timer: Timer::new(),
            kind,
        }
    }
}

impl Drop for ProfilingGuard {
    fn drop(&mut self) {
        if !is_profiling_enabled() {
            return;
        }
        let duration = self.timer.elapsed();
        match self.kind {
            ProfilingKind::Load => record_mod_load_time(&self.mod_id, duration),
            ProfilingKind::Init => record_mod_init_time(&self.mod_id, duration),
            ProfilingKind::AssetOverride => record_asset_override_time(&self.mod_id, duration),
            ProfilingKind::InputHook => record_input_hook_call(&self.mod_id, duration),
            ProfilingKind::RuntimeHook => record_runtime_hook_call(&self.mod_id, duration),
            ProfilingKind::Frame => record_frame_time(&self.mod_id, duration),
        }
    }
}

/// Convenience macros for profiling
#[macro_export]
macro_rules! profile_mod_load {
    ($mod_id:expr, $code:block) => {{
        let _guard = $crate::profiling::ProfilingGuard::new($mod_id, $crate::profiling::ProfilingKind::Load);
        $code
    }};
}

#[macro_export]
macro_rules! profile_mod_init {
    ($mod_id:expr, $code:block) => {{
        let _guard = $crate::profiling::ProfilingGuard::new($mod_id, $crate::profiling::ProfilingKind::Init);
        $code
    }};
}

#[macro_export]
macro_rules! profile_asset_override {
    ($mod_id:expr, $code:block) => {{
        let _guard = $crate::profiling::ProfilingGuard::new($mod_id, $crate::profiling::ProfilingKind::AssetOverride);
        $code
    }};
}

#[macro_export]
macro_rules! profile_input_hook {
    ($mod_id:expr, $code:block) => {{
        let _guard = $crate::profiling::ProfilingGuard::new($mod_id, $crate::profiling::ProfilingKind::InputHook);
        $code
    }};
}

#[macro_export]
macro_rules! profile_runtime_hook {
    ($mod_id:expr, $code:block) => {{
        let _guard = $crate::profiling::ProfilingGuard::new($mod_id, $crate::profiling::ProfilingKind::RuntimeHook);
        $code
    }};
}

#[macro_export]
macro_rules! profile_frame {
    ($mod_id:expr, $code:block) => {{
        let _guard = $crate::profiling::ProfilingGuard::new($mod_id, $crate::profiling::ProfilingKind::Frame);
        $code
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiling_disabled_by_default() {
        assert!(!is_profiling_enabled());
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10.0);
    }

    #[test]
    fn test_profiling_guard() {
        set_profiling_enabled(true);
        start_profiling_session();

        {
            let _guard = ProfilingGuard::new("test_mod", ProfilingKind::Load);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let profile = get_mod_profile("test_mod").unwrap();
        assert!(profile.load_time_ms >= 5.0);

        set_profiling_enabled(false);
        reset_profiling();
    }
}