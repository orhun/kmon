use crate::app::ScrollDirection;
use crate::util;

/* Kernel activity logs */
pub struct KernelLogs {
	pub output: String,
	last_line: String,
	pub version: String,
	pub scroll_offset: u16,
}

impl KernelLogs {
	/**
	 * Create a new kernel logs instance.
	 *
	 * @return KernelLogs
	 */
	pub fn new() -> Self {
		Self {
			output: String::new(),
			last_line: String::new(),
			version: util::exec_cmd("uname", &["-srm"]).unwrap(),
			scroll_offset: 0,
		}
	}

	/**
	 * Update the output variable value if 'dmesg' logs changed.
	 *
	 * @return logs_updated
	 */
	pub fn update(&mut self) -> bool {
		self.output = util::exec_cmd(
			"sh",
			&["-c", "dmesg --kernel --human --color=never | tac"],
		)
		.expect("failed to retrieve dmesg output");
		let logs_updated = self.output.lines().next().unwrap() != self.last_line;
		self.last_line = self.output.lines().next().unwrap().to_string();
		logs_updated
	}

	/**
	 * Scroll the kernel logs up/down.
	 *
	 * @param direction
	 */
	pub fn scroll(&mut self, direction: ScrollDirection) {
		match direction {
			ScrollDirection::Up => {
				if self.scroll_offset > 2 {
					self.scroll_offset -= 3;
				}
			}
			ScrollDirection::Down => {
				if !self.output.is_empty() {
					self.scroll_offset += 3;
					self.scroll_offset %= (self.output.lines().count() as u16) * 2;
				}
			}
			_ => {}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_kernel_logs() {
		let mut kernel_logs = KernelLogs::new();
		assert_eq!(true, kernel_logs.update());
		assert_ne!(0, kernel_logs.output.lines().count());
	}
}
