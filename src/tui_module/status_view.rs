// src/tui/status_view.rs

use crate::utils::print_info;
use crossterm::event::{KeyCode, KeyEvent};
use git2::{Repository as GitRepo, StatusOptions};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct StatusView {
    pub items: Vec<String>,
}

impl StatusView {
    pub fn new() -> StatusView {
        StatusView { items: vec![] }
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| ListItem::new(i.clone()))
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        f.render_widget(list, area);
    }

    // Update the function signature to use crossterm::event::KeyEvent
    pub fn handle_input(&mut self, key: KeyEvent) {
        // Handle inputs specific to the Status view if needed
        // Example: Press 'a' to add files, etc.
        match key.code {
            KeyCode::Char('a') => {
                // Implement file staging logic
                print_info("Add functionality not yet implemented.");
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.items.clear();
        match GitRepo::open(".") {
            Ok(repo) => {
                let mut opts = StatusOptions::new();
                opts.include_untracked(true)
                    .renames_head_to_index(true)
                    .renames_index_to_workdir(true);

                match repo.statuses(Some(&mut opts)) {
                    Ok(statuses) => {
                        if statuses.is_empty() {
                            self.items
                                .push("Nothing to commit, working tree clean.".to_string());
                        } else {
                            for entry in statuses.iter() {
                                let status = entry.status();
                                let file_path = entry.path().unwrap_or("Unknown");

                                let status_str = match status {
                                    s if s.is_index_new() => "A",
                                    s if s.is_index_modified() => "M",
                                    s if s.is_index_deleted() => "D",
                                    s if s.is_wt_new() => "??",
                                    s if s.is_wt_modified() => "M",
                                    s if s.is_wt_deleted() => "D",
                                    _ => " ",
                                };

                                self.items.push(format!("{} {}", status_str, file_path));
                            }
                        }
                    }
                    Err(e) => {
                        self.items.push(format!("Error retrieving status: {}", e));
                    }
                }
            }
            Err(e) => {
                self.items.push(format!("Error opening repository: {}", e));
            }
        }
    }
}
