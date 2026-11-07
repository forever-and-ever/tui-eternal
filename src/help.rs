//! Help: the card `?` raises over an app, listing the bindings in force and the app's notes.

use std::cell::Cell;
use std::mem;

use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Clear, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use unicode_width::{UnicodeWidthChar as _, UnicodeWidthStr as _};

use crate::chrome::panel;
use crate::keys::{Binding, Command, Keymap, Resolution, Section, Shortcut, legend};
use crate::palette::{ACCENT, FG, GOLD, KEY, MUTED};

/// The widest the card's text runs, in cells, so that its lines stay readable on wide terminals.
const MEASURE: u16 = 76;
/// The card's border and padding: columns, then lines.
const CHROME: (u16, u16) = (6, 4);
/// Cells between the key column and the descriptions.
const GUTTER: usize = 2;
/// Lines one wheel detent scrolls.
const DETENT: i16 = 3;

/// The help card while it is up.
#[derive(Debug, Default)]
pub struct Help {
    /// Lines scrolled past the top.
    scroll: u16,
    /// What the last frame showed, which bounds scrolling and sizes a page. Drawing reports it
    /// through a shared borrow, as apps draw from a shared borrow of their state.
    view: Cell<View>,
}

#[derive(Clone, Copy, Debug, Default)]
struct View {
    /// Lines the card shows at once.
    height: u16,
    /// The farthest the card scrolls.
    reach: u16,
}

/// Whether the help card stays up after a key.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The card stays up.
    Keep,
    /// The card goes: the app drops it.
    Dismiss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Turn {
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
    Close,
}

/// Scrolling, which the card's legend offers only while its text overflows.
const SCROLL: Section<Turn> = Section {
    title: "Help",
    bindings: &[
        Binding::new(
            "↑/↓ k/j",
            "scroll a line",
            &[
                Command(Shortcut::Code(KeyCode::Up), Turn::Up),
                Command(Shortcut::Code(KeyCode::Down), Turn::Down),
                Command(Shortcut::Char('k'), Turn::Up),
                Command(Shortcut::Char('j'), Turn::Down),
            ],
        )
        .hint("↑↓", "scroll"),
        Binding::new(
            "PgUp/PgDn",
            "scroll a page",
            &[
                Command(Shortcut::Code(KeyCode::PageUp), Turn::PageUp),
                Command(Shortcut::Code(KeyCode::PageDown), Turn::PageDown),
            ],
        ),
        Binding::new(
            "Home/End g/G",
            "scroll to the top or the bottom",
            &[
                Command(Shortcut::Code(KeyCode::Home), Turn::Top),
                Command(Shortcut::Code(KeyCode::End), Turn::Bottom),
                Command(Shortcut::Char('g'), Turn::Top),
                Command(Shortcut::Char('G'), Turn::Bottom),
            ],
        ),
    ],
};

const CLOSE: Section<Turn> = Section {
    title: "Help",
    bindings: &[Binding::new(
        "Esc ? q",
        "close",
        &[
            Command(Shortcut::Code(KeyCode::Esc), Turn::Close),
            Command(Shortcut::Char('?'), Turn::Close),
            Command(Shortcut::Char('q'), Turn::Close),
        ],
    )
    .hint("Esc", "close")],
};

const OVERFLOWING: &[Section<Turn>] = &[SCROLL, CLOSE];
const FITTING: &[Section<Turn>] = &[CLOSE];

impl Help {
    /// Answers `key`: the card scrolls, or goes on Esc, `?` or `q`, and ignores any other key.
    pub fn key(&mut self, key: KeyEvent) -> Verdict {
        let View { height, reach } = self.view.get();
        let page = i16::try_from(height.saturating_sub(1).max(1)).unwrap_or(i16::MAX);
        match Keymap::default().resolve(OVERFLOWING, key) {
            Resolution::Command(Turn::Close) => return Verdict::Dismiss,
            Resolution::Command(Turn::Up) => self.shift(-1),
            Resolution::Command(Turn::Down) => self.shift(1),
            Resolution::Command(Turn::PageUp) => self.shift(-page),
            Resolution::Command(Turn::PageDown) => self.shift(page),
            Resolution::Command(Turn::Top) => self.scroll = 0,
            Resolution::Command(Turn::Bottom) => self.scroll = reach,
            Resolution::Ignored | Resolution::Pending => {}
        }
        Verdict::Keep
    }

