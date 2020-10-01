use crate::app::ScrollDirection;
use crate::kernel::cmd::{Command, ModuleCommand};
use crate::style::{Style, StyledText, Symbol};
use crate::util;
use bytesize::ByteSize;
use clap::ArgMatches;
use std::slice::Iter;
use tui::widgets::Text;

/* Type of the sorting of module list */
#[derive(Clone, Copy, Debug)]
enum SortType {
	None,
	Size,
	Name,
	Dependent,
}

impl SortType {
	/**
	 * Return iterator for the sort types.
	 *
	 * @return Iter
	 */
	#[allow(dead_code)]
	pub fn iter() -> Iter<'static, SortType> {
		[
			SortType::None,
			SortType::Size,
			SortType::Name,
			SortType::Dependent,
		]
		.iter()
	}
}

/* Listing properties of module list */
pub struct ListArgs {
	sort: SortType,
	reverse: bool,
}

impl ListArgs {
	/**
	 * Create a new list arguments instance.
	 *
	 * @param  ArgMatches
	 * @return ListArgs
	 */
	pub fn new(args: &ArgMatches<'_>) -> Self {
		let mut sort_type = SortType::None;
		if let Some(matches) = args.subcommand_matches("sort") {
			if matches.is_present("size") {
				sort_type = SortType::Size;
			} else if matches.is_present("dependent") {
				sort_type = SortType::Dependent;
			} else {
				sort_type = SortType::Name;
			}
		}
		Self {
			sort: sort_type,
			reverse: args.is_present("reverse"),
		}
	}
}

/* Loadable kernel modules */
pub struct KernelModules<'a> {
	pub default_list: Vec<Vec<String>>,
	pub list: Vec<Vec<String>>,
	pub current_name: String,
	pub current_info: StyledText<'a>,
	pub command: ModuleCommand,
	pub index: usize,
	pub info_scroll_offset: usize,
	pub style: Style,
	pub args: ListArgs,
}

