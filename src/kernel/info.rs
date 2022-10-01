use crate::util;
use std::vec::IntoIter;

/* Kernel and system information */
pub struct KernelInfo {
	pub current_info: Vec<String>,
	uname_output: IntoIter<Vec<String>>,
}

impl Default for KernelInfo {
	fn default() -> Self {
		Self::new()
	}
}

impl KernelInfo {
	/**
	 * Create a new kernel info instance.
	 *
	 * @return KernelInfo
	 */
	pub fn new() -> Self {
		let mut kernel_info = Self {
			current_info: Vec::new(),
			uname_output: Vec::new().into_iter(),
		};
		kernel_info.refresh();
		kernel_info
	}

	/* Refresh the kernel information fields. */
	pub fn refresh(&mut self) {
		self.uname_output = KernelInfo::get_infos();
		self.next();
	}

	/**
	 * Select the next 'uname' output as kernel information.
	 */
	pub fn next(&mut self) {
		match self.uname_output.next() {
			Some(v) => self.current_info = v,
			None => self.refresh(),
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
				util::exec_cmd("uname", &["-srn"])
					.unwrap_or_else(|_| String::from("?")),
			],
			vec![
				String::from("Kernel Version"),
				util::exec_cmd("uname", &["-v"])
					.unwrap_or_else(|_| String::from("?")),
			],
			vec![
				String::from("Kernel Platform"),
				util::exec_cmd("uname", &["-om"])
					.unwrap_or_else(|_| String::from("?")),
			],
		]
		.into_iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_info() {
		let mut kernel_info = KernelInfo::default();
		for _x in 0..kernel_info.uname_output.len() + 1 {
			kernel_info.next();
		}
		assert_eq!("Kernel Release", kernel_info.current_info[0]);
		assert_eq!(
			util::exec_cmd("uname", &["-srn"]).unwrap(),
			kernel_info.current_info[1]
		);
	}
}
