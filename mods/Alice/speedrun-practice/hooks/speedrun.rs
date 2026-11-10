// Speedrun Practice Runtime Hook
// Provides timer, splits, save states, and practice tools

use looking_glass_mod::loader::{ModContext, ModApi, RuntimeHook, ModState};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use anyhow::Result;
use serde_json::Value;

/// Timer state for speedrun practice
#[derive(Debug, Clone)]
struct TimerState {
    running: bool,
    start_time: Option<Instant>,
    elapsed: Duration,
    splits: Vec<Split>,
    current_split_index: usize,
    pb_splits: Option<Vec<Duration>>,
    best_segment_splits: Option<Vec<Duration>>,
    average_splits: Option<Vec<Duration>>,
}

/// Individual split data
#[derive(Debug, Clone)]
struct Split {
    name: String,
    time: Duration,
    segment_time: Duration,
    comparison: Option<SplitComparison>,
}

/// Comparison data for a split
#[derive(Debug, Clone)]
struct SplitComparison {
    pb: Option<Duration>,
    average: Option<Duration>,
    best_segment: Option<Duration>,
}

/// Save state for practice
#[derive(Debug, Clone)]
struct SaveState {
    slot: usize,
    timestamp: Instant,
    map_name: String,
    player_position: (f32, f32, f32),
    player_rotation: (f32, f32),
    game_time: Duration,
    inventory: Vec<String>,
    health: f32,
    will: f32,
}

/// Input display state
#[derive(Debug, Clone)]
struct InputDisplay {
    keys_pressed: HashMap<String, bool>,
    gamepad_buttons: HashMap<String, bool>,
    gamepad_axes: HashMap<String, (f32, f32)>,
}

/// Route visualization point
#[derive(Debug, Clone)]
struct RoutePoint {
    position: (f32, f32, f32),
    action: RouteAction,
}

/// Route action types
#[derive(Debug, Clone)]
enum RouteAction {
    Walk,
    Jump,
    Interact,
    Combat,
    Cutscene,
    LoadTrigger,
}

/// Main speedrun practice state
pub struct SpeedrunPracticeHook {
    timer: Arc<Mutex<TimerState>>,
    save_states: Arc<Mutex<HashMap<usize, SaveState>>>,
    input_display: Arc<Mutex<InputDisplay>>,
    route_points: Arc<Mutex<Vec<RoutePoint>>>,
    config: SpeedrunConfig,
    last_load_state: Option<SaveState>,
    pause_buffer_active: bool,
}

/// Configuration for speedrun practice
#[derive(Debug, Clone)]
struct SpeedrunConfig {
    enable_timer: bool,
    timer_position: String,
    show_splits: bool,
    split_comparison: String,
    enable_save_states: bool,
    save_state_slots: usize,
    enable_input_display: bool,
    input_display_position: String,
    enable_route_visualization: bool,
    auto_split_on_level: bool,
    auto_split_on_save: bool,
    timer_format: String,
    show_segment_pace: bool,
    enable_pause_buffering: bool,
    hotkey_timer_start: String,
    hotkey_timer_split: String,
    hotkey_save_state: String,
    hotkey_load_state: String,
    hotkey_undo_load: String,
}

impl Default for SpeedrunConfig {
    fn default() -> Self {
        Self {
            enable_timer: true,
            timer_position: "top-right".to_string(),
            show_splits: true,
            split_comparison: "pb".to_string(),
            enable_save_states: true,
            save_state_slots: 5,
            enable_input_display: false,
            input_display_position: "bottom-left".to_string(),
            enable_route_visualization: false,
            auto_split_on_level: true,
            auto_split_on_save: false,
            timer_format: "hmsc".to_string(),
            show_segment_pace: true,
            enable_pause_buffering: true,
            hotkey_timer_start: "F6".to_string(),
            hotkey_timer_split: "F7".to_string(),
            hotkey_save_state: "F8".to_string(),
            hotkey_load_state: "F9".to_string(),
            hotkey_undo_load: "F10".to_string(),
        }
    }
}

