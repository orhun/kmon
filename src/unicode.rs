use clap::ArgMatches;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Symbol {
	None,
	Anchor,
	CircleX,
	SquareX,
}

#[derive(Debug)]
pub struct Unicode<'a> {
	symbols: HashMap<Symbol, &'a [&'a str; 2]>,
	replace: bool,
}

impl Unicode<'_> {
	pub fn new(args: &ArgMatches) -> Self {
		Self {
			symbols: map! {
			Symbol::None => &["", ""],
			Symbol::Anchor => &["\u{2693}", ""],
			Symbol::CircleX => &["\u{1F167} ", ""],
			Symbol::SquareX => &["\u{1F187} ", ""]
			},
			replace: args.is_present("unicode"),
		}
	}
	pub fn get(&self, symbol: Symbol) -> &str {
		self.symbols[&symbol][self.replace as usize]
	}
}