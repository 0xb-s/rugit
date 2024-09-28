// src/tui/help_view.rs

use crate::utils::print_info;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct HelpView {
    pub visible: bool,
}

impl HelpView {
    pub fn new() -> HelpView {
        HelpView { visible: false }
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        if self.visible {
            let help_text = vec![
                "Help - Available Commands",
                "",
                "Navigation:",
                "  - Tab        : Switch between views",
                "  - q          : Exit application",
                "",
                "Status View:",
                "  - a          : Stage a file",
                "",
                "Log View:",
                "  - r          : Refresh commit logs",
                "",
                "Branch View:",
                "  - Up/Down    : Navigate branches",
                "  - c          : Create a new branch",
                "  - d          : Delete the selected branch",
                "",
                "Commit View:",
                "  - c          : Write a commit message",
                "",
                "General:",
                "  - Esc        : Cancel current operation",
                "",
                "Press 'h' again to hide this help.",
            ];
            let paragraph = Paragraph::new(help_text.join("\n"))
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(tui::layout::Alignment::Left);
            f.render_widget(paragraph, area);
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('h') {
            self.visible = !self.visible;
            if self.visible {
                print_info("Help view opened.");
            } else {
                print_info("Help view closed.");
            }
        }
    }

    pub fn update(&mut self) {
//todo
    }
}
