//! Co-op entry and personal menus. Connected-session controls are drawn one
//! frame at a time so opening a menu never stops the partner's simulation.
use super::{invite::Invite, preview::Launch, protocol::Visit};
use crate::{
    assets::Assets,
    input::Input,
    menu::Menu,
    preferences::Preferences,
    ui::{Canvas, Ui, PAPER_TEXT},
    Options,
};
use macroquad::prelude::*;

#[derive(Default)]
struct LinkInput {
    text: String,
    selected: bool,
}
impl LinkInput {
    fn replace(&mut self, text: &str) {
        // Bound clipboard input before storing it. Invite::parse is still the
        // authority; do not truncate an overlong link into a different invite.
        self.text = if text.len() <= 2048 {
            text.trim().to_owned()
        } else {
            String::new()
        };
        self.selected = false;
    }
    fn character(&mut self, c: char) {
        if !c.is_ascii_graphic() {
            return;
        }
        if self.selected {
            self.text.clear();
            self.selected = false;
        }
        if self.text.len() < 2048 {
            self.text.push(c);
        }
    }
    fn backspace(&mut self) {
        if self.selected {
            self.text.clear();
        } else {
            self.text.pop();
        }
        self.selected = false;
    }
}

/// Starts a fresh co-op visit while retaining a suspended single-player game in
/// its caller. No campaign save is read, replaced or written by this menu.
pub async fn open(assets: &mut Assets, options: &Options, backdrop: &mut Menu, visit: Visit) {
    let ui = match Ui::load(assets) {
        Ok(ui) => ui,
        Err(error) => {
            backdrop.message = error.to_string();
            return;
        }
    };
    let prefs = Preferences::load();
    let mut input = Input::default();
    let mut link = LinkInput::default();
    let mut joining = false;
    let mut selected = 0;
    let mut message = String::new();
    show_mouse(false);
    // The activating main-menu key/click must not also select Host.
    next_frame().await;
    loop {
        if is_quit_requested() {
            break;
        }
        let focused = crate::look::window_focused();
        input.update(&prefs, focused);
        if focused && input.ui(KeyCode::Escape) {
            if joining {
                joining = false;
                selected = 0;
                message.clear();
            } else {
                break;
            }
        }
        if joining && focused {
            let control = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
            if control && is_key_pressed(KeyCode::A) {
                link.selected = true;
            }
            if control && is_key_pressed(KeyCode::V) {
                paste(&mut link, &mut message);
            } else if is_key_pressed(KeyCode::Backspace)
                || is_key_pressed(KeyCode::Delete) && link.selected
            {
                link.backspace();
            }
            while let Some(c) = get_char_pressed() {
                if !control {
                    link.character(c);
                }
            }
        } else {
            while get_char_pressed().is_some() {}
        }
        backdrop.backdrop();
        panel(&ui, if joining { "Join co-op" } else { "Co-op" });
        let mut launch = None;
        if joining {
            text(
                &ui,
                "Paste the complete invite link from your friend.",
                120.,
                16.,
            );
            let canvas = Canvas::new(screen_width(), screen_height());
            let field = Rect::new(106., 155., 428., 76.);
            let r = canvas.rect(field);
            draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.03, 0.02, 0.02, 0.9));
            draw_rectangle_lines(
                r.x,
                r.y,
                r.w,
                r.h,
                2.,
                if link.selected { YELLOW } else { PAPER_TEXT },
            );
            let shown: String = link.text.chars().take(156).collect();
            let shown = if shown.is_empty() {
                "Invite link".to_owned()
            } else {
                shown
            };
            // URLs have no spaces, so wrap by character without changing the input.
            for (row, chunk) in shown.as_bytes().chunks(52).take(3).enumerate() {
                ui.courier.draw_left(
                    canvas,
                    &String::from_utf8_lossy(chunk),
                    Rect::new(114., 164. + row as f32 * 20., 412., 20.),
                    14.,
                    WHITE,
                );
            }
            text(
                &ui,
                "Ctrl+V: paste   Ctrl+A: replace   Enter: join",
                240.,
                13.,
            );
            if let Some(choice) = buttons(
                &ui,
                &input,
                &mut selected,
                &["Join", "Paste link", "Back"],
                280.,
            ) {
                match choice {
                    0 => match Invite::parse(&link.text) {
                        Ok(invite) => {
                            launch = Some(Launch {
                                host: false,
                                internet: true,
                                invite_file: None,
                                invite: Some(invite),
                                start: None,
                                drive: false,
                                death_check: None,
                                scene_check: None,
                                travel_check: false,
                                conditions: Default::default(),
                            })
                        }
                        Err(error) => message = error.to_string(),
                    },
                    1 => {
                        paste(&mut link, &mut message);
                        selected = 0;
                    }
                    _ => {
                        joining = false;
                        selected = 0;
                        message.clear();
                    }
                }
            }
        } else {
            text(&ui, "Play together as blue and red Alice.", 120., 18.);
            text(
                &ui,
                "Host creates an invite. Join uses your friend's link.",
                152.,
                15.,
            );
            text(&ui, "Hosting begins at this chapter's entrance.", 182., 14.);
            if let Some(choice) =
                buttons(&ui, &input, &mut selected, &["Host", "Join", "Back"], 224.)
            {
                match choice {
                    0 => {
                        launch = Some(Launch {
                            host: true,
                            internet: true,
                            invite_file: None,
                            invite: None,
                            start: Some(visit.clone()),
                            drive: false,
                            death_check: None,
                            scene_check: None,
                            travel_check: false,
                            conditions: Default::default(),
                        })
                    }
                    1 => {
                        joining = true;
                        selected = 0;
                        message.clear();
                    }
                    _ => break,
                }
            }
            text(
                &ui,
                "Development build: campaign co-op is still in progress.",
                378.,
                12.,
            );
        }
        if !message.is_empty() {
            let canvas = Canvas::new(screen_width(), screen_height());
            ui.dialog(canvas.rect(Rect::new(86., 410., 470., 66.)));
            text(&ui, &message, 418., 13.);
        }
        if focused {
            ui.cursor();
        }
        capture();
        if let Some(launch) = launch {
            input.suppress();
            next_frame().await;
            message = match super::preview::run(assets, options, &launch).await {
                Ok(()) => "Co-op closed. You can host or join again.".into(),
                Err(error) => {
                    eprintln!("Co-op session ended: {error}");
                    format!("Could not continue co-op: {error}")
                }
            };
            // Macroquad's text queue survives gameplay frames. Movement keys
            // typed there must not be appended to the retained invite on return.
            while get_char_pressed().is_some() {}
            set_default_camera();
            set_cursor_grab(false);
            show_mouse(false);
            input.suppress();
            selected = 0;
        }
        next_frame().await;
    }
    show_mouse(true);
}

