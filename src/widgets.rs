use ratatui::widgets::ListState;

/// List widget with TUI controlled states.
#[derive(Debug)]
pub struct StatefulList<T> {
	/// List items (states).
	pub items: Vec<T>,
	/// State that can be modified by TUI.
	pub state: ListState,
}

impl<T> StatefulList<T> {
	/// Constructs a new instance of `StatefulList`.
	pub fn new(items: Vec<T>, mut state: ListState) -> StatefulList<T> {
		state.select(Some(0));
		Self { items, state }
	}

	/// Construct a new `StatefulList` with given items.
	pub fn with_items(items: Vec<T>) -> StatefulList<T> {
		Self::new(items, ListState::default())
	}

	/// Returns the selected item.
	pub fn selected(&self) -> Option<&T> {
		self.items.get(self.state.selected()?)
	}

	/// Selects the next item.
	pub fn next(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i >= self.items.len() - 1 {
					0
				} else {
					i + 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}

	/// Selects the previous item.
	pub fn previous(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i == 0 {
					self.items.len() - 1
				} else {
					i - 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_stateful_list() {
		let mut list = StatefulList::with_items(vec!["data1", "data2", "data3"]);
		list.state.select(Some(1));
		assert_eq!(Some(&"data2"), list.selected());
		list.next();
		assert_eq!(Some(2), list.state.selected());
		list.previous();
		assert_eq!(Some(1), list.state.selected());
	}
}
