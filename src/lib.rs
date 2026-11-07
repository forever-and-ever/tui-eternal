//! One look and one conduct for the house's terminal apps, drawn with ratatui.
//!
//! - [`palette`]: eight color roles on one dark canvas.
//! - [`panel`], [`tally`], [`masthead`] and [`badge`]: rounded chrome, lit where the keys go.
//! - [`Screen`]: the terminal in raw mode on the alternate screen, restored however the app ends.
//! - [`Ember`]: the stock shimmer that dresses work under way.
//! - [`Section`]s of [`Binding`]s, declared once: the [`Keymap`] routes them, the [`Help`] card
//!   explains them and the [`legend`] labels them, so the three cannot disagree.
//!
//! `DESIGN.md` states the design language these pieces carry.
//!
//! ```no_run
//! use crossterm::event::{self, Event};
//! use ratatui::text::Line;
//! use ratatui::widgets::Paragraph;
//! use tui_eternal::{
//!     Binding, Command, Help, Keymap, Reports, Resolution, Screen, Section, Shortcut, Verdict,
//!     legend, panel,
//! };
//!
//! #[derive(Clone, Copy)]
//! enum Action {
//!     Help,
//!     Quit,
//! }
//!
//! const APPLICATION: Section<Action> = Section {
//!     title: "Application",
//!     bindings: &[
//!         Binding::new("?", "show this help", &[Command(Shortcut::Char('?'), Action::Help)])
//!             .hint("?", "help"),
//!         Binding::new("q", "quit", &[Command(Shortcut::Char('q'), Action::Quit)])
//!             .hint("q", "quit"),
//!     ],
//! };
//!
//! fn main() -> std::io::Result<()> {
//!     let mut screen = Screen::open(Reports::default())?;
//!     let mut keymap = Keymap::default();
//!     let mut help: Option<Help> = None;
//!     loop {
//!         screen.draw(|frame| {
//!             let footer = Paragraph::new(Line::from(legend(&[APPLICATION])));
//!             frame.render_widget(footer.block(panel("Demo", true)), frame.area());
//!             if let Some(help) = &help {
//!                 help.render(frame, &[APPLICATION], &[]);
//!             }
//!         })?;
//!         let Event::Key(key) = event::read()? else {
//!             continue;
//!         };
//!         if let Some(card) = &mut help {
//!             if card.key(key) == Verdict::Dismiss {
//!                 help = None;
//!             }
//!             continue;
//!         }
//!         match keymap.resolve(&[APPLICATION], key) {
//!             Resolution::Command(Action::Help) => help = Some(Help::default()),
//!             Resolution::Command(Action::Quit) => return Ok(()),
//!             Resolution::Ignored | Resolution::Pending => {}
//!         }
//!     }
//! }
//! ```

mod activity;
mod chrome;
mod help;
mod keys;
pub mod palette;
mod terminal;

pub use activity::{Ember, GLYPHS};
pub use chrome::{badge, masthead, panel, tally};
pub use help::{Help, Verdict};
pub use keys::{Binding, Command, Hint, Keymap, Resolution, Section, Shortcut, legend};
pub use terminal::{Reports, Screen};
