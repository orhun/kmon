use clap::ArgMatches;
use colorsys::Rgb;
use std::collections::HashMap;
use tui::style::{Color, Modifier, Style as TuiStyle};
use tui::widgets::Text;

/* Unicode symbol */
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Symbol {
	None,
	Blank,
	Gear,
	Cloud,
	Anchor,
	Helmet,
	CircleX,
	SquareX,
	NoEntry,
	FuelPump,
	Magnifier,
	HighVoltage,
	LeftBracket,
	RightBracket,
}

/* Supported Unicode symbols */
#[derive(Clone, Debug)]
pub struct Unicode<'a> {
	symbols: HashMap<Symbol, &'a [&'a str; 2]>,
	replace: bool,
}

impl Unicode<'_> {
	/**
	 * Create a new Unicode instance.
	 *
	 * @param  replace
	 * @return Unicode
	 */
	pub fn new(replace: bool) -> Self {
		Self {
			symbols: map! {
			Symbol::None => &["", ""],
			Symbol::Blank => &["\u{2800} ", "\u{2800} "],
			Symbol::Gear => &[" \u{2699} ", ""],
			Symbol::Cloud => &[" \u{26C5} ", ""],
			Symbol::Anchor => &[" \u{2693}", ""],
			Symbol::Helmet => &[" \u{26D1} ", ""],
			Symbol::CircleX => &[" \u{1F167} ", ""],
			Symbol::SquareX => &[" \u{1F187} ", ""],
			Symbol::NoEntry => &[" \u{26D4}", ""],
			Symbol::FuelPump => &[" \u{26FD}", ""],
			Symbol::Magnifier => &[" \u{1F50D}", ""],
			Symbol::HighVoltage => &[" \u{26A1}", ""],
			Symbol::LeftBracket => &["\u{2997}", "("],
			Symbol::RightBracket => &["\u{2998}", ")"]
			},
			replace,
		}
	}
	/**
	 * Get string from a Unicode symbol.
	 *
	 * @param  Symbol
	 * @return str
	 */
	pub fn get(&self, symbol: Symbol) -> &str {
		self.symbols[&symbol][self.replace as usize]
	}
}

/* Style properties */
#[derive(Clone, Debug)]
pub struct Style {
	pub default: TuiStyle,
	pub bold: TuiStyle,
	pub colored: TuiStyle,
	pub unicode: Unicode<'static>,
}

impl Style {
	/**
	 * Create a new style instance from given arguments.
	 *
	 * @param  args
	 * @return Style
	 */
	pub fn new(args: &ArgMatches) -> Self {
		let colors = map![
			"black" => Color::Black,
			"red" => Color::Red,
			"green" => Color::Green,
			"yellow" => Color::Yellow,
			"blue" => Color::Blue,
			"magenta" => Color::Magenta,
			"cyan" => Color::Cyan,
			"gray" => Color::Gray,
			"darkgray" => Color::DarkGray,
			"lightred" => Color::LightRed,
			"lightgreen" => Color::LightGreen,
			"lightyellow" => Color::LightYellow,
			"lightblue" => Color::LightBlue,
			"lightmagenta" => Color::LightMagenta,
			"lightcyan" => Color::LightCyan,
			"white" => Color::White
		];
		let main_color = match args.value_of("color") {
			Some(v) => *colors.get::<str>(&v.to_lowercase()).unwrap_or({
				if let Ok(rgb) = Rgb::from_hex_str(&format!("#{}", v)) {
					Box::leak(Box::new(Color::Rgb(
						rgb.get_red() as u8,
						rgb.get_green() as u8,
						rgb.get_blue() as u8,
					)))
				} else {
					&Color::DarkGray
				}
			}),
			None => Color::DarkGray,
		};
		Self {
			default: TuiStyle::default(),
			bold: TuiStyle::default().modifier(Modifier::BOLD),
			colored: TuiStyle::default().fg(main_color),
			unicode: Unicode::new(!args.is_present("unicode")),
		}
	}
}

/* Styled text that has raw and style parts */
#[derive(Debug, Default)]
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
	 * Set a styled text.
	 *
	 * @param text
	 * @param newline_count
	 * @param placeholder
	 */
	pub fn set(
		&mut self,
		text: Vec<Text<'static>>,
		newline_count: usize,
		placeholder: String,
	) {
		self.styled_text = text;
		self.raw_text = placeholder;
		/* Append empty strings as much as newlines. */
		for _i in 0..newline_count * 2 {
			self.styled_text.push(Text::raw(""));
		}
	}

	/**
	 * Add style to given text depending on a delimiter.
	 *
	 * @param  text
	 * @param  delimiter
	 * @param  style
	 * @return vector
	 */
	pub fn stylize_data(
		&mut self,
		text: &str,
		delimiter: &str,
		style: Style,
	) -> Vec<Text<'a>> {
		self.styled_text = Vec::new();
		self.raw_text = text.to_string();
		for line in text.lines() {
			let data = line.split(delimiter).collect::<Vec<&str>>();
			if data.len() > 1 && data[0].trim().len() > 2 {
				self.styled_text.extend_from_slice(&[
					Text::styled(format!("{}{}", data[0], delimiter), style.colored),
					Text::styled(
						format!("{}\n", data[1..data.len()].join(delimiter)),
						style.default,
					),
				]);
			} else {
				self.styled_text
					.push(Text::styled(format!("{}\n", line), style.default));
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

#[cfg(test)]
mod tests {
	use super::*;
	use clap::{App, Arg};
	use tui::widgets::Text;
	#[test]
	fn test_style() {
		let mut args = App::new("test").get_matches();
		for color in vec!["black", "000000", "lightblue", "3c70a4"] {
			args = App::new("test")
				.arg(Arg::with_name("color").default_value(color))
				.get_matches();
			Style::new(&args);
		}
		let style = Style::new(&args);
		let mut styled_text = StyledText::default();
		styled_text.set(
			vec![Text::styled("styled\ntext", style.colored)],
			0,
			String::from("test"),
		);
		assert_eq!(
			vec![Text::styled("styled\ntext", style.colored)],
			styled_text.get()
		);
		assert_eq!(1, styled_text.lines());
		assert_eq!("test", styled_text.raw_text);
	}
	#[test]
	fn test_unicode() {
		let mut unicode = Unicode::new(true);
		for symbol in unicode.symbols.clone() {
			if symbol.0 != Symbol::Blank {
				assert_eq!(true, symbol.1[1].len() < 2)
			}
		}
		unicode.replace = false;
		for symbol in unicode.symbols {
			if symbol.0 != Symbol::None {
				assert_ne!("", symbol.1[0]);
			}
		}
	}
}
