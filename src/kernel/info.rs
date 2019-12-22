use crate::util;
use std::vec::IntoIter;

/* Kernel and system information */
pub struct KernelInfo {
	pub current_info: Vec<String>,
	uname_output: IntoIter<Vec<String>>,
}

impl Default for KernelInfo {
	/**
	 * Create a new kernel info instance.
	 *
	 * @return KernelInfo
	 */
	fn default() -> Self {
		let mut kernel_info = Self {
			current_info: Vec::new(),
			uname_output: KernelInfo::get_infos(),
		};
		kernel_info.next();
		kernel_info
	}
}

impl KernelInfo {
	/**
	 * Select the next 'uname' output as kernel information.
	 */
	pub fn next(&mut self) {
		match self.uname_output.next() {
			Some(v) => self.current_info = v,
			None => {
				self.uname_output = KernelInfo::get_infos();
				self.next();
			}
		}
	}

	/**
	 * Execute 'uname' command and return its output along with its description.
	 *
	 * @return Iterator
	 */
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
