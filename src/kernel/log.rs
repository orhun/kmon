use crate::app::ScrollDirection;
use crate::util;

/* Kernel activity logs */
pub struct KernelLogs {
	pub output: String,
	last_line: String,
	pub index: usize,
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
			index: 0,
		}
	}

	/**
	 * Update the output variable value if 'dmesg' logs changed.
	 *
	 * @return logs_updated
	 */
	pub fn update(&mut self) -> bool {
		self.output =
			util::exec_cmd("sh", &["-c", "dmesg --kernel --human --color=never"])
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
				if self.index + 3 <= self.output.lines().count() {
					self.index += 3;
				} else {
					self.index = self.output.lines().count();
				}
			}
			ScrollDirection::Down => {
				if self.index > 2 {
					self.index -= 3;
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
