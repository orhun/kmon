use crate::util::exec_cmd;

/* Scrolling directions enumerator */
pub enum ScrollDirection {
    Up,
    Down,
    Top,
    Bottom,
}

/* Kernel modules struct and implementation */
pub struct KernelModules {
    pub default_list: Vec<Vec<String>>,
    pub list: Vec<Vec<String>>,
    pub current_name: String,
    pub current_info: String,
    pub index: usize,
    pub info_scroll_offset: u16,
}
impl KernelModules {
    /**
     * Create a new kernel modules instance.
     *
     * @param  list
     * @return KernelModules
     */
    pub fn new(module_list: Vec<Vec<String>>) -> Self {
        Self {
            default_list: module_list.clone(),
            list: module_list,
            current_name: String::new(),
            current_info: String::new(),
            index: 0,
            info_scroll_offset: 0,
        }
    }
    /**
     * Scroll module list and select module.
     *
     * @param direction
     */
    pub fn scroll_list(&mut self, direction: ScrollDirection) {
        self.info_scroll_offset = 0;
        if self.list.len() == 0 {
            self.index = 0;
        } else {
            match direction {
                ScrollDirection::Up => self.previous_module(),
                ScrollDirection::Down => self.next_module(),
                ScrollDirection::Top => self.index = 0,
                ScrollDirection::Bottom => self.index = self.list.len() - 1,
            }
            self.current_name = self.list[self.index][0]
                .split_whitespace()
                .next()
                .unwrap()
                .to_string();
            self.current_info = exec_cmd("modinfo", &[&self.current_name]).unwrap();
        }
    }
    /**
     * Select the next module.
     */
    pub fn next_module(&mut self) {
        self.index += 1;
        if self.index > self.list.len() - 1 {
            self.index = 0;
        }
    }
    /**
     * Select the previous module.
     */
    pub fn previous_module(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.list.len() - 1;
        }
    }
    /**
     * Scroll the module information text.
     *
     * @param direction
     */
    pub fn scroll_mod_info(&mut self, direction: ScrollDirection) {
        match direction {
            ScrollDirection::Up => {
                if self.info_scroll_offset > 1 {
                    self.info_scroll_offset -= 2;
                }
            }
            ScrollDirection::Down => {
                if self.current_info.lines().count() > 0 {
                    self.info_scroll_offset += 2;
                    self.info_scroll_offset %= (self.current_info.lines().count() as u16) * 2;
                }
            }
            _ => {}
        }
    }
}