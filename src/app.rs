use crate::event::Event;
use enum_unitary::enum_unitary;
use std::sync::mpsc::Sender;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Frame;

/* Terminal block widgets enumerator */
enum_unitary! {
    #[derive(PartialEq)]
    pub enum Blocks {
        SearchInput,
        ModuleTable,
        ModuleInfo,
        Activities,
    }
}

/* Terminal application struct */
pub struct App {
    pub selected_block: Blocks,
    pub search_mode: bool,
    pub search_query: String,
    pub table_header: [&'static str; 3],
    pub title_style: Style,
    pub selected_style: Style,
    pub unselected_style: Style,
}

/* Terminal application implementation */
impl App {
    /**
     * Create a new app instance.
     *
     * @param  block
     * @return App
     */
    pub fn new(block: Blocks) -> Self {
        Self {
            selected_block: block,
            search_mode: false,
            search_query: String::new(),
            table_header: ["Module", "Size", "Used by"],
            title_style: Style::default().modifier(Modifier::BOLD),
            selected_style: Style::default().fg(Color::White),
            unselected_style: Style::default().fg(Color::DarkGray),
        }
    }

    /**
     * Get style depending on the selected state of the block.
     *
     * @param  block
     * @return Style
     */
    pub fn block_style(&self, block: Blocks) -> Style {
        if block == self.selected_block {
            self.selected_style
        } else {
            self.unselected_style
        }
    }

    pub fn draw_search_input<B>(&self, frame: &mut Frame<B>, layout: Rect, tx: &Sender<Event<Key>>)
    where
        B: Backend,
    {
        Paragraph::new([Text::raw(self.search_query.to_string())].iter())
            .block(
                Block::default()
                    .title_style(self.title_style)
                    .border_style(match self.selected_block {
                        Blocks::SearchInput => {
                            if !self.search_mode {
                                tx.send(Event::Input(Key::Char('\n'))).unwrap();
                            }
                            self.selected_style
                        }
                        _ => self.unselected_style,
                    })
                    .borders(Borders::ALL)
                    .title("Search"),
            )
            .render(frame, layout);
    }
}

/**
 * Unit tests.
 */
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app() {
        let app = App::new(Blocks::ModuleTable);
        assert_eq!(app.selected_style, app.block_style(Blocks::ModuleTable));
        assert_eq!(app.unselected_style, app.block_style(Blocks::Activities));
    }
}
