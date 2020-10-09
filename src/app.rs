use crate::event::Event;
use crate::kernel::cmd::ModuleCommand;
use crate::kernel::lkm::KernelModules;
use crate::kernel::log::KernelLogs;
use crate::kernel::Kernel;
use crate::style::{Style, StyledText, Symbol};
use crate::util;
use clipboard::{ClipboardContext, ClipboardProvider};
use enum_unitary::{enum_unitary, Bounded, EnumUnitary};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::slice::Iter;
use std::sync::mpsc::Sender;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Rect};
use tui::style::Style as TuiStyle;
use tui::widgets::{
	Block as TuiBlock, Borders, Paragraph, Row, Table, Text, Widget,
};
use tui::Frame;

/* Table header of the module table */
pub const TABLE_HEADER: &[&str] = &[" Module", "Size", "Used by"];

/* Supported directions of scrolling */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDirection {
	Up,
	Down,
	Left,
	Right,
	Top,
	Bottom,
}

impl ScrollDirection {
	/**
	 * Return iterator of the available scroll directions.
	 *
	 * @return Iter
	 */
	#[allow(dead_code)]
	pub fn iter() -> Iter<'static, ScrollDirection> {
		[
			ScrollDirection::Up,
			ScrollDirection::Down,
			ScrollDirection::Left,
			ScrollDirection::Right,
			ScrollDirection::Top,
			ScrollDirection::Bottom,
		]
		.iter()
	}
}

/* Main blocks of the terminal */
enum_unitary! {
	#[derive(Copy, Debug, PartialEq)]
	pub enum Block {
		UserInput,
		ModuleTable,
		ModuleInfo,
		Activities,
	}
}

/* Sizes of the terminal blocks */
pub struct BlockSize {
	pub input: u16,
	pub info: u16,
	pub activities: u16,
}

/* Default initialization values for BlockSize */
impl Default for BlockSize {
	fn default() -> Self {
		Self {
			input: 60,
			info: 40,
			activities: 25,
		}
	}
}

/* User input mode */
enum_unitary! {
	#[derive(Copy, Debug, PartialEq)]
	pub enum InputMode {
		None,
		Search,
		Load,
	}
}

impl InputMode {
	/**
	 * Check if input mode is set.
	 *
	 * @return bool
	 */
	pub fn is_none(self) -> bool {
		self == Self::None
	}
}

/* Implementation of Display for using InputMode members as string */
impl Display for InputMode {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut input_mode = *self;
		if input_mode.is_none() {
			input_mode = match InputMode::min_value().next_variant() {
				Some(v) => v,
				None => input_mode,
			}
		}
		write!(f, "{:?}", input_mode)
	}
}

/* Application settings and related methods  */
pub struct App {
	pub selected_block: Block,
	pub default_block: Block,
	pub block_size: BlockSize,
	pub block_index: u8,
	pub input_mode: InputMode,
	pub input_query: String,
	style: Style,
}

impl App {
	/**
	 * Create a new app instance.
	 *
	 * @param  Block
	 * @param  Style
	 * @return App
	 */
	pub fn new(block: Block, style: Style) -> Self {
		Self {
			selected_block: block,
			default_block: block,
			block_size: BlockSize::default(),
			block_index: 0,
			input_mode: InputMode::None,
			input_query: String::new(),
			style,
		}
	}

	/* Reset app properties to default. */
	pub fn refresh(&mut self) {
		self.selected_block = self.default_block;
		self.block_size = BlockSize::default();
		self.block_index = 0;
		self.input_mode = InputMode::None;
		self.input_query = String::new();
	}

	/**
	 * Get style depending on the selected state of the block.
	 *
	 * @param  block
	 * @return TuiStyle
	 */
	pub fn block_style(&self, block: Block) -> TuiStyle {
		if block == self.selected_block {
			self.style.default
		} else {
			self.style.colored
		}
	}

	/**
	 * Get the size of the selected block.
	 *
	 * @return u16
	 */
	pub fn block_size(&mut self) -> &mut u16 {
		match self.selected_block {
			Block::ModuleInfo => &mut self.block_size.info,
			Block::Activities => &mut self.block_size.activities,
			_ => &mut self.block_size.input,
		}
	}