impl SpeedrunPracticeHook {
    pub fn new() -> Self {
        Self {
            timer: Arc::new(Mutex::new(TimerState {
                running: false,
                start_time: None,
                elapsed: Duration::ZERO,
                splits: Vec::new(),
                current_split_index: 0,
                pb_splits: None,
                best_segment_splits: None,
                average_splits: None,
            })),
            save_states: Arc::new(Mutex::new(HashMap::new())),
            input_display: Arc::new(Mutex::new(InputDisplay {
                keys_pressed: HashMap::new(),
                gamepad_buttons: HashMap::new(),
                gamepad_axes: HashMap::new(),
            })),
            route_points: Arc::new(Mutex::new(Vec::new())),
            config: SpeedrunConfig::default(),
            last_load_state: None,
            pause_buffer_active: false,
        }
    }

    /// Initialize the hook with config from mod settings
    pub fn init(&mut self, config: Value) {
        if let Some(enable_timer) = config.get("enable_timer").and_then(|v| v.as_bool()) {
            self.config.enable_timer = enable_timer;
        }
        if let Some(timer_position) = config.get("timer_position").and_then(|v| v.as_str()) {
            self.config.timer_position = timer_position.to_string();
        }
        if let Some(show_splits) = config.get("show_splits").and_then(|v| v.as_bool()) {
            self.config.show_splits = show_splits;
        }
        if let Some(split_comparison) = config.get("split_comparison").and_then(|v| v.as_str()) {
            self.config.split_comparison = split_comparison.to_string();
        }
        if let Some(enable_save_states) = config.get("enable_save_states").and_then(|v| v.as_bool()) {
            self.config.enable_save_states = enable_save_states;
        }
        if let Some(save_state_slots) = config.get("save_state_slots").and_then(|v| v.as_u64()) {
            self.config.save_state_slots = save_state_slots as usize;
        }
        if let Some(enable_input_display) = config.get("enable_input_display").and_then(|v| v.as_bool()) {
            self.config.enable_input_display = enable_input_display;
        }
        if let Some(input_display_position) = config.get("input_display_position").and_then(|v| v.as_str()) {
            self.config.input_display_position = input_display_position.to_string();
        }
        if let Some(enable_route_visualization) = config.get("enable_route_visualization").and_then(|v| v.as_bool()) {
            self.config.enable_route_visualization = enable_route_visualization;
        }
        if let Some(auto_split_on_level) = config.get("auto_split_on_level").and_then(|v| v.as_bool()) {
            self.config.auto_split_on_level = auto_split_on_level;
        }
        if let Some(auto_split_on_save) = config.get("auto_split_on_save").and_then(|v| v.as_bool()) {
            self.config.auto_split_on_save = auto_split_on_save;
        }
        if let Some(timer_format) = config.get("timer_format").and_then(|v| v.as_str()) {
            self.config.timer_format = timer_format.to_string();
        }
        if let Some(show_segment_pace) = config.get("show_segment_pace").and_then(|v| v.as_bool()) {
            self.config.show_segment_pace = show_segment_pace;
        }
        if let Some(enable_pause_buffering) = config.get("enable_pause_buffering").and_then(|v| v.as_bool()) {
            self.config.enable_pause_buffering = enable_pause_buffering;
        }
        if let Some(hotkey_timer_start) = config.get("hotkey_timer_start").and_then(|v| v.as_str()) {
            self.config.hotkey_timer_start = hotkey_timer_start.to_string();
        }
        if let Some(hotkey_timer_split) = config.get("hotkey_timer_split").and_then(|v| v.as_str()) {
            self.config.hotkey_timer_split = hotkey_timer_split.to_string();
        }
        if let Some(hotkey_save_state) = config.get("hotkey_save_state").and_then(|v| v.as_str()) {
            self.config.hotkey_save_state = hotkey_save_state.to_string();
        }
        if let Some(hotkey_load_state) = config.get("hotkey_load_state").and_then(|v| v.as_str()) {
            self.config.hotkey_load_state = hotkey_load_state.to_string();
        }
        if let Some(hotkey_undo_load) = config.get("hotkey_undo_load").and_then(|v| v.as_str()) {
            self.config.hotkey_undo_load = hotkey_undo_load.to_string();
        }
    }

