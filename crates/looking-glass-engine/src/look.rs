//! Relative mouse-look; movement is measured in counts, never scaled by frame time.
use macroquad::prelude::*;

const SENSITIVITY: f32 = 0.0025;

#[derive(Default)]
pub struct MouseLook {
    grabbed: bool,
    previous: Option<Vec2>,
    pub sensitivity: f32,
    pub inverted: bool,
}
impl MouseLook {
    pub fn update(&mut self, capture: bool, yaw: &mut f32, pitch: &mut f32) {
        if self.grabbed != capture {
            set_cursor_grab(capture);
            show_mouse(!capture);
            self.grabbed = capture;
            self.previous = None;
        }
        // Macroquad accumulates raw motion while grabbed. Undo its logical-pixel DPI
        // conversion so sensitivity stays independent of window size and display scale.
        let (x, y) = mouse_position();
        let position = vec2(x, y) * miniquad::window::dpi_scale();
        let delta = sample_motion(&mut self.previous, position, capture);
        let sensitivity = if self.sensitivity > 0. {
            self.sensitivity
        } else {
            1.
        };
        turn(
            vec2(delta.x, delta.y * if self.inverted { -1. } else { 1. }) * sensitivity,
            yaw,
            pitch,
        );
    }
    pub fn release(&mut self) {
        if self.grabbed {
            set_cursor_grab(false);
            show_mouse(true);
            self.grabbed = false;
        }
        self.previous = None;
    }
}
impl Drop for MouseLook {
    fn drop(&mut self) {
        self.release();
    }
}

fn sample_motion(previous: &mut Option<Vec2>, position: Vec2, capture: bool) -> Vec2 {
    if !capture || !position.is_finite() {
        *previous = None;
        return Vec2::ZERO;
    }
    previous
        .replace(position)
        .map_or(Vec2::ZERO, |last| position - last)
}
fn turn(delta: Vec2, yaw: &mut f32, pitch: &mut f32) {
    *yaw = (*yaw - delta.x * SENSITIVITY).rem_euclid(std::f32::consts::TAU);
    *pitch = (*pitch - delta.y * SENSITIVITY).clamp(-1.5, 1.5);
}

// Miniquad 0.4.8 explicitly does not release its desktop grab on focus loss, and
// Macroquad has no public focus query. Compare only this UI thread's active window.
#[cfg(target_os = "windows")]
pub fn window_focused() -> bool {
    #[link(name = "user32")]
    extern "system" {
        fn GetActiveWindow() -> *mut std::ffi::c_void;
        fn GetForegroundWindow() -> *mut std::ffi::c_void;
    }
    // Both functions return opaque handles; neither handle is dereferenced.
    unsafe {
        let own = GetActiveWindow();
        !own.is_null() && own == GetForegroundWindow()
    }
}