	/**
	 * Get clipboard contents as String.
	 *
	 * @return contents
	 */
	pub fn get_clipboard_contents(&self) -> String {
		let clipboard_context: Result<ClipboardContext, Box<dyn Error>> =
			ClipboardProvider::new();
		match clipboard_context {
			Ok(mut v) => v.get_contents().unwrap_or_default(),
			Err(_) => String::new(),
		}
	}

	/**
	 * Set clipboard contents.
	 *
	 * @param contents
	 */
	pub fn set_clipboard_contents(&self, contents: &str) {
		let clipboard_context: Result<ClipboardContext, Box<dyn Error>> =
			ClipboardProvider::new();
		if let Ok(mut v) = clipboard_context {
			v.set_contents(contents.to_string()).unwrap();
		}
	}

	/**
	 * Show help message on the information block.
	 *
	 * @param kernel_modules
	 */
	pub fn show_help_message(&mut self, kernel_modules: &mut KernelModules<'_>) {
		let key_bindings: Vec<(&str, &str)> = util::KEY_BINDINGS.to_vec();
		let mut help_text: Vec<Text<'static>> = Vec::new();
		for (key, desc) in &key_bindings {
			help_text.push(Text::styled(
				format!("{}:\n{}", key, self.style.unicode.get(Symbol::Blank)),
				self.style.colored,
			));
			help_text.push(Text::styled(format!("{}\n", desc), self.style.default));
		}
		kernel_modules.info_scroll_offset = 0;
		kernel_modules.command = ModuleCommand::None;
		kernel_modules.current_name =
			format!("!Help{}", self.style.unicode.get(Symbol::Helmet));
		kernel_modules.current_info.set(
			help_text,
			key_bindings.len(),
			kernel_modules.current_name.clone(),
		);
	}

	/**
	 * Show dependent modules on the information block.
	 *
	 * @param kernel_modules
	 */
	#[allow(clippy::nonminimal_bool)]
	pub fn show_dependent_modules(
		&mut self,
		kernel_modules: &mut KernelModules<'_>,
	) {
		let dependent_modules_list = kernel_modules.default_list
			[kernel_modules.index][2]
			.split(' ')
			.last()
			.unwrap_or("-")
			.split(',')
			.collect::<Vec<&str>>();
		if !(dependent_modules_list[0] == "-"
			|| kernel_modules.current_name.contains("Dependent modules"))
			|| cfg!(test)
		{
			kernel_modules.info_scroll_offset = 0;
			kernel_modules.command = ModuleCommand::None;
			kernel_modules.current_name = format!(
				"!Dependent modules of {}{}",
				kernel_modules.current_name,
				self.style.unicode.get(Symbol::HistoricSite)
			);
			let mut dependent_modules: Vec<Text<'_>> = Vec::new();
			for module in &dependent_modules_list {
				dependent_modules.push(Text::styled("-", self.style.colored));
				dependent_modules.push(Text::styled(
					format!(" {}\n", module),
					self.style.default,
				));
			}
			kernel_modules.current_info.set(
				dependent_modules.clone(),
				dependent_modules.len(),
				kernel_modules.current_name.clone(),
			);
		}
	}

	/**
	 * Draw a block according to the index.
	 *
	 * @param frame
	 * @param area
	 * @param kernel
	 */
	pub fn draw_dynamic_block<B>(
		&mut self,
		frame: &mut Frame<'_, B>,
		area: Rect,
		kernel: &mut Kernel,
	) where
		B: Backend,
	{
		match self.block_index {
			0 => self.draw_kernel_modules(frame, area, &mut kernel.modules),
			1 => self.draw_module_info(frame, area, &mut kernel.modules),
			_ => self.draw_kernel_activities(frame, area, &mut kernel.logs),
		}
		if self.block_index < 2 {
			self.block_index += 1;
		} else {
			self.block_index = 0;
		}
	}

