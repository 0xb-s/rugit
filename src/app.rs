// src/app.rs

use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame,
};

use crate::tui_module::{
    branch_view::BranchView, commit_view::CommitView, help_view::HelpView, log_view::LogView,
    status_view::StatusView,
};

pub struct App {
    pub active_view: ActiveView,
    pub status_view: StatusView,
    pub log_view: LogView,
    pub branch_view: BranchView,
    pub commit_view: CommitView,
    pub help_view: HelpView,
    pub messages: Vec<String>, 
}

#[derive(PartialEq, Debug)]
pub enum ActiveView {
    Status,
    Log,
    Branch,
    Commit,
    Help,
}

impl App {
    pub fn new() -> App {
        App {
            active_view: ActiveView::Status,
            status_view: StatusView::new(),
            log_view: LogView::new(),
            branch_view: BranchView::new(),
            commit_view: CommitView::new(),
            help_view: HelpView::new(),
            messages: Vec::new(),
        }
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        // Define the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3), // Title
                    Constraint::Min(1),    // Main Content
                    Constraint::Length(5), // Messages
                    Constraint::Length(3), // Footer
                ]
                .as_ref(),
            )
            .split(f.size());

        // Render the title
        let title = tui::widgets::Paragraph::new("Rugit-TUI: Rust Git Interface")
            .style(tui::style::Style::default().fg(tui::style::Color::Yellow))
            .alignment(tui::layout::Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Render the main content based on the active view
        match self.active_view {
            ActiveView::Status => self.status_view.render(f, chunks[1]),
            ActiveView::Log => self.log_view.render(f, chunks[1]),
            ActiveView::Branch => self.branch_view.render(f, chunks[1]),
            ActiveView::Commit => self.commit_view.render(f, chunks[1]),
            ActiveView::Help => self.help_view.render(f, chunks[1]),
        }

        // Render the messages
        let messages_text = self.messages.join("\n");
        let messages = tui::widgets::Paragraph::new(messages_text)
            .block(Block::default().borders(Borders::ALL).title("Messages"))
            .style(tui::style::Style::default().fg(tui::style::Color::Magenta))
            .alignment(tui::layout::Alignment::Left)
            .wrap(tui::widgets::Wrap { trim: true });
        f.render_widget(messages, chunks[2]);

        // Render the footer
        let footer = tui::widgets::Paragraph::new("Press 'q' to exit | Tab to switch views")
            .style(tui::style::Style::default().fg(tui::style::Color::Magenta))
            .alignment(tui::layout::Alignment::Center);
        f.render_widget(footer, chunks[3]);
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> bool {
        if key.code == KeyCode::Char('q') {
            return true;
        }

        if key.code == KeyCode::Tab {
            self.switch_view();
            return false;
        }

        match self.active_view {
            ActiveView::Status => {
                // if let Err(e) = self.status_view.handle_input(key) {
                //     self.messages.push(format!("Error: {}", e));
                // }
            }
            ActiveView::Log => {
                if let Err(e) = self.log_view.handle_input(key, &mut self.messages) {
                    self.messages.push(format!("Error: {}", e));
                }
            }
            ActiveView::Branch => {
                if let Err(e) = self.branch_view.handle_input(key, &mut self.messages) {
                    self.messages.push(format!("Error: {}", e));
                }
            }
            ActiveView::Commit => {
                if let Err(e) = self.commit_view.handle_input(key, &mut self.messages) {
                    self.messages.push(format!("Error: {}", e));
                }
            }
            ActiveView::Help => {
                self.help_view.handle_input(key);
            }
        }

        false
    }

    fn switch_view(&mut self) {
        self.active_view = match self.active_view {
            ActiveView::Status => ActiveView::Log,
            ActiveView::Log => ActiveView::Branch,
            ActiveView::Branch => ActiveView::Commit,
            ActiveView::Commit => ActiveView::Help,
            ActiveView::Help => ActiveView::Status,
        };
        self.messages
            .push(format!("Switched to {:?}", self.active_view));
    }

    pub fn on_tick(&mut self) {
        match self.active_view {
            ActiveView::Status => self.status_view.update(),
            ActiveView::Log => self.log_view.update(),
            ActiveView::Branch => self.branch_view.update(),
            ActiveView::Commit => {}
            ActiveView::Help => {}
        }
    }
}