    /// Start or reset the timer
    pub fn toggle_timer(&mut self) {
        let mut timer = self.timer.lock().unwrap();
        if timer.running {
            // Reset timer
            timer.running = false;
            timer.start_time = None;
            timer.elapsed = Duration::ZERO;
            timer.splits.clear();
            timer.current_split_index = 0;
        } else {
            // Start timer
            timer.running = true;
            timer.start_time = Some(Instant::now());
            timer.elapsed = Duration::ZERO;
        }
    }

    /// Create a manual split
    pub fn create_split(&mut self, name: String) {
        let mut timer = self.timer.lock().unwrap();
        if !timer.running {
            return;
        }

        let now = timer.start_time.unwrap().elapsed();
        let segment_time = if timer.splits.is_empty() {
            now
        } else {
            now - timer.splits.last().unwrap().time
        };

        let comparison = self.calculate_split_comparison(&timer, timer.current_split_index, segment_time);

        timer.splits.push(Split {
            name,
            time: now,
            segment_time,
            comparison,
        });
        timer.current_split_index += 1;
    }

    /// Calculate split comparison against PB, average, or best segment
    fn calculate_split_comparison(&self, timer: &TimerState, split_index: usize, segment_time: Duration) -> Option<SplitComparison> {
        let comparison_type = &self.config.split_comparison;
        if comparison_type == "none" {
            return None;
        }

        let comparison_splits = match comparison_type.as_str() {
            "pb" => &timer.pb_splits,
            "average" => &timer.average_splits,
            "best-segment" => &timer.best_segment_splits,
            _ => &timer.pb_splits,
        };

        if let Some(ref splits) = comparison_splits {
            if split_index < splits.len() {
                let target_time = splits[split_index];
                let diff = if segment_time > target_time {
                    segment_time - target_time
                } else {
                    target_time - segment_time
                };
                let ahead = segment_time <= target_time;

                return Some(SplitComparison {
                    pb: timer.pb_splits.as_ref().map(|s| s.get(split_index).copied()).flatten(),
                    average: timer.average_splits.as_ref().map(|s| s.get(split_index).copied()).flatten(),
                    best_segment: timer.best_segment_splits.as_ref().map(|s| s.get(split_index).copied()).flatten(),
                });
            }
        }
        None
    }

    /// Save current game state to a slot
    pub fn save_state(&mut self, slot: usize, map_name: String, position: (f32, f32, f32), rotation: (f32, f32), game_time: Duration, inventory: Vec<String>, health: f32, will: f32) {
        if !self.config.enable_save_states || slot >= self.config.save_state_slots {
            return;
        }

        let state = SaveState {
            slot,
            timestamp: Instant::now(),
            map_name,
            player_position: position,
            player_rotation: rotation,
            game_time,
            inventory,
            health,
            will,
        };

        self.save_states.lock().unwrap().insert(slot, state);
    }

    /// Load game state from a slot
    pub fn load_state(&mut self, slot: usize) -> Option<SaveState> {
        if !self.config.enable_save_states {
            return None;
        }

        let states = self.save_states.lock().unwrap();
        states.get(&slot).cloned()
    }

    /// Undo last load state
    pub fn undo_load(&mut self) -> Option<SaveState> {
        self.last_load_state.take()
    }

    /// Update timer each frame
    pub fn update_timer(&mut self, delta_time: f32) {
        let mut timer = self.timer.lock().unwrap();
        if timer.running {
            if let Some(start) = timer.start_time {
                timer.elapsed = start.elapsed();
            }
        }
    }

    /// Handle level transition for auto-splitting
    pub fn on_level_change(&mut self, new_map: &str) {
        if self.config.auto_split_on_level {
            self.create_split(new_map.to_string());
        }
    }