    /// Answers the mouse, whose wheel scrolls the card; whether the card took `mouse`.
    pub fn mouse(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.shift(-DETENT),
            MouseEventKind::ScrollDown => self.shift(DETENT),
            _ => return false,
        }
        true
    }

    /// Scrolls `lines`, upward when negative, within the reach the last frame showed.
    fn shift(&mut self, lines: i16) {
        let reach = self.view.get().reach;
        self.scroll = self
            .scroll
            .min(reach)
            .saturating_add_signed(lines)
            .min(reach);
    }

    /// Draws the card centered over the frame: `sections`, the bindings in force, most specific
    /// first, then `notes`, facts about the app that no binding states.
    pub fn render<A>(&self, frame: &mut Frame<'_>, sections: &[Section<A>], notes: &[&str]) {
        let area = frame.area();
        let (columns, rows) = CHROME;
        // A cell of margin keeps the card off the frame's edges.
        let room = area.width.saturating_sub(columns + 2).min(MEASURE);
        let measure = natural(sections, notes).min(usize::from(room)).max(1);
        let lines = page(sections, notes, measure);
        let card = centered(
            area,
            cells(measure).saturating_add(columns),
            cells(lines.len())
                .saturating_add(rows)
                .min(area.height.saturating_sub(2)),
        );
        let height = card.height.saturating_sub(rows);
        let reach = cells(lines.len()).saturating_sub(height);
        let scroll = self.scroll.min(reach);
        self.view.set(View { height, reach });

        let mut hints = vec![Span::raw(" ")];
        hints.extend(legend(if reach > 0 { OVERFLOWING } else { FITTING }));
        hints.push(Span::raw(" "));
        let block = panel("Help", true)
            .title_bottom(Line::from(hints).right_aligned())
            .padding(Padding::new(2, 2, 1, 1));
        let inner = block.inner(card);
        frame.render_widget(Clear, card);
        frame.render_widget(block, card);
        let shown: Vec<Line<'static>> = lines
            .into_iter()
            .skip(usize::from(scroll))
            .take(usize::from(height))
            .collect();
        frame.render_widget(Paragraph::new(shown), inner);
        if reach > 0 {
            let track = Rect {
                x: card.right().saturating_sub(1),
                y: card.y.saturating_add(1),
                width: 1,
                height: card.height.saturating_sub(2),
            };
            let mut state = ScrollbarState::new(usize::from(reach) + 1)
                .position(usize::from(scroll))
                .viewport_content_length(usize::from(height));
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .track_style(ACCENT.fg())
                    .thumb_symbol("┃")
                    .thumb_style(FG.fg()),
                track,
                &mut state,
            );
        }
    }
}

/// The widest line the card would hold if nothing wrapped.
fn natural<A>(sections: &[Section<A>], notes: &[&str]) -> usize {
    let bindings = || sections.iter().flat_map(|section| section.bindings);
    let column = bindings().map(|binding| binding.keys.width()).max();
    let rows = bindings().map(|binding| column.unwrap_or(0) + GUTTER + binding.does.width());
    let headings = sections.iter().map(|section| section.title.width());
    let notes = notes.iter().map(|note| note.width() + 2);
    rows.chain(headings).chain(notes).max().unwrap_or(0)
}

/// The card's lines at `measure` cells: each section in force under its heading, then the notes.
fn page<A>(sections: &[Section<A>], notes: &[&str], measure: usize) -> Vec<Line<'static>> {
    let column = sections
        .iter()
        .flat_map(|section| section.bindings)
        .map(|binding| binding.keys.width())
        .max()
        .unwrap_or(0)
        .min(measure / 3);
    let mut lines = Vec::new();
    for section in sections
        .iter()
        .filter(|section| !section.bindings.is_empty())
    {
        heading(&mut lines, section.title);
        for binding in section.bindings {
            row(&mut lines, binding, column, measure);
        }
    }
    if !notes.is_empty() {
        heading(&mut lines, "Notes");
        for note in notes {
            let mut wrapped = wrap(note, measure.saturating_sub(2).max(1)).into_iter();
            lines.extend(wrapped.next().map(|first| {
                Line::from(vec![
                    Span::styled("· ", MUTED.fg()),
                    Span::styled(first, FG.fg()),
                ])
            }));
            lines.extend(
                wrapped.map(|line| Line::from(vec![Span::raw("  "), Span::styled(line, FG.fg())])),
            );
        }
    }
    lines
}

