//! looking-glass-tui - TUI interface for Looking Glass launcher

pub mod app;
pub mod ui;
pub mod events;

pub use app::TuiApp;
pub use ui::{render_game_list, render_game_detail, render_help};
pub use events::{handle_events, EventHandler};

mod app {
    use super::*;
    use ratatui::Terminal;
    use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
    use crossterm::execute;
    use std::io::stdout;

    pub struct TuiApp {
        terminal: Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    }

    impl TuiApp {
        pub fn new() -> anyhow::Result<Self> {
            enable_raw_mode()?;
            let mut stdout = stdout();
            execute!(stdout, EnterAlternateScreen)?;
            let backend = ratatui::backend::CrosstermBackend::new(stdout);
            let terminal = Terminal::new(backend)?;
            Ok(Self { terminal })
        }

        pub fn run(&mut self) -> anyhow::Result<()> {
            loop {
                self.terminal.draw(|f| {
                    // TODO: Render UI
                })?;
                // TODO: Handle events
            }
        }
    }

    impl Drop for TuiApp {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen);
        }
    }
}

mod ui {
    use ratatui::{layout::{Layout, Constraint, Direction}, style::{Style, Color}, widgets::{Block, Borders, List, ListItem, Paragraph}};

    pub fn render_game_list(_frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
        // TODO: Implement game list rendering
    }

    pub fn render_game_detail(_frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
        // TODO: Implement game detail rendering
    }

    pub fn render_help(_frame: &mut ratatui::Frame, _area: ratatui::layout::Rect) {
        // TODO: Implement help rendering
    }
}

mod events {
    use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers};

    pub struct EventHandler;

    impl EventHandler {
        pub fn handle(&mut self, event: Event) -> anyhow::Result<Option<Action>> {
            match event {
                Event::Key(key) => self.handle_key(key),
                _ => Ok(None),
            }
        }

        fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
            match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Ok(Some(Action::Quit)),
                KeyCode::Char('j') | KeyCode::Down => Ok(Some(Action::Down)),
                KeyCode::Char('k') | KeyCode::Up => Ok(Some(Action::Up)),
                KeyCode::Enter => Ok(Some(Action::Select)),
                _ => Ok(None),
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Action {
        Quit,
        Up,
        Down,
        Select,
        Back,
        Search,
        Refresh,
    }

    pub fn handle_events() -> anyhow::Result<()> {
        // TODO: Implement event loop
        Ok(())
    }
}