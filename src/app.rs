use enum_unitary::enum_unitary;
use tui::style::{Color, Modifier, Style};

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
