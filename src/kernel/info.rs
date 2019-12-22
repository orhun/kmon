use crate::util;
use std::vec::IntoIter;

pub struct KernelInfo {
	pub current_info: Vec<String>,
	uname_output: IntoIter<Vec<String>>,
}

impl KernelInfo {
	pub fn new() -> Self {
		let mut kernel_info = Self {
			current_info: Vec::new(),
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

	fn get_infos() -> IntoIter<Vec<String>> {
		vec![
			vec![
				String::from("Kernel Release"),
				util::exec_cmd("uname", &["-srn"]).unwrap(),
			],
			vec![
				String::from("Kernel Version"),
				util::exec_cmd("uname", &["-v"]).unwrap(),
			],
			vec![
				String::from("Kernel Platform"),
				util::exec_cmd("uname", &["-om"]).unwrap(),
			],
		]
		.into_iter()
	}
}
