use crate::event::Event;
use crate::kernel::lkm::KernelModules;
use crate::kernel::log::KernelLogs;
use enum_unitary::enum_unitary;
use std::sync::mpsc::Sender;
use termion::event::Key;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget};
use tui::Frame;

/* Main blocks of the terminal */
enum_unitary! {
    #[derive(PartialEq)]
    pub enum Blocks {
        SearchInput,
        ModuleTable,
        ModuleInfo,
        Activities,
    }
}

/* Application settings and related methods  */
pub struct App {
    pub selected_block: Blocks,
    pub search_mode: bool,
    pub search_query: String,
    pub table_header: [&'static str; 3],
    pub title_style: Style,
    pub selected_style: Style,
    pub unselected_style: Style,
}

impl App {
    /**
     * Create a new app instance.
     *
     * @param  block
     * @return App
     */
    pub fn new(block: Blocks) -> Self {
        Self {
            selected_block: block,
            search_mode: false,
            search_query: String::new(),
            table_header: ["Module", "Size", "Used by"],
            title_style: Style::default().modifier(Modifier::BOLD),
            selected_style: Style::default().fg(Color::White),
            unselected_style: Style::default().fg(Color::DarkGray),
        }
    }

    /**
     * Get style depending on the selected state of the block.
     *
     * @param  block
     * @return Style
     */
    pub fn block_style(&self, block: Blocks) -> Style {
        if block == self.selected_block {
            self.selected_style
        } else {
            self.unselected_style
        }
    }

    /**
     * Draw a paragraph widget for using as search input.
     *
     * @param frame
     * @param area
     * @param tx
     */
    pub fn draw_search_input<B>(&self, frame: &mut Frame<B>, area: Rect, tx: &Sender<Event<Key>>)
    where
        B: Backend,
    {
        Paragraph::new([Text::raw(self.search_query.to_string())].iter())
            .block(
                Block::default()
                    .title_style(self.title_style)
                    .border_style(match self.selected_block {
                        Blocks::SearchInput => {
                            if !self.search_mode {
                                tx.send(Event::Input(Key::Char('\n'))).unwrap();
                            }
                            self.selected_style
                        }
                        _ => self.unselected_style,
                    })
                    .borders(Borders::ALL)
                    .title("Search"),
            )
            .render(frame, area);
    }

    /**
     * Draw a paragraph widget for showing the kernel version.
     *
     * @param frame
     * @param area
     * @param version
     */
    pub fn draw_kernel_version<B>(&self, frame: &mut Frame<B>, area: Rect, version: &str)
    where
        B: Backend,
    {
        Paragraph::new([Text::raw(version)].iter())
            .block(
                Block::default()
                    .title_style(self.title_style)
                    .border_style(self.unselected_style)
                    .borders(Borders::ALL)
                    .title("Kernel Version"),
            )
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
        frame: &mut Frame<B>,
        area: Rect,
        kernel_modules: &mut KernelModules,
    ) where
        B: Backend,
    {
        /* Filter the module list depending on the search query. */
        let mut kernel_module_list = kernel_modules.default_list.clone();
        if self.search_query.len() > 0 {
            kernel_module_list.retain(|module| {
                module[0]
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            });
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
            self.table_header.iter(),
            kernel_modules
                .list
                .iter()
                .skip(modules_scroll_offset)
                .enumerate()
                .map(|(i, item)| {
                    if Some(i) == kernel_modules.index.checked_sub(modules_scroll_offset) {
                        Row::StyledData(
                            item.into_iter(),
                            self.selected_style.modifier(Modifier::BOLD),
                        )
                    } else {
                        Row::StyledData(item.into_iter(), self.selected_style)
                    }
                })
                .into_iter(),
        )
        .block(
            Block::default()
                .title_style(self.title_style)
                .border_style(self.block_style(Blocks::ModuleTable))
                .borders(Borders::ALL)
                .title(&format!(
                    "Loaded Kernel Modules ({}/{}) [{}%]",
                    match kernel_modules.list.len() {
                        0 => kernel_modules.index,
                        _ => kernel_modules.index + 1,
                    },
                    kernel_modules.list.len(),
                    ((kernel_modules.index + 1) as f64 / kernel_modules.list.len() as f64 * 100.0)
                        as usize
                )),
        )
        .widths(&[
            (f64::from(area.width - 3) * 0.3) as u16,
            (f64::from(area.width - 3) * 0.2) as u16,
            (f64::from(area.width - 3) * 0.5) as u16,
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
        frame: &mut Frame<B>,
        area: Rect,
        kernel_modules: &mut KernelModules,
    ) where
        B: Backend,
    {
        Paragraph::new([Text::raw(kernel_modules.current_info.to_string())].iter())
            .block(
                Block::default()
                    .title_style(self.title_style)
                    .border_style(self.block_style(Blocks::ModuleInfo))
                    .borders(Borders::ALL)
                    .title(&format!("Module: {}", kernel_modules.current_name)),
            )
            .wrap(true)
            .scroll(kernel_modules.info_scroll_offset)
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
        frame: &mut Frame<B>,
        area: Rect,
        kernel_logs: &mut KernelLogs,
    ) where
        B: Backend,
    {
        Paragraph::new([Text::raw(kernel_logs.output.to_string())].iter())
            .block(
                Block::default()
                    .title_style(self.title_style)
                    .border_style(self.block_style(Blocks::Activities))
                    .borders(Borders::ALL)
                    .title("Kernel Activities"),
            )
            .wrap(true)
            .scroll(kernel_logs.scroll_offset)
            .render(frame, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app() {
        let app = App::new(Blocks::ModuleTable);
        assert_eq!(app.selected_style, app.block_style(Blocks::ModuleTable));
        assert_eq!(app.unselected_style, app.block_style(Blocks::Activities));
    }
}
