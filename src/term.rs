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
/* Terminal settings struct */
pub struct Settings {
    pub selected_block: Blocks,
    pub search_mode: bool,
    pub search_query: String,
    pub title_style: Style,
    pub selected_style: Style,
    pub unselected_style: Style,
}
/* Terminal settings implementation */
impl Settings {
    /**
     * Create a new settings instance.
     *
     * @param  block
     * @return Settings
     */
    pub fn new(block: Blocks) -> Self {
        Self {
            selected_block: block,
            search_mode: false,
            search_query: String::new(),
            title_style: Style::default().modifier(Modifier::BOLD),
            selected_style: Style::default().fg(Color::White),
            unselected_style: Style::default().fg(Color::DarkGray),
        }
    }
    pub fn block_style(&self, block: Blocks) -> Style {
        if block == self.selected_block {
            self.selected_style
        } else {
            self.unselected_style
        }
    }
}
