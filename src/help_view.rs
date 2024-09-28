

use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct HelpView;

impl HelpView {
    pub fn new() -> HelpView {
        HelpView
    }

    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let help_text = vec![
            "Help Menu".to_string(),
            "".to_string(),
            "General:".to_string(),
            "  - q: Quit application".to_string(),
            "  - Tab: Switch between views".to_string(),
            "".to_string(),
            "Status View:".to_string(),
            "  - a: Add files to staging".to_string(),
            "".to_string(),
            "Log View:".to_string(),
            "  - r: Refresh commit logs".to_string(),
            "".to_string(),
            "Branch View:".to_string(),
            "  - c: Create a new branch".to_string(),
            "  - d: Delete the selected branch".to_string(),
            "  - Up/Down: Navigate branches".to_string(),
            "".to_string(),
            "Commit View:".to_string(),
            "  - c: Write a commit message".to_string(),
            "".to_string(),
        ];

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(help_text.join("\n"))
            .block(block)
            .style(Style::default().fg(Color::White))
            .alignment(tui::layout::Alignment::Left)
            .wrap(tui::widgets::Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    pub fn handle_input(&self, _key: KeyEvent) {}
}
