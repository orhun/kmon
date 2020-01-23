use crate::app::ScrollDirection;
use crate::util;

/* Kernel activity logs */
#[derive(Clone, Default)]
pub struct KernelLogs {
	pub output: String,
	pub selected_output: String,
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
				.unwrap_or_else(|_| String::from("failed to retrieve dmesg output"));
		let logs_updated =
			self.output.lines().next().unwrap_or_default() != self.last_line;
		self.last_line = self.output.lines().next().unwrap_or_default().to_string();
		logs_updated
	}

	pub fn refresh(&mut self) {
		self.last_line = String::new();
		self.index = 0;
		self.update();
	}

	/**
	 * Select a part of the output depending on the area properties.
	 *
	 * @param  area_height
	 * @param  area_sub
	 * @return selected_output
	 */
	pub fn select(&mut self, area_height: u16, area_sub: u16) -> &str {
		self.selected_output = self
			.output
			.lines()
			.skip(
				area_height
					.checked_sub(area_sub)
					.and_then(|height| {
						(self.output.lines().count() - self.index)
							.checked_sub(height as usize)
					})
					.unwrap_or(0),
			)
			.map(|i| format!("{}\n", i))
			.collect::<String>();
		&self.selected_output
	}

	/**
	 * Scroll the kernel logs up/down.
	 *
	 * @param direction
	 * @param smooth_scroll
	 */
	pub fn scroll(&mut self, direction: ScrollDirection, smooth_scroll: bool) {
		let scroll_amount = if smooth_scroll { 1 } else { 3 };
		match direction {
			ScrollDirection::Up => {
				if self.index + scroll_amount <= self.output.lines().count() {
					self.index += scroll_amount;
				}
			}
			ScrollDirection::Down => {
				if self.index > scroll_amount - 1 {
					self.index -= scroll_amount;
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
		for direction in ScrollDirection::iter().rev().chain(ScrollDirection::iter())
		{
			kernel_logs.scroll(*direction, *direction == ScrollDirection::Top);
		}
		assert_eq!(true, kernel_logs.update());
		assert_ne!(0, kernel_logs.output.lines().count());
		assert_ne!(0, kernel_logs.select(10, 2).len());
	}
}