	/**
	 * Draw a paragraph widget for using as user input.
	 *
	 * @param frame
	 * @param area
	 * @param tx
	 */
	pub fn draw_user_input<B>(
		&self,
		frame: &mut Frame<'_, B>,
		area: Rect,
		tx: &Sender<Event<Key>>,
	) where
		B: Backend,
	{
		Paragraph::new([Text::raw(self.input_query.to_string())].iter())
			.block(
				TuiBlock::default()
					.title_style(self.style.bold)
					.border_style(match self.selected_block {
						Block::UserInput => {
							if self.input_mode.is_none() {
								tx.send(Event::Input(Key::Char('\n'))).unwrap();
							}
							self.style.default
						}
						_ => self.style.colored,
					})
					.borders(Borders::ALL)
					.title(&format!(
						"{}{}",
						self.input_mode.to_string(),
						match self.input_mode {
							InputMode::Load =>
								self.style.unicode.get(Symbol::Anchor),
							_ => self.style.unicode.get(Symbol::Magnifier),
						}
					)),
			)
			.alignment(Alignment::Left)
			.wrap(false)
			.render(frame, area);
	}

	/**
	 * Draw a paragraph widget for showing the kernel information.
	 *
	 * @param frame
	 * @param area
	 * @param info
	 */
	pub fn draw_kernel_info<B>(
		&self,
		frame: &mut Frame<'_, B>,
		area: Rect,
		info: &[String],
	) where
		B: Backend,
	{
		Paragraph::new([Text::raw(&info[1])].iter())
			.block(
				TuiBlock::default()
					.title_style(self.style.bold)
					.border_style(self.style.colored)
					.borders(Borders::ALL)
					.title(&format!(
						"{}{}",
						info[0],
						self.style.unicode.get(Symbol::Gear)
					)),
			)
			.alignment(Alignment::Center)
			.wrap(true)
			.render(frame, area);
	}

	/**
	 * Configure and draw kernel modules table.
	 *
	 * @param frame
	 * @param area
	 * @param kernel_modules
	 */
	pub fn draw_kernel_modules<B>(
		&self,
		frame: &mut Frame<'_, B>,
		area: Rect,
		kernel_modules: &mut KernelModules<'_>,
	) where
		B: Backend,
	{
		/* Filter the module list depending on the input query. */
		let mut kernel_module_list = kernel_modules.default_list.clone();
		if (self.input_mode == InputMode::None
			|| self.input_mode == InputMode::Search)
			&& !self.input_query.is_empty()
		{
			kernel_module_list.retain(|module| {
				module[0]
					.to_lowercase()
					.contains(&self.input_query.to_lowercase())
			});
		}
		/* Append '...' if dependent modules exceed the block width. */
		let dependent_width = (area.width / 2).saturating_sub(7) as usize;
		for module in &mut kernel_module_list {
			if module[2].len() > dependent_width {
				module[2].truncate(dependent_width);
				module[2] = format!("{}...", module[2]);
			}
		}
		kernel_modules.list = kernel_module_list;
		/* Set the scroll offset for modules. */
		let modules_scroll_offset = area
			.height
			.checked_sub(5)
			.and_then(|height| kernel_modules.index.checked_sub(height as usize))
			.unwrap_or(0);
		/* Set selected state of the modules and render the table widget. */
		Table::new(
			TABLE_HEADER.iter(),
			kernel_modules
				.list
				.iter()
				.skip(modules_scroll_offset)
				.enumerate()
				.map(|(i, item)| {
					if Some(i)
						== kernel_modules.index.checked_sub(modules_scroll_offset)
					{
						Row::StyledData(item.iter(), self.style.default)
					} else {
						Row::StyledData(item.iter(), self.style.colored)
					}
				}),
		)
		.block(
			TuiBlock::default()
				.title_style(self.style.bold)
				.border_style(self.block_style(Block::ModuleTable))
				.borders(Borders::ALL)
				.title(&format!(
					"Loaded Kernel Modules {}{}/{}{} {}{}%{}",
					self.style.unicode.get(Symbol::LeftBracket),
					match kernel_modules.list.len() {
						0 => kernel_modules.index,
						_ => kernel_modules.index + 1,
					},
					kernel_modules.list.len(),
					self.style.unicode.get(Symbol::RightBracket),
					self.style.unicode.get(Symbol::LeftBracket),
					if !kernel_modules.list.is_empty() {
						((kernel_modules.index + 1) as f64
							/ kernel_modules.list.len() as f64
							* 100.0) as u64
					} else {
						0
					},
					self.style.unicode.get(Symbol::RightBracket),
				)),
		)
		.header_style(self.style.bold)
		.widths(&[
			Constraint::Percentage(30),
			Constraint::Percentage(20),
			Constraint::Percentage(50),
		])
		.render(frame, area);
	}

