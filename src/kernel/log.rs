use crate::app::ScrollDirection;
use crate::util;

/* Kernel activity logs */
#[derive(Clone, Default)]
pub struct KernelLogs {
	pub output: String,
	last_line: String,
	pub index: usize,
}

impl KernelLogs {
	/**
	 * Update the output variable value if 'dmesg' logs changed.
	 *
	 * @return logs_updated
	 */
	pub fn update(&mut self) -> bool {
		self.output =
			util::exec_cmd("dmesg", &["--kernel", "--human", "--color=never"])
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
				}
			}
			ScrollDirection::Down => {
				if self.index > 2 {
					self.index -= 3;
				} else {
					self.index = 0;
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
		let mut kernel_logs = KernelLogs::default();
		assert_eq!(true, kernel_logs.update());
		assert_ne!(0, kernel_logs.output.lines().count());
	}
}
