use crate::event::Event;
use crate::kernel::cmd::ModuleCommand;
use crate::kernel::lkm::KernelModules;
use crate::kernel::log::KernelLogs;
use crate::kernel::Kernel;
use crate::style::{Style, StyledText, Symbol};
use crate::util;
use crate::widgets::StatefulList;
use clipboard::{ClipboardContext, ClipboardProvider};
use enum_unitary::{enum_unitary, Bounded, EnumUnitary};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::slice::Iter;
use std::sync::mpsc::Sender;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::Style as TuiStyle;
use tui::text::{Span, Spans, Text};
use tui::widgets::{
	Block as TuiBlock, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap,
};
use tui::Frame;
use unicode_width::UnicodeWidthStr;

/* Table header of the module table */
pub const TABLE_HEADER: &[&str] = &[" Module", "Size", "Used by"];

/* Available options in the module management menu */
const OPTIONS: &[(&str, &str)] = &[
	("unload", "Unload the module"),
	("reload", "Reload the module"),
	("blacklist", "Blacklist the module"),
	("dependent", "Show the dependent modules"),
	("copy", "Copy the module name"),
	("load", "Load a kernel module"),
	("clear", "Clear the ring buffer"),
];

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
	pub options: StatefulList<(String, String)>,
	pub show_options: bool,
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
			options: StatefulList::with_items(
				OPTIONS
					.iter()
					.map(|(option, text)| {
						(String::from(*option), String::from(*text))
					})
					.collect(),
			),
			show_options: false,
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
		self.options.state.select(Some(0));
		self.show_options = false;
	}

	/**
	 * Get style depending on the selected state of the block.
	 *
	 * @param  block
	 * @return TuiStyle
	 */
	pub fn block_style(&self, block: Block) -> TuiStyle {
		if self.show_options {
			self.style.colored
		} else if block == self.selected_block {
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
		let mut help_text = Vec::new();
		let mut help_text_raw = Vec::new();
		for (key, desc) in &key_bindings {
			help_text.push(Spans::from(Span::styled(
				format!("{}:", &key),
				self.style.colored,
			)));
			help_text_raw.push(format!("{}:", key));
			help_text.push(Spans::from(Span::styled(
				format!("{}{}", self.style.unicode.get(Symbol::Blank), &desc),
				self.style.default,
			)));
			help_text_raw.push(format!(" {}", &desc));
		}
		kernel_modules.info_scroll_offset = 0;
		kernel_modules.command = ModuleCommand::None;
		kernel_modules.current_name =
			format!("!Help{}", self.style.unicode.get(Symbol::Helmet));
		kernel_modules
			.current_info
			.set(Text::from(help_text), help_text_raw.join("\n"));
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
			let mut dependent_modules = Vec::new();
			for module in &dependent_modules_list {
				dependent_modules.push(Spans::from(vec![
					Span::styled("-", self.style.colored),
					Span::styled(format!(" {}", module), self.style.default),
				]));
			}
			kernel_modules.current_info.set(
				Text::from(dependent_modules),
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
		frame.render_widget(
			Paragraph::new(Span::raw(self.input_query.to_string()))
				.block(
					TuiBlock::default()
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
						.title(Span::styled(
							format!(
								"{}{}",
								self.input_mode.to_string(),
								match self.input_mode {
									InputMode::Load =>
										self.style.unicode.get(Symbol::Anchor),
									_ => self.style.unicode.get(Symbol::Magnifier),
								}
							),
							self.style.bold,
						)),
				)
				.alignment(Alignment::Left),
			area,
		);
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
		frame.render_widget(
			Paragraph::new(Span::raw(&info[1]))
				.block(
					TuiBlock::default()
						.border_style(self.style.colored)
						.borders(Borders::ALL)
						.title(Span::styled(
							&format!(
								"{}{}",
								info[0],
								self.style.unicode.get(Symbol::Gear)
							),
							self.style.bold,
						))
						.title_alignment(Alignment::Center),
				)
				.alignment(Alignment::Center)
				.wrap(Wrap { trim: true }),
			area,
		);
	}

	/**
	 * Configure and draw kernel modules table.
	 *
	 * @param frame
	 * @param area
	 * @param kernel_modules
	 */
	pub fn draw_kernel_modules<B>(
		&mut self,
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
		frame.render_widget(
			Table::new(
				kernel_modules
					.list
					.iter()
					.skip(modules_scroll_offset)
					.enumerate()
					.map(|(i, item)| {
						let item = item.iter().map(|v| v.to_string());
						if Some(i)
							== kernel_modules
								.index
								.checked_sub(modules_scroll_offset)
						{
							Row::new(item).style(self.style.default)
						} else {
							Row::new(item).style(self.style.colored)
						}
					}),
			)
			.header(
				Row::new(TABLE_HEADER.iter().map(|v| v.to_string()))
					.style(self.style.bold),
			)
			.block(
				TuiBlock::default()
					.border_style(self.block_style(Block::ModuleTable))
					.borders(Borders::ALL)
					.title(Span::styled(
						format!(
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
									/ kernel_modules.list.len() as f64 * 100.0) as u64
							} else {
								0
							},
							self.style.unicode.get(Symbol::RightBracket),
						),
						self.style.bold,
					)),
			)
			.widths(&[
				Constraint::Percentage(30),
				Constraint::Percentage(20),
				Constraint::Percentage(50),
			]),
			area,
		);
		if self.show_options {
			self.draw_options_menu(frame, area, kernel_modules);
		}
	}

	/**
	 * Draws the options menu as a popup.
	 *
	 * @param frame
	 * @param area
	 */
	pub fn draw_options_menu<B>(
		&mut self,
		frame: &mut Frame<'_, B>,
		area: Rect,
		kernel_modules: &mut KernelModules<'_>,
	) where
		B: Backend,
	{
		let block_title = format!(
			"Options ({})",
			kernel_modules.list[kernel_modules.index][0]
				.split_whitespace()
				.next()
				.unwrap_or("?")
				.trim()
				.to_string()
		);
		let items = self
			.options
			.items
			.iter()
			.map(|(_, text)| ListItem::new(Span::raw(format!(" {}", text))))
			.collect::<Vec<ListItem<'_>>>();
		let (mut percent_y, mut percent_x) = (40, 60);
		let text_height = items.iter().map(|v| v.height() as f32).sum::<f32>() + 3.;
		if area.height.checked_sub(5).unwrap_or(area.height) as f32 > text_height {
			percent_y = ((text_height / area.height as f32) * 100.) as u16;
		}
		if let Some(text_width) = self
			.options
			.items
			.iter()
			.map(|(_, text)| text.width())
			.chain(vec![block_title.width()].into_iter())
			.max()
			.map(|v| v as f32 + 7.)
		{
			if area.width.checked_sub(2).unwrap_or(area.width) as f32 > text_width {
				percent_x = ((text_width / area.width as f32) * 100.) as u16;
			}
		}
		let popup_layout = Layout::default()
			.direction(Direction::Vertical)
			.constraints(
				[
					Constraint::Percentage((100 - percent_y) / 2),
					Constraint::Percentage(percent_y),
					Constraint::Percentage((100 - percent_y) / 2),
				]
				.as_ref(),
			)
			.split(area);
		let popup_rect = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(
				[
					Constraint::Percentage((100 - percent_x) / 2),
					Constraint::Percentage(percent_x),
					Constraint::Percentage((100 - percent_x) / 2),
				]
				.as_ref(),
			)
			.split(popup_layout[1])[1];
		frame.render_widget(Clear, popup_rect);
		frame.render_stateful_widget(
			List::new(items)
				.block(
					TuiBlock::default()
						.title(Span::styled(block_title, self.style.bold))
						.title_alignment(Alignment::Center)
						.style(self.style.default)
						.borders(Borders::ALL),
				)
				.style(self.style.colored)
				.highlight_style(self.style.default),
			popup_rect,
			&mut self.options.state,
		);
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
		frame.render_widget(
			Paragraph::new(kernel_modules.current_info.get())
				.block(
					TuiBlock::default()
						.border_style(self.block_style(Block::ModuleInfo))
						.borders(Borders::ALL)
						.title(Span::styled(
							format!(
								"{}{}",
								kernel_modules.get_current_command().title,
								self.style.unicode.get(
									kernel_modules.get_current_command().symbol
								)
							),
							self.style.bold,
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
				.wrap(Wrap { trim: true })
				.scroll((kernel_modules.info_scroll_offset as u16, 0)),
			area,
		);
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
		frame.render_widget(
			Paragraph::new(StyledText::default().stylize_data(
				kernel_logs.select(area.height, 2),
				"] ",
				self.style.clone(),
			))
			.block(
				TuiBlock::default()
					.border_style(self.block_style(Block::Activities))
					.borders(Borders::ALL)
					.title(Span::styled(
						format!(
							"Kernel Activities{}",
							self.style.unicode.get(Symbol::HighVoltage)
						),
						self.style.bold,
					)),
			)
			.alignment(Alignment::Left),
			area,
		);
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