fn paste(link: &mut LinkInput, message: &mut String) {
    match miniquad::window::clipboard_get() {
        Some(value) if value.len() <= 2048 => {
            link.replace(&value);
            message.clear();
        }
        Some(_) => *message = "That invite link is too long.".into(),
        None => *message = "Copy your friend's invite link first.".into(),
    }
}

pub fn panel(ui: &Ui, title: &str) {
    let canvas = Canvas::new(screen_width(), screen_height());
    let r = canvas.rect(Rect::new(76., 56., 488., 358.));
    ui.dialog(r);
    ui.font
        .draw(canvas, title, Rect::new(100., 72., 440., 36.), 30., WHITE);
}

pub fn text(ui: &Ui, value: &str, y: f32, size: f32) {
    let canvas = Canvas::new(screen_width(), screen_height());
    let rows = crate::story::wrap(value, 424., |s| ui.body.width(s, size));
    for (i, row) in rows.iter().take(3).enumerate() {
        ui.body.draw(
            canvas,
            row,
            Rect::new(108., y + i as f32 * (size + 5.), 424., size + 5.),
            size,
            PAPER_TEXT,
        );
    }
}

pub fn buttons(
    ui: &Ui,
    input: &Input,
    selected: &mut usize,
    labels: &[&str],
    y: f32,
) -> Option<usize> {
    let focused = crate::look::window_focused();
    let canvas = Canvas::new(screen_width(), screen_height());
    let pointer = canvas.pointer(Vec2::from(mouse_position()));
    if focused && (input.ui(KeyCode::Down) || input.ui(KeyCode::Tab)) {
        *selected = (*selected + 1) % labels.len();
    }
    if focused && input.ui(KeyCode::Up) {
        *selected = (*selected + labels.len() - 1) % labels.len();
    }
    let mut click = None;
    for (index, label) in labels.iter().enumerate() {
        let rect = Rect::new(220., y + index as f32 * 46., 200., 36.);
        let hover = focused && rect.contains(pointer);
        ui.plaque(canvas.rect(rect), hover || index == *selected);
        ui.font.draw(canvas, label, rect, 24., WHITE);
        if hover && is_mouse_button_pressed(MouseButton::Left) {
            *selected = index;
            click = Some(index);
        }
    }
    if focused && input.ui(KeyCode::Enter) {
        click = Some(*selected);
    }
    click
}

pub fn capture() {
    if crate::look::window_focused() && is_key_pressed(KeyCode::F12) {
        let _ = crate::viewer::save_capture(&crate::preferences::path("coop-menu.png"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paste_replace_and_edit_keep_the_full_invite_and_bound_clipboard_input() {
        let invite = Invite::new("https://example.trycloudflare.com")
            .unwrap()
            .link();
        let mut field = LinkInput::default();
        field.replace(&format!("  {invite}\n"));
        assert_eq!(Invite::parse(&field.text).unwrap().link(), invite);
        field.selected = true;
        field.character('x');
        assert_eq!(field.text, "x");
        field.backspace();
        assert!(field.text.is_empty());
        field.replace(&"x".repeat(2049));
        assert!(field.text.is_empty());
        field.replace(&"x".repeat(2048));
        field.character('y');
        assert_eq!(field.text.len(), 2048);
        field.selected = true;
        field.backspace();
        assert!(field.text.is_empty());
    }
}
