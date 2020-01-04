use tui::style::{Color, Modifier, Style as TuiStyle};
use tui::widgets::Text;

/* Style properties */
#[derive(Debug, Clone, Copy)]
pub struct Style {
	pub default: TuiStyle,
	pub bold: TuiStyle,
	pub colored: TuiStyle,
}

impl Style {
	/**
	 * Create a new style instance.
	 *
	 * @return Style
	 */
	pub fn new() -> Self {
		Self {
			default: TuiStyle::default(),
			bold: TuiStyle::default().modifier(Modifier::BOLD),
			colored: TuiStyle::default().fg(Color::DarkGray),
		}
	}
}

/* Styled text that has raw and style parts */
#[derive(Default)]
pub struct StyledText<'a> {
	pub raw_text: String,
	pub styled_text: Vec<Text<'a>>,
}

impl<'a> StyledText<'a> {
	/**
	 * Get a vector of Text widget from styled text.
	 *
	 * @return vector
	 */
	pub fn get(&'a self) -> Vec<Text<'a>> {
		if self.styled_text.is_empty() {
			vec![Text::raw(&self.raw_text)]
		} else {
			self.styled_text.to_vec()
		}
	}

	/**
	 * Set a raw text.
	 *
	 * @param text
	 */
	pub fn set_raw_text(&mut self, text: String) {
		self.raw_text = text;
		self.styled_text = Vec::new();
	}

	/**
	 * Set a styled text.
	 *
	 * @param text
	 * @param newline_count
	 */
	pub fn set_styled_text(
		&mut self,
		text: Vec<Text<'static>>,
		newline_count: usize,
	) {
		self.styled_text = text;
		/* Append empty strings as much as newlines. */
		for _i in 0..newline_count * 2 {
			self.styled_text.push(Text::raw(""));
		}
		self.raw_text = String::new();
	}

	/**
	 * Add style to given text depending on a delimiter.
	 *
	 * @param  text
	 * @param  delimiter
	 * @param  style
	 * @return vector
	 */
	pub fn stylize_data(&mut self, text: &str, delimiter: char, style: Style) -> Vec<Text<'a>> {
		self.styled_text = Vec::new();
		self.raw_text = text.to_string();
		for line in text.lines() {
			let data = line.split(delimiter).collect::<Vec<&str>>();
			if data.len() > 1 && data[0].trim().len() > 2 {
				self.styled_text.extend_from_slice(&[
					Text::styled(
						format!("{}{}", data[0], delimiter),
						style.colored,
					),
					Text::styled(
						format!(
							"{}\n",
							data[1..data.len()].join(&delimiter.to_string())
						),
						style.default,
					),
				]);
			} else {
				self.styled_text.push(Text::styled(
					format!("{}\n", line),
					style.default,
				));
			}
		}
		self.styled_text.clone()
	}

	/**
	 * Return the line count of styled text.
	 *
	 * @return usize
	 */
	pub fn lines(&self) -> usize {
		if self.styled_text.is_empty() {
			self.raw_text.lines().count()
		} else {
			self.styled_text.len()
		}
	}
}
