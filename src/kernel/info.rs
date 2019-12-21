use crate::util;

pub struct KernelInfo {
	pub current_info: String,
    uname_output: std::vec::IntoIter<String>,
}

impl KernelInfo {
    pub fn new() -> Self {
        let mut kernel_info = Self {
            current_info: String::new(),
            uname_output: KernelInfo::get_infos(),
        };
        kernel_info.next();
        kernel_info
    }

    pub fn next(&mut self) {
        match self.uname_output.next() {
            Some(v) => self.current_info = v,
            None => {
                self.uname_output = KernelInfo::get_infos();
                self.next();
            }
        }
    }

    fn get_infos() -> std::vec::IntoIter<String> {
        vec![util::exec_cmd("uname", &["-srm"]).unwrap(),
				util::exec_cmd("uname", &["-v"]).unwrap(),
				util::exec_cmd("uname", &["-opi"]).unwrap()].into_iter()
    }
}