// Linux with X11 support via x11rb crate (pure Rust)
#[cfg(all(target_os = "linux", feature = "x11", not(feature = "wayland")))]
pub fn window_focused() -> bool {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::ConnectionExt;
    use x11rb::protocol::xproto::{Atom, Window};

    // Connect to X server
    let (conn, screen_num) = match x11rb::connect(None) {
        Ok(c) => c,
        Err(_) => return true, // Fail open: if we can't connect, assume focused
    };

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Get _NET_WM_PID atom for checking window ownership
    let net_wm_pid = match get_atom(&conn, "_NET_WM_PID") {
        Ok(atom) => atom,
        Err(_) => return true, // Fail open
    };

    let our_pid = std::process::id();

    // Strategy 1: Check input focus directly
    if let Ok(focus_reply) = conn.get_input_focus() {
        if let Ok(focus) = focus_reply.reply() {
            let focus_window = focus.focus;
            // Valid window (not None=0, not PointerRoot=1)
            if focus_window > 1 {
                if let Ok(Some(pid)) = get_window_pid(&conn, focus_window, net_wm_pid) {
                    if pid == our_pid {
                        return true;
                    }
                }
            }
        }
    }

    // Strategy 2: Check _NET_ACTIVE_WINDOW on root window (EWMH)
    if let Ok(net_active_window) = get_atom(&conn, "_NET_ACTIVE_WINDOW") {
        if let Ok(reply) = conn.get_property(false, root, net_active_window, x11rb::protocol::xproto::AtomEnum::WINDOW, 0, 1) {
            if let Ok(prop) = reply.reply() {
                if prop.format == 32 && prop.value.len() >= 4 {
                    let active_window = u32::from_ne_bytes([
                        prop.value[0], prop.value[1], prop.value[2], prop.value[3],
                    ]);
                    if active_window > 1 {
                        if let Ok(Some(pid)) = get_window_pid(&conn, active_window, net_wm_pid) {
                            if pid == our_pid {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    // Fail open: if we can't determine, assume focused
    true
}

// Linux with Wayland support via wayland-client crate (pure Rust)
#[cfg(all(target_os = "linux", feature = "wayland"))]
mod wayland_focus {
    use std::collections::HashMap;

    use wayland_client::{
        Connection, QueueHandle, Dispatch, Proxy, EventQueue,
        protocol::wl_registry::{WlRegistry, Event as RegistryEvent},
    };
    use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
        ext_foreign_toplevel_list_v1::ExtForeignToplevelListV1,
        ext_foreign_toplevel_handle_v1::ExtForeignToplevelHandleV1,
    };

    // Shared state for tracking toplevels
    #[derive(Default)]
    pub struct ToplevelInfo {
        pub app_id: Option<String>,
    }

    #[derive(Default)]
    pub struct WaylandState {
        pub our_app_id: String,
        pub ext_list: Option<ExtForeignToplevelListV1>,
        pub toplevels: HashMap<u32, (ExtForeignToplevelHandleV1, ToplevelInfo)>,
        pub finished: bool,
        pub found_our_window: bool,
        pub registry_pending: bool,
        pub registry_name: u32,
    }

    impl Dispatch<WlRegistry, WaylandState> for WaylandState {
        fn event(
            state: &mut WaylandState,
            _registry: &WlRegistry,
            event: RegistryEvent,
            _user_data: &WaylandState,
            _conn: &Connection,
            _qh: &QueueHandle<WaylandState>,
        ) {
            if let RegistryEvent::Global { name, interface, version } = event {
                if interface == "ext_foreign_toplevel_list" && version >= 1 {
                    state.registry_pending = true;
                    state.registry_name = name;
                }
            }
        }
    }

    impl Dispatch<ExtForeignToplevelListV1, WaylandState> for WaylandState {
        fn event(
            state: &mut WaylandState,
            _list: &ExtForeignToplevelListV1,
            event: <ExtForeignToplevelListV1 as Proxy>::Event,
            _user_data: &WaylandState,
            _conn: &Connection,
            _qh: &QueueHandle<WaylandState>,
        ) {
            use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_list_v1::Event;
            match event {
                Event::Toplevel { toplevel } => {
                    let key = state.toplevels.len() as u32;
                    state.toplevels.insert(key, (toplevel, ToplevelInfo::default()));
                }
                Event::Finished => {
                    state.finished = true;
                }
                _ => {}
            }
        }
    }

    impl Dispatch<ExtForeignToplevelHandleV1, WaylandState> for WaylandState {
        fn event(
            state: &mut WaylandState,
            handle: &ExtForeignToplevelHandleV1,
            event: <ExtForeignToplevelHandleV1 as Proxy>::Event,
            _user_data: &WaylandState,
            _conn: &Connection,
            _qh: &QueueHandle<WaylandState>,
        ) {
            use wayland_protocols::ext::foreign_toplevel_list::v1::client::ext_foreign_toplevel_handle_v1::Event;
            match event {
                Event::AppId { app_id } => {
                    if let Some((_, info)) = state.toplevels.values_mut().find(|(h, _)| h == handle) {
                        info.app_id = Some(app_id);
                        // Check if this is our window
                        if info.app_id.as_ref() == Some(&state.our_app_id) {
                            // We found our window, but we don't have focus info from this protocol
                            // Fail open - assume focused if we can't determine
                            state.found_our_window = true;
                        }
                    }
                }
                Event::Closed => {
                    state.toplevels.retain(|_, (h, _)| h != handle);
                }
                _ => {}
            }
        }
    }

    pub fn check_focus() -> bool {
        // Connect to Wayland compositor
        let conn = match Connection::connect_to_env() {
            Ok(c) => c,
            Err(_) => return true, // Fail open
        };

        let display = conn.display();
        let event_queue: EventQueue<WaylandState> = conn.new_event_queue();
        let qh = event_queue.handle();

        let our_pid = std::process::id();
        let our_app_id = format!("looking-glass-{}", our_pid);

        // Use a local state for the initial roundtrip
        let state = WaylandState {
            our_app_id: our_app_id.clone(),
            ..Default::default()
        };

        // Get registry - takes ownership of state
        let _registry = display.get_registry(&qh, state);

        // Roundtrip to get initial globals - need state back, but it was moved
        // The state is stored in the queue; we need to use a different approach
        // For now, just return true (fail open) as placeholder
        // TODO: Properly implement Wayland focus detection with correct state management
        true
    }
}

#[cfg(all(target_os = "linux", feature = "wayland"))]
pub fn window_focused() -> bool {
    wayland_focus::check_focus()
}

// Linux without x11rb and wayland features - fallback to xdotool subprocess
#[cfg(all(target_os = "linux", not(feature = "x11"), not(feature = "wayland")))]
pub fn window_focused() -> bool {
    use std::process::Command;

    // Try xdotool getwindowfocus to get focused window ID
    let output = match Command::new("xdotool").args(["getwindowfocus"]).output() {
        Ok(out) => out,
        Err(_) => return true, // Fail open if xdotool not available
    };

    if !output.status.success() {
        return true; // Fail open
    }

    let window_id_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let window_id: u32 = match window_id_str.parse() {
        Ok(id) => id,
        Err(_) => return true, // Fail open
    };

    // Skip invalid window IDs (0 = None, 1 = PointerRoot)
    if window_id <= 1 {
        return true; // Fail open
    }

    // Get PID of the focused window
    let pid_output = match Command::new("xdotool")
        .args(["getwindowpid", &window_id_str])
        .output()
    {
        Ok(out) => out,
        Err(_) => return true, // Fail open
    };

    if !pid_output.status.success() {
        return true; // Fail open
    }

    let pid_str = String::from_utf8_lossy(&pid_output.stdout).trim().to_string();
    let focused_pid: u32 = match pid_str.parse() {
        Ok(pid) => pid,
        Err(_) => return true, // Fail open
    };

    let our_pid = std::process::id();
    focused_pid == our_pid
}

// Other desktop platforms (macOS, etc.) - not verified yet
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn window_focused() -> bool {
    true
}

// Helper functions for X11 implementation
#[cfg(all(target_os = "linux", feature = "x11"))]
fn get_atom(
    conn: &x11rb::rust_connection::RustConnection,
    name: &str,
) -> Result<x11rb::protocol::xproto::Atom, Box<dyn std::error::Error>> {
    use x11rb::protocol::xproto::ConnectionExt;
    let reply = conn.intern_atom(false, name.as_bytes())?.reply()?;
    Ok(reply.atom)
}

#[cfg(all(target_os = "linux", feature = "x11"))]
fn get_window_pid(
    conn: &x11rb::rust_connection::RustConnection,
    window: x11rb::protocol::xproto::Window,
    net_wm_pid: x11rb::protocol::xproto::Atom,
) -> Result<Option<u32>, Box<dyn std::error::Error>> {
    use x11rb::protocol::xproto::ConnectionExt;
    let reply = conn.get_property(
        false,
        window,
        net_wm_pid,
        x11rb::protocol::xproto::AtomEnum::CARDINAL,
        0,
        1,
    )?;
    let reply = reply.reply()?;
    if reply.format == 32 && reply.value.len() >= 4 {
        let pid = u32::from_ne_bytes([
            reply.value[0],
            reply.value[1],
            reply.value[2],
            reply.value[3],
        ]);
        Ok(Some(pid))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn release_and_recapture_ignore_absolute_cursor_jumps() {
        let mut previous = None;
        assert_eq!(
            sample_motion(&mut previous, vec2(600., 300.), true),
            Vec2::ZERO
        );
        assert_eq!(
            sample_motion(&mut previous, vec2(620., 290.), true),
            vec2(20., -10.)
        );
        assert_eq!(
            sample_motion(&mut previous, vec2(10., 10.), false),
            Vec2::ZERO
        );
        assert_eq!(
            sample_motion(&mut previous, vec2(1000., 700.), true),
            Vec2::ZERO
        );
        assert_eq!(
            sample_motion(&mut previous, vec2(1002., 697.), true),
            vec2(2., -3.)
        );
    }
    #[test]
    fn right_and_up_motion_turn_right_and_up_without_flipping() {
        let (mut yaw, mut pitch) = (1., 0.);
        turn(vec2(100., -50.), &mut yaw, &mut pitch);
        assert!((yaw - 0.75).abs() < 1e-6 && (pitch - 0.125).abs() < 1e-6);
        turn(vec2(0., -100000.), &mut yaw, &mut pitch);
        assert_eq!(pitch, 1.5);
        turn(vec2(0., 100000.), &mut yaw, &mut pitch);
        assert_eq!(pitch, -1.5);
    }
    #[test]
    fn equal_mouse_distance_has_equal_turn_at_different_frame_rates() {
        let mut angles = Vec::new();
        for frames in [30, 60, 144] {
            let (mut yaw, mut pitch) = (3., 0.);
            for _ in 0..frames {
                turn(vec2(300., 80.) / frames as f32, &mut yaw, &mut pitch);
            }
            angles.push(vec2(yaw, pitch));
        }
        assert!(angles
            .windows(2)
            .all(|pair| pair[0].distance(pair[1]) < 0.0001));
    }
}
