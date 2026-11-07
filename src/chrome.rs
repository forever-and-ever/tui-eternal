//! Chrome: the rounded panels that frame an app, and the badge that names it.

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType};

use crate::palette::{ACCENT, BG, FG, MUTED, PAGE};

/// A rounded panel named `name`, its border and name lit while it holds the keys; a [`tally`]
/// may follow as a further title.
pub fn panel(name: &str, focused: bool) -> Block<'static> {
    let tint = if focused { ACCENT } else { FG };
    rim(focused).title(Span::styled(
        format!(" {name} "),
        tint.fg().add_modifier(Modifier::BOLD),
    ))
}

/// The masthead's rim: a rounded panel with no name, around the header bar that the badge leads.
/// It never holds the keys.
pub fn masthead() -> Block<'static> {
    rim(false)
}

/// The app's badge, its name reversed out of the accent, which leads the header bar.
pub fn badge(name: &str) -> Span<'static> {
    Span::styled(
        format!(" {name} "),
        BG.fg().bg(ACCENT.color()).add_modifier(Modifier::BOLD),
    )
}

/// A panel's tally, such as a count, for its top-right corner.
pub fn tally(text: &str) -> Line<'static> {
    Line::styled(format!(" {text} "), MUTED.fg()).right_aligned()
}

fn rim(focused: bool) -> Block<'static> {
    let border = if focused { ACCENT } else { MUTED };
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(border.fg())
        .style(PAGE)
}