/// A section's heading, a blank line after whatever precedes it.
fn heading(lines: &mut Vec<Line<'static>>, title: &str) {
    if !lines.is_empty() {
        lines.push(Line::default());
    }
    lines.push(Line::styled(
        title.to_uppercase(),
        GOLD.fg().add_modifier(Modifier::BOLD),
    ));
}

/// A binding's keys, right-aligned in a column `column` cells wide, then what they do, wrapped
/// under itself. Keys wider than the column take a line of their own.
fn row<A>(lines: &mut Vec<Line<'static>>, binding: &Binding<A>, column: usize, measure: usize) {
    let keys = Span::styled(binding.keys, KEY.fg().add_modifier(Modifier::BOLD));
    let indent = column + GUTTER;
    let mut does = wrap(binding.does, measure.saturating_sub(indent).max(1)).into_iter();
    match column.checked_sub(binding.keys.width()) {
        Some(lead) => lines.push(Line::from(vec![
            Span::raw(" ".repeat(lead)),
            keys,
            Span::raw(" ".repeat(GUTTER)),
            Span::styled(does.next().unwrap_or_default(), FG.fg()),
        ])),
        None => lines.push(Line::from(keys)),
    }
    lines.extend(does.map(|line| {
        Line::from(vec![
            Span::raw(" ".repeat(indent)),
            Span::styled(line, FG.fg()),
        ])
    }));
}

/// `text` in lines at most `width` cells wide, broken at spaces, and within a word only where the
/// word is wider than a line.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for word in text.split_whitespace() {
        if used > 0 && used + 1 + word.width() > width {
            lines.push(mem::take(&mut line));
            used = 0;
        }
        if used > 0 {
            line.push(' ');
            used += 1;
        }
        let mut rest = word;
        while used + rest.width() > width {
            let cut = cut(rest, width - used);
            line.push_str(&rest[..cut]);
            lines.push(mem::take(&mut line));
            used = 0;
            rest = &rest[cut..];
        }
        line.push_str(rest);
        used += rest.width();
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

/// The byte length of the longest prefix of `word` at most `cells` wide, and of one character at
/// least, so that wrapping always advances.
fn cut(word: &str, cells: usize) -> usize {
    let mut used = 0;
    for (index, character) in word.char_indices() {
        used += character.width().unwrap_or(0);
        if used > cells {
            return if index == 0 {
                character.len_utf8()
            } else {
                index
            };
        }
    }
    word.len()
}

fn cells(count: usize) -> u16 {
    u16::try_from(count).unwrap_or(u16::MAX)
}

/// The `width` × `height` rectangle centered in `area`, clipped to it.
fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::{Help, Verdict};
    use crate::keys::{Binding, Section};

    const KEYS: Section<()> = Section {
        title: "Keys",
        bindings: &[
            Binding::new(
                "x",
                "a binding whose explanation wraps across several of the card's lines",
                &[],
            ),
            Binding::new(
                "y/z",
                "another binding, so that the card overflows a short terminal",
                &[],
            ),
        ],
    };

    const NOTES: &[&str] = &[
        "A note long enough to wrap, which pushes the card's last line further below its fold.",
        "omega",
    ];

    fn shown(terminal: &mut Terminal<TestBackend>, help: &Help) -> String {
        terminal
            .draw(|frame| help.render(frame, &[KEYS], NOTES))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect()
    }

    #[test]
    fn the_bottom_shows_the_last_note_and_scrolling_back_answers_at_once() {
        let press = |code| KeyEvent::new(code, KeyModifiers::NONE);
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        let mut help = Help::default();
        assert!(!shown(&mut terminal, &help).contains("omega"));
        assert_eq!(help.key(press(KeyCode::End)), Verdict::Keep);
        assert!(shown(&mut terminal, &help).contains("omega"));
        // Scrolling past the bottom must not bank lines that scrolling back then spends unseen.
        assert_eq!(help.key(press(KeyCode::Down)), Verdict::Keep);
        assert_eq!(help.key(press(KeyCode::Up)), Verdict::Keep);
        assert!(!shown(&mut terminal, &help).contains("omega"));
        assert_eq!(help.key(press(KeyCode::Esc)), Verdict::Dismiss);
    }
}
