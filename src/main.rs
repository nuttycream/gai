use color_eyre::{Result, eyre::Ok};
use crossterm::event::{self, Event};
use git2::Repository;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
};

struct AppState {
    repo: Repository,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let repo = Repository::open(".")?;

    let mut state = AppState { repo };

    let terminal = ratatui::init();
    let result = run(terminal, &mut state);

    ratatui::restore();

    result
}

fn run(mut terminal: DefaultTerminal, app_state: &mut AppState) -> Result<()> {
    loop {
        terminal.draw(|f| render(f))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Esc => break Ok(()),
                event::KeyCode::Char('q' | 'Q') => break Ok(()),
                _ => {}
            }
        }
    }
}

fn render(frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    frame.render_widget(
        Paragraph::new("something").block(Block::new().title("gai").borders(Borders::ALL)),
        layout[0],
    );
    frame.render_widget(
        Paragraph::new("foo").block(Block::new().title("status").borders(Borders::ALL)),
        layout[1],
    );
}
