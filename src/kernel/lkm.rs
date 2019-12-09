use crate::util::{exec_cmd, ScrollDirection};
use bytesize::ByteSize;

/* Loaded kernel modules */
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
     * @param  args
     * @return KernelModules
     */
    pub fn new(args: &clap::ArgMatches) -> Self {
        let mut module_list: Vec<Vec<String>> = Vec::new();
        /* Set the command for reading kernel modules and execute it. */
        let mut module_read_cmd = String::from("cat /proc/modules");
        if let Some(matches) = args.subcommand_matches("sort") {
            if matches.is_present("size") {
                module_read_cmd += " | sort -n -r -t ' ' -k2";
            } else {
                module_read_cmd += " | sort -t ' ' -k1";
            }
        }
        let modules_content =
            exec_cmd("sh", &["-c", &module_read_cmd]).expect("failed to read /proc/modules");
        /* Parse content for module name, size and related information. */
        for line in modules_content.lines() {
            let columns: Vec<&str> = line.split_whitespace().collect();
            let mut module_name = columns[0].to_string();
            if columns.len() >= 7 {
                module_name = format!("{} {}", module_name, columns[6]);
            }
            let mut used_modules = format!("{} {}", columns[2], columns[3]);
            if used_modules.chars().last().unwrap() == ',' {
                used_modules.pop();
            }
            let module_size = ByteSize::b(columns[1].to_string().parse().unwrap()).to_string();
            module_list.push(vec![module_name, module_size, used_modules]);
        }
        /* Return kernel modules. */
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::App;
    #[test]
    fn test_kernel_modules() {
        let matches = App::new("test").get_matches();
        let mut kernel_modules = KernelModules::new(&matches);
        kernel_modules.scroll_list(ScrollDirection::Top);
        assert_eq!(0, kernel_modules.index);
        assert_ne!(0, kernel_modules.default_list.len());
        assert_ne!(0, kernel_modules.current_name.len());
        assert_ne!(0, kernel_modules.current_info.len());
    }
}
