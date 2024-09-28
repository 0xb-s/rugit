

use crate::git_utils::commit_changes;
use crate::utils::{print_error, print_info};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
#[derive(Debug)]
pub struct CommitView {
    pub input_mode: InputMode,
    pub commit_message: String,
}

#[derive(PartialEq, Debug)]
pub enum InputMode {
    Normal,
    WritingCommit,
}

impl CommitView {
    pub fn new() -> CommitView {
        CommitView {
            input_mode: InputMode::Normal,
            commit_message: String::new(),
        }
    }

    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        match self.input_mode {
            InputMode::Normal => {
                let block = Block::default().borders(Borders::ALL).title("Commit");
                let paragraph = Paragraph::new("Press 'c' to write a commit message.")
                    .block(block)
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(tui::layout::Alignment::Left);
                f.render_widget(paragraph, area);
            }
            InputMode::WritingCommit => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title("Enter Commit Message")
                    .style(Style::default().fg(Color::Green));
                let paragraph = Paragraph::new(&self.commit_message[..])
                    .block(block)
                    .style(Style::default().fg(Color::White))
                    .alignment(tui::layout::Alignment::Left);
                f.render_widget(Clear, area); // Clear the area before rendering the input
                f.render_widget(paragraph, area);
            }
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent, messages: &mut Vec<String>) -> Result<()> {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('c') => {
                    self.input_mode = InputMode::WritingCommit;
                    self.commit_message.clear();
                    messages.push("Enter your commit message below.".to_string());
                }
                _ => {}
            },
            InputMode::WritingCommit => match key.code {
                KeyCode::Enter => {
                    let message = self.commit_message.trim();
                    if message.is_empty() {
                        messages.push("Commit message cannot be empty.".to_string());
                    } else {
                        match commit_changes(".", message) {
                            Ok(_) => {
                                messages.push(format!("Committed with message: '{}'", message))
                            }
                            Err(e) => messages.push(format!("Failed to commit: {}", e)),
                        }
                        self.input_mode = InputMode::Normal;
                        self.commit_message.clear();
                    }
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.commit_message.clear();
                    messages.push("Commit cancelled.".to_string());
                }
                KeyCode::Char(c) => {
                    self.commit_message.push(c);
                }
                KeyCode::Backspace => {
                    self.commit_message.pop();
                }
                _ => {}
            },
        }
        Ok(())
    }
}
