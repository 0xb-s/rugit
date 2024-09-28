

pub mod status_view;

use crossterm::event::KeyEvent;

/// Trait defining the behavior of a view.
pub trait View {
    fn render<B: tui::backend::Backend>(&mut self, f: &mut tui::Frame<B>, area: tui::layout::Rect);
    fn handle_input(&mut self, key: KeyEvent, messages: &mut Vec<String>);
    fn update(&mut self);
}
