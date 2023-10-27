use clap::ArgMatches;
use colorsys::Rgb;
use ratatui::style::{Color, Modifier, Style as TuiStyle};
use ratatui::text::{Line, Span, Text};
use std::collections::HashMap;

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
	HistoricSite,
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
			Symbol::RightBracket => &["\u{2998}", ")"],
			Symbol::HistoricSite => &[" \u{26EC} ", ""]
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
		let mut default = TuiStyle::reset();
		if let Ok(true) = args.try_contains_id("accent-color") {
			default =
				default.fg(Self::get_color(args, "accent-color", Color::White));
		}
		Self {
			default,
			bold: TuiStyle::reset().add_modifier(Modifier::BOLD),
			colored: TuiStyle::reset().fg(Self::get_color(
				args,
				"color",
				Color::DarkGray,
			)),
			unicode: Unicode::new(
				args.try_get_one::<bool>("unicode").ok().flatten() == Some(&false),
			),
		}
	}

	/**
	 * Parse a color value from arguments.
	 *
	 * @param  args
	 * @param  arg_name
	 * @param  default_color
	 * @return Color
	 */
	fn get_color(args: &ArgMatches, arg_name: &str, default_color: Color) -> Color {
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
		match args.try_get_one::<String>(arg_name) {
			Ok(Some(v)) => *colors.get::<str>(&v.to_lowercase()).unwrap_or({
				if let Ok(rgb) = Rgb::from_hex_str(&format!("#{v}")) {
					Box::leak(Box::new(Color::Rgb(
						rgb.red() as u8,
						rgb.green() as u8,
						rgb.blue() as u8,
					)))
				} else {
					&default_color
				}
			}),
			_ => default_color,
		}
	}
}

/* Styled text that has raw and style parts */
#[derive(Debug, Default)]
pub struct StyledText<'a> {
	pub raw_text: String,
	pub styled_text: Text<'a>,
}

impl<'a> StyledText<'a> {
	/**
	 * Get a vector of Text widget from styled text.
	 *
	 * @return vector
	 */
	pub fn get(&'a self) -> Text<'a> {
		if self.styled_text.lines.is_empty() {
			Text::raw(&self.raw_text)
		} else {
			self.styled_text.clone()
		}
	}

	/**
	 * Set a styled text.
	 *
	 * @param text
	 * @param placeholder
	 */
	pub fn set(&mut self, text: Text<'static>, placeholder: String) {
		self.styled_text = text;
		self.raw_text = placeholder;
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
		text: &'a str,
		delimiter: &str,
		style: Style,
	) -> Text<'a> {
		self.styled_text = Text::default();
		self.raw_text = text.to_string();
		for line in text.lines() {
			let data = line.split(delimiter).collect::<Vec<&str>>();
			if data.len() > 1 && data[0].trim().len() > 2 {
				self.styled_text.lines.push(Line::from(vec![
					Span::styled(format!("{}{}", data[0], delimiter), style.colored),
					Span::styled(data[1..data.len()].join(delimiter), style.default),
				]));
			} else {
				self.styled_text
					.lines
					.push(Line::from(Span::styled(line, style.default)))
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
		if self.styled_text.lines.is_empty() {
			self.raw_text.lines().count()
		} else {
			self.styled_text.lines.len()
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use clap::ArgMatches;
	#[test]
	fn test_style() {
		let args = ArgMatches::default();
		let style = Style::new(&args);
		let mut styled_text = StyledText::default();
		styled_text.set(
			Text::styled("styled\ntext", style.colored),
			String::from("test"),
		);
		assert_eq!(
			Text::styled("styled\ntext", style.colored),
			styled_text.get()
		);
		assert_eq!(2, styled_text.lines());
		assert_eq!("test", styled_text.raw_text);
	}
	#[test]
	fn test_unicode() {
		let mut unicode = Unicode::new(true);
		for symbol in unicode.symbols.clone() {
			if symbol.0 != Symbol::Blank {
				assert!(symbol.1[1].len() < 2)
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