impl KernelModules<'_> {
	/**
	 * Create a new kernel modules instance.
	 *
	 * @param  ListArgs
	 * @param  Style
	 * @return KernelModules
	 */
	pub fn new(args: ListArgs, style: Style) -> Self {
		let mut kernel_modules = Self {
			default_list: Vec::new(),
			list: Vec::new(),
			current_name: String::new(),
			current_info: StyledText::default(),
			command: ModuleCommand::None,
			index: 0,
			info_scroll_offset: 0,
			args,
			style,
		};
		kernel_modules.refresh();
		kernel_modules
	}

	/* Parse kernel modules from '/proc/modules'. */
	pub fn refresh(&mut self) {
		let mut module_list: Vec<Vec<String>> = Vec::new();
		/* Set the command for reading kernel modules and execute it. */
		let mut module_read_cmd = String::from("cat /proc/modules");
		match self.args.sort {
			SortType::Size => module_read_cmd += " | sort -n -r -t ' ' -k2",
			SortType::Name => module_read_cmd += " | sort -t ' ' -k1",
			SortType::Dependent => module_read_cmd += " | sort -n -r -t ' ' -k3",
			_ => {}
		}
		let modules_content = util::exec_cmd("sh", &["-c", &module_read_cmd])
			.expect("failed to read /proc/modules");
		/* Parse content for module name, size and related information. */
		for line in modules_content.lines() {
			let columns: Vec<&str> = line.split_whitespace().collect();
			let mut module_name = format!(" {}", columns[0].to_string());
			if columns.len() >= 7 {
				module_name = format!("{} {}", module_name, columns[6]);
			}
			let mut used_modules = format!("{} {}", columns[2], columns[3]);
			if used_modules.ends_with(',') {
				used_modules.pop();
			}
			let module_size =
				ByteSize::b(columns[1].to_string().parse().unwrap_or(0)).to_string();
			module_list.push(vec![module_name, module_size, used_modules]);
		}
		/* Reverse the kernel modules if the argument is provided. */
		if self.args.reverse {
			module_list.reverse();
		}
		self.default_list = module_list.clone();
		self.list = module_list;
		self.scroll_list(ScrollDirection::Top);
	}

	/**
	 * Get the current command using current module name.
	 *
	 * @return Command
	 */
	pub fn get_current_command(&self) -> Command {
		self.command.get(&self.current_name)
	}

	/**
	 * Set the current module command and show confirmation message.
	 *
	 * @param module_command
	 * @param command_name
	 */
	pub fn set_current_command(
		&mut self,
		module_command: ModuleCommand,
		command_name: String,
	) {
		if !command_name.contains(' ') && !self.current_name.starts_with('!') {
			if !command_name.is_empty() {
				self.current_name = command_name;
			}
			self.command = module_command;
			self.current_info.set(
				vec![
					Text::styled(
						"Execute the following command? [y/N]:\n",
						self.style.colored,
					),
					Text::styled(self.get_current_command().cmd, self.style.default),
					Text::styled(
						format!("\n\n{}", self.get_current_command().desc),
						self.style.colored,
					),
				],
				3,
				self.get_current_command().cmd,
			);
			self.info_scroll_offset = 0;
		}
	}

	/**
	 * Execute the current module command.
	 *
	 * @return command_executed
	 */
	pub fn execute_command(&mut self) -> bool {
		let mut command_executed = false;
		if !self.command.is_none() {
			match util::exec_cmd("sh", &["-c", &self.get_current_command().cmd]) {
				Ok(_) => command_executed = true,
				Err(e) => {
					self.current_info.set(
						vec![
							Text::styled(
								"Failed to execute command:",
								self.style.colored,
							),
							Text::styled(
								format!(
									"\n'{}'\n\n{}",
									self.get_current_command().cmd,
									e
								),
								self.style.default,
							),
						],
						3,
						format!(
							"Execution Error\n'{}'\n{}",
							self.get_current_command().cmd,
							e
						),
					);
					self.current_name =
						format!("!Error{}", self.style.unicode.get(Symbol::NoEntry));
				}
			}
			self.command = ModuleCommand::None;
		}
		command_executed
	}

	/**
	 * Cancel the execution of the current command.
	 *
	 * @return cancelled
	 */
	pub fn cancel_execution(&mut self) -> bool {
		if !self.command.is_none() {
			self.command = ModuleCommand::None;
			if self.index != 0 {
				self.index -= 1;
				self.scroll_list(ScrollDirection::Down);
			} else {
				self.index += 1;
				self.scroll_list(ScrollDirection::Up);
			};
			true
		} else {
			false
		}
	}

	/**
	 * Scroll to the position of used module at given index.
	 *
	 * @param mod_index
	 */
	pub fn show_used_module(&mut self, mod_index: usize) {
		if let Some(used_module) = self.list[self.index][2]
			.split(' ')
			.collect::<Vec<&str>>()
			.get(1)
			.unwrap_or(&"")
			.split(',')
			.collect::<Vec<&str>>()
			.get(mod_index)
		{
			if let Some(v) = self
				.list
				.iter()
				.position(|module| module[0] == format!(" {}", used_module))
			{
				match v {
					0 => {
						self.index = v + 1;
						self.scroll_list(ScrollDirection::Up);
					}
					v if v > 0 => {
						self.index = v - 1;
						self.scroll_list(ScrollDirection::Down);
					}
					_ => {}
				}
			}
		}
	}

	/**
	 * Scroll module list up/down and select module.
	 *
	 * @param direction
	 */
	pub fn scroll_list(&mut self, direction: ScrollDirection) {
		self.info_scroll_offset = 0;
		if self.list.is_empty() {
			self.index = 0;
		} else {
			/* Scroll module list. */
			match direction {
				ScrollDirection::Up => self.previous_module(),
				ScrollDirection::Down => self.next_module(),
				ScrollDirection::Top => self.index = 0,
				ScrollDirection::Bottom => self.index = self.list.len() - 1,
				_ => {}
			}
			/* Set current module name. */
			self.current_name = self.list[self.index][0]
				.split_whitespace()
				.next()
				.unwrap_or("?")
				.trim()
				.to_string();
			/* Execute 'modinfo' and add style to its output. */
			self.current_info.stylize_data(
				Box::leak(
					util::exec_cmd("modinfo", &[&self.current_name])
						.unwrap_or_else(|_| {
							String::from("module information not available")
						})
						.replace("signature: ", "signature: \n")
						.into_boxed_str(),
				),
				":",
				self.style.clone(),
			);
			/* Clear the current command. */
			if !self.command.is_none() {
				self.command = ModuleCommand::None;
			}
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
	 * Scroll the module information text up/down.
	 *
	 * @param direction
	 * @param smooth_scroll
	 */
	pub fn scroll_mod_info(
		&mut self,
		direction: ScrollDirection,
		smooth_scroll: bool,
	) {
		let scroll_amount = if smooth_scroll { 1 } else { 2 };
		match direction {
			ScrollDirection::Up => {
				if self.info_scroll_offset > scroll_amount - 1 {
					self.info_scroll_offset -= scroll_amount;
				}
			}
			ScrollDirection::Down => {
				if self.current_info.lines() > 0 {
					self.info_scroll_offset += scroll_amount;
					self.info_scroll_offset %= self.current_info.lines() * 2;
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
	fn test_kernel_modules() {
		let args = ArgMatches::default();
		let mut list_args = ListArgs::new(&args);
		list_args.sort = SortType::Size;
		list_args.reverse = true;
		let mut kernel_modules = KernelModules::new(list_args, Style::new(&args));
		for sort_type in SortType::iter().rev().chain(SortType::iter()) {
			kernel_modules.args.sort = *sort_type;
			kernel_modules.refresh();
		}
		for direction in ScrollDirection::iter().rev().chain(ScrollDirection::iter())
		{
			kernel_modules.show_used_module(0);
			kernel_modules.scroll_list(*direction);
			kernel_modules
				.scroll_mod_info(*direction, *direction == ScrollDirection::Up);
		}
		kernel_modules.scroll_list(ScrollDirection::Down);
		assert_eq!(0, kernel_modules.index);
		kernel_modules.scroll_list(ScrollDirection::Up);
		assert_eq!(kernel_modules.default_list.len() - 1, kernel_modules.index);
		assert_ne!(0, kernel_modules.default_list.len());
		assert_ne!(0, kernel_modules.current_name.len());
		assert_ne!(0, kernel_modules.current_info.lines());
		kernel_modules
			.set_current_command(ModuleCommand::Load, String::from("test"));
		assert_eq!("test", kernel_modules.current_name);
		assert_eq!(false, kernel_modules.execute_command());
		kernel_modules.set_current_command(ModuleCommand::Load, String::new());
		kernel_modules.scroll_list(ScrollDirection::Top);
		for command in vec![
			ModuleCommand::Unload,
			ModuleCommand::Blacklist,
			ModuleCommand::None,
		] {
			kernel_modules.set_current_command(command.clone(), String::new());
			assert_eq!(!command.is_none(), kernel_modules.cancel_execution());
		}
	}
}