    /// Handle manual save for auto-splitting
    pub fn on_manual_save(&mut self) {
        if self.config.auto_split_on_save {
            self.create_split("Manual Save".to_string());
        }
    }

    /// Update input display state
    pub fn update_input(&mut self, key: &str, pressed: bool) {
        if self.config.enable_input_display {
            self.input_display.lock().unwrap().keys_pressed.insert(key.to_string(), pressed);
        }
    }

    /// Update gamepad input for display
    pub fn update_gamepad_input(&mut self, button: &str, pressed: bool) {
        if self.config.enable_input_display {
            self.input_display.lock().unwrap().gamepad_buttons.insert(button.to_string(), pressed);
        }
    }

    /// Update gamepad axis for display
    pub fn update_gamepad_axis(&mut self, axis: &str, x: f32, y: f32) {
        if self.config.enable_input_display {
            self.input_display.lock().unwrap().gamepad_axes.insert(axis.to_string(), (x, y));
        }
    }

    /// Get formatted timer string
    pub fn get_timer_string(&self) -> String {
        let timer = self.timer.lock().unwrap();
        if !timer.running && timer.start_time.is_none() && timer.elapsed == Duration::ZERO {
            return self.format_duration(Duration::ZERO);
        }
        self.format_duration(timer.elapsed)
    }

    /// Format duration based on timer_format config
    fn format_duration(&self, duration: Duration) -> String {
        let total_millis = duration.as_millis() as u64;
        let hours = total_millis / 3_600_000;
        let minutes = (total_millis % 3_600_000) / 60_000;
        let seconds = (total_millis % 60_000) / 1_000;
        let centiseconds = (total_millis % 1_000) / 10;

        match self.config.timer_format.as_str() {
            "hms" => {
                if hours > 0 {
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                } else {
                    format!("{:02}:{:02}", minutes, seconds)
                }
            }
            "hmsc" => {
                if hours > 0 {
                    format!("{:02}:{:02}:{:02}.{:02}", hours, minutes, seconds, centiseconds)
                } else {
                    format!("{:02}:{:02}.{:02}", minutes, seconds, centiseconds)
                }
            }
            "seconds" => {
                format!("{:.2}", total_millis as f64 / 1000.0)
            }
            _ => format!("{:02}:{:02}.{:02}", minutes, seconds, centiseconds),
        }
    }

    /// Get split strings for display
    pub fn get_split_strings(&self) -> Vec<String> {
        let timer = self.timer.lock().unwrap();
        if !self.config.show_splits || timer.splits.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        for (i, split) in timer.splits.iter().enumerate() {
            let split_str = format!("{}: {} ({})",
                i + 1,
                self.format_duration(split.time),
                self.format_duration(split.segment_time)
            );

            if let Some(ref comp) = split.comparison {
                if self.config.show_segment_pace {
                    if let Some(pb) = comp.pb {
                        let diff = if split.segment_time > pb {
                            split.segment_time - pb
                        } else {
                            pb - split.segment_time
                        };
                        let ahead = split.segment_time <= pb;
                        let sign = if ahead { "-" } else { "+" };
                        result.push(format!("{} {} {} vs PB", split_str, sign, self.format_duration(diff)));
                    } else {
                        result.push(split_str);
                    }
                } else {
                    result.push(split_str);
                }
            } else {
                result.push(split_str);
            }
        }
        result
    }

    /// Load PB splits from file
    pub fn load_pb_splits(&mut self, path: &str) {
        // Implementation would load from file
        // For now, create some example PB splits for skool1 -> potears1 -> etc.
        self.timer.lock().unwrap().pb_splits = Some(vec![
            Duration::from_secs(120),  // skool1
            Duration::from_secs(300),  // potears1
            Duration::from_secs(480),  // gvillage
            Duration::from_secs(720),  // pandemonium
            Duration::from_secs(1080), // etc.
        ]);
    }

