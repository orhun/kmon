use enum_unitary::enum_unitary;

/* Terminal block widgets enumerator */
enum_unitary! {
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
}
/* Terminal settings implementation */
impl Settings {
    pub fn new(block: Blocks) -> Self {
        Self {
            selected_block: block,
            search_mode: false,
            search_query: String::new(),
        }
    }
}