	/**
	 * Draw a paragraph widget for showing module information.
	 *
	 * @param frame
	 * @param area
	 * @param kernel_modules
	 */
	pub fn draw_module_info<B>(
		&self,
		frame: &mut Frame<'_, B>,
		area: Rect,
		kernel_modules: &mut KernelModules<'_>,
	) where
		B: Backend,
	{
		Paragraph::new(kernel_modules.current_info.get().iter())
			.block(
				TuiBlock::default()
					.title_style(self.style.bold)
					.border_style(self.block_style(Block::ModuleInfo))
					.borders(Borders::ALL)
					.title(&format!(
						"{}{}",
						kernel_modules.get_current_command().title,
						self.style
							.unicode
							.get(kernel_modules.get_current_command().symbol)
					)),
			)
			.alignment(
				if kernel_modules.command.is_none()
					&& !kernel_modules
						.current_info
						.raw_text
						.contains("Execution Error\n")
				{
					Alignment::Left
				} else {
					Alignment::Center
				},
			)
			.wrap(true)
			.scroll(kernel_modules.info_scroll_offset as u16)
			.render(frame, area);
	}

	/**
	 * Draw a paragraph widget for showing kernel activities.
	 *
	 * @param frame
	 * @param area
	 * @param kernel_logs
	 */
	pub fn draw_kernel_activities<B>(
		&self,
		frame: &mut Frame<'_, B>,
		area: Rect,
		kernel_logs: &mut KernelLogs,
	) where
		B: Backend,
	{
		Paragraph::new(
			StyledText::default()
				.stylize_data(
					&kernel_logs.select(area.height, 2),
					"] ",
					self.style.clone(),
				)
				.iter(),
		)
		.block(
			TuiBlock::default()
				.title_style(self.style.bold)
				.border_style(self.block_style(Block::Activities))
				.borders(Borders::ALL)
				.title(&format!(
					"Kernel Activities{}",
					self.style.unicode.get(Symbol::HighVoltage)
				)),
		)
		.alignment(Alignment::Left)
		.wrap(false)
		.render(frame, area);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::event::Events;
	use crate::kernel::info;
	use crate::kernel::lkm::ListArgs;
	use clap::ArgMatches;
	use tui::backend::TestBackend;
	use tui::Terminal;
	#[test]
	fn test_app() {
		let args = ArgMatches::default();
		let mut kernel_modules =
			KernelModules::new(ListArgs::new(&args), Style::new(&args));
		let mut app = App::new(Block::ModuleTable, kernel_modules.style.clone());
		app.set_clipboard_contents("test");
		assert_ne!("x", app.get_clipboard_contents());
		assert_eq!(app.style.default, app.block_style(Block::ModuleTable));
		assert_eq!(app.style.colored, app.block_style(Block::Activities));
		let mut kernel_logs = KernelLogs::default();
		let backend = TestBackend::new(20, 10);
		let mut terminal = Terminal::new(backend).unwrap();
		terminal
			.draw(|mut f| {
				let size = f.size();
				app.selected_block = Block::UserInput;
				app.draw_user_input(
					&mut f,
					size,
					&Events::new(100, &kernel_logs).tx,
				);
				app.draw_kernel_info(
					&mut f,
					size,
					&info::KernelInfo::new().current_info,
				);
				app.input_query = String::from("a");
				app.draw_kernel_modules(&mut f, size, &mut kernel_modules);
				app.draw_module_info(&mut f, size, &mut kernel_modules);
				app.draw_kernel_activities(&mut f, size, &mut kernel_logs);
			})
			.unwrap();
	}
	#[test]
	fn test_input_mode() {
		let mut input_mode = InputMode::Load;
		assert_eq!(false, input_mode.is_none());
		assert_eq!(true, input_mode.to_string().contains("Load"));
		input_mode = InputMode::None;
		assert_eq!(true, input_mode.to_string().contains("Search"));
	}
}
