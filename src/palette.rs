//! The palette: eight roles, one color each, on one dark canvas.
//!
//! A role names what a color means, never how it looks. Colors outside these roles, such as a
//! progress bar's fill, belong to the app that draws them.

use libshimmer::{Palette, Rgb};
use ratatui::style::{Color, Style};

/// A 24-bit color: red, green and blue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tint(pub [u8; 3]);

impl Tint {
    /// The color as ratatui draws it.
    pub const fn color(self) -> Color {
        let Self([red, green, blue]) = self;
        Color::Rgb(red, green, blue)
    }

    /// The color as libshimmer blends it.
    pub const fn rgb(self) -> Rgb {
        let Self([red, green, blue]) = self;
        Rgb::new(red, green, blue)
    }

    /// Text in this color.
    pub const fn fg(self) -> Style {
        Style::new().fg(self.color())
    }

    /// Cells filled with this color.
    pub const fn bg(self) -> Style {
        Style::new().bg(self.color())
    }
}

/// The canvas under everything.
pub const BG: Tint = Tint([12, 17, 24]);
/// Body text.
pub const FG: Tint = Tint([221, 227, 234]);
/// Secondary text, such as labels, units and explanations, and the borders of panels that do not
/// hold the keys.
pub const MUTED: Tint = Tint([129, 145, 160]);
/// Whatever holds the keys or the eye: the focused panel, the selection, names and the badge.
pub const ACCENT: Tint = Tint([64, 184, 173]);
/// Faults, alarms and consequences that destroy.
pub const HOT: Tint = Tint([227, 116, 94]);
/// Caution and waiting, and the headings of help.
pub const GOLD: Tint = Tint([236, 180, 71]);
/// Keys, wherever a legend or help names them.
pub const KEY: Tint = Tint([211, 218, 226]);
/// The band behind the row under the cursor, in the panel that holds the keys.
pub const CURSOR: Tint = Tint([28, 44, 61]);

/// Body text on the canvas: the style every frame starts from.
pub const PAGE: Style = FG.fg().bg(BG.color());

/// The page as libshimmer blends against it.
pub const SHIMMER: Palette = Palette {
    fg: FG.rgb(),
    bg: BG.rgb(),
};
