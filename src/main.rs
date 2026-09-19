use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

struct AppState {
    repo: git2::Repository,
    branches: Vec<String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            repo: git2::Repository::open(".").expect("could not open repo"),
            branches: vec![],
        }
    }

    fn load_branches(&mut self) -> anyhow::Result<()> {
        self.branches.clear();

        let branches = self.repo.branches(None)?;
        for b in branches.flatten() {
            if let Some(name) = b.0.name()? {
                self.branches.push(name.into());
            }
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize the terminal and enter raw mode automatically
    let mut terminal = ratatui::init();

    let mut app_state = AppState::new();

    // Run the main application loop
    let _app_result = run_app(&mut terminal, &mut app_state)?;

    // Restore the terminal to its normal state
    ratatui::restore();

    Ok(())
}

fn run_app(terminal: &mut DefaultTerminal, app_state: &mut AppState) -> anyhow::Result<()> {
    app_state.load_branches()?;

    loop {
        // 1. Draw the UI frame
        terminal.draw(|frame| ui(frame, app_state))?;

        // 2. Handle interactive keyboard events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Ignore key release events to avoid double-counting on Windows
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        _x => {
                            println!("invalid key  {_x}");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn ui(frame: &mut Frame, app_state: &AppState) {
    // Split the screen area layout
    let counter = 0;
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(frame.area());

    // Create a styled block container for the counter
    let main_block = Block::default()
        .title("Branches ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Construct the center text
    let text = format!("\nCounter Value: {}", counter);
    let paragraph = Paragraph::new(text)
        .block(main_block)
        .alignment(Alignment::Center)
        .style(Style::default().add_modifier(Modifier::BOLD));

    // Render the main paragraph
    frame.render_widget(paragraph, chunks[0]);

    // Render a small instruction footer
    let instructions = Paragraph::new("Use [Up/Down] arrows to adjust, press [q] to quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(instructions, chunks[1]);
}