    /// Check if a hotkey is pressed
    pub fn check_hotkey(&self, key: &str, hotkey: &str) -> bool {
        key.eq_ignore_ascii_case(hotkey)
    }
}

impl RuntimeHook for SpeedrunPracticeHook {
    async fn initialize(&mut self, context: &ModContext) -> Result<()> {
        // Register config validator
        let config = context.config.get("speedrun-practice").cloned().unwrap_or_default();
        self.init(config);

        // Load PB splits if available
        self.load_pb_splits("speedrun_pb.json");

        // Register commands for the console
        context.mod_api.register_command("speedrun_timer_start", |args| {
            // Start/reset timer
            Ok("Timer toggled".to_string())
        })?;

        context.mod_api.register_command("speedrun_split", |args| {
            // Manual split
            Ok("Split created".to_string())
        })?;

        context.mod_api.register_command("speedrun_save_state", |args| {
            // Save state: speedrun_save_state <slot>
            if let Some(slot_str) = args.first() {
                if let Ok(slot) = slot_str.parse::<usize>() {
                    // Would need game state access here
                    Ok(format!("Saved state to slot {}", slot))
                } else {
                    Err("Invalid slot number".into())
                }
            } else {
                Err("Usage: speedrun_save_state <slot>".into())
            }
        })?;

        context.mod_api.register_command("speedrun_load_state", |args| {
            // Load state: speedrun_load_state <slot>
            if let Some(slot_str) = args.first() {
                if let Ok(slot) = slot_str.parse::<usize>() {
                    Ok(format!("Loaded state from slot {}", slot))
                } else {
                    Err("Invalid slot number".into())
                }
            } else {
                Err("Usage: speedrun_load_state <slot>".into())
            }
        })?;

        context.mod_api.register_command("speedrun_undo_load", |args| {
            // Undo last load
            Ok("Undid last load".to_string())
        })?;

        Ok(())
    }

    async fn update(&mut self, delta_time: f32) -> Result<()> {
        self.update_timer(delta_time);

        // Check for pause buffering
        if self.config.enable_pause_buffering && self.pause_buffer_active {
            // Process buffered inputs during pause
        }

        Ok(())
    }

    async fn on_event(&mut self, event: &str, data: &Value) -> Result<()> {
        match event {
            "level_change" => {
                if let Some(map) = data.get("map").and_then(|v| v.as_str()) {
                    self.on_level_change(map);
                }
            }
            "manual_save" => {
                self.on_manual_save();
            }
            "input_key" => {
                if let Some(key) = data.get("key").and_then(|v| v.as_str()) {
                    if let Some(pressed) = data.get("pressed").and_then(|v| v.as_bool()) {
                        self.update_input(key, pressed);

                        // Check hotkeys
                        if self.check_hotkey(key, &self.config.hotkey_timer_start) && pressed {
                            self.toggle_timer();
                        } else if self.check_hotkey(key, &self.config.hotkey_timer_split) && pressed {
                            self.create_split("Manual".to_string());
                        }
                        // Save/load state hotkeys would need number key combination handling
                    }
                }
            }
            "input_gamepad_button" => {
                if let Some(button) = data.get("button").and_then(|v| v.as_str()) {
                    if let Some(pressed) = data.get("pressed").and_then(|v| v.as_bool()) {
                        self.update_gamepad_input(button, pressed);
                    }
                }
            }
            "input_gamepad_axis" => {
                if let Some(axis) = data.get("axis").and_then(|v| v.as_str()) {
                    if let Some(x) = data.get("x").and_then(|v| v.as_f64()) {
                        if let Some(y) = data.get("y").and_then(|v| v.as_f64()) {
                            self.update_gamepad_axis(axis, x as f32, y as f32);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // Save PB splits to file
        Ok(())
    }
}

// Export the hook for the mod system
#[no_mangle]
pub extern "C" fn create_speedrun_hook() -> Box<dyn RuntimeHook> {
    Box::new(SpeedrunPracticeHook::new())
}