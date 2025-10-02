use ratatui::style::{Modifier, Style, palette::tailwind::SLATE};

use crate::ui::UI;

const SELECTED_STYLE: Style =
    Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);

impl UI {}
