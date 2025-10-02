use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Stylize, palette::tailwind},
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, Padding, Paragraph, Widget,
    },
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

#[derive(Default, Clone, Copy, Display, FromRepr, EnumIter)]
pub enum SelectedTab {
    #[default]
    Diffs,
    OpenAI,
    Claude,
    Gemini,
}

impl Widget for SelectedTab {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let items = Vec::new();
        let content = "Hello Friends!";
        self.layout(area, buf, items, content);
    }
}

impl SelectedTab {
    /// Get the previous tab, if there is no previous tab return the current tab.
    pub fn previous(self) -> Self {
        let current_index: usize = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }

    /// Get the next tab, if there is no next tab return the current tab.
    pub fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    pub fn find_tab(self, tab: usize) -> Self {
        Self::from_repr(tab).unwrap_or(self)
    }

    pub fn title(self) -> Line<'static> {
        let idx = self as usize + 1;

        format!(" [{idx}] {self} ")
            .fg(tailwind::SLATE.c200)
            .bg(self.palette().c900)
            .into()
    }

    pub fn layout(
        self,
        area: Rect,
        buf: &mut Buffer,
        items: Vec<&str>,
        content: &str,
    ) {
        let horizontal = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ]);
        let [list_area, paragraph_area] = horizontal.areas(area);

        let items: Vec<ListItem> =
            items.iter().map(|item| ListItem::new(*item)).collect();

        let title = match self {
            SelectedTab::Diffs => "Diffs",
            _ => "Commits",
        };

        let list = List::new(items).block(
            Block::bordered()
                .title(title)
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1))
                .border_style(self.palette().c700),
        );

        list.render(list_area, buf);

        let paragraph = Paragraph::new(content).block(
            Block::bordered()
                .title("Content")
                .borders(Borders::ALL)
                .padding(Padding::horizontal(1))
                .border_style(self.palette().c700),
        );

        paragraph.render(paragraph_area, buf);
    }

    pub const fn palette(self) -> tailwind::Palette {
        match self {
            Self::Diffs => tailwind::GREEN,
            Self::OpenAI => tailwind::GRAY,
            Self::Claude => tailwind::ORANGE,
            Self::Gemini => tailwind::CYAN,
        }
    }
}
