//! Keys: an app's shortcuts, declared once, so that the keymap that routes them, the help that
//! explains them and the legend that labels them cannot disagree.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::text::Span;

use crate::palette::{KEY, MUTED};

/// Modifiers that make a key a different command: every one but Shift, which belongs to the
/// character it types.
const COMMAND: KeyModifiers = KeyModifiers::CONTROL
    .union(KeyModifiers::ALT)
    .union(KeyModifiers::SUPER)
    .union(KeyModifiers::HYPER)
    .union(KeyModifiers::META);

/// What the user types to invoke a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shortcut {
    /// A character typed without a command modifier.
    Char(char),
    /// A named key, such as an arrow, Enter or Esc, pressed without a command modifier.
    Code(KeyCode),
    /// A character typed with Ctrl held.
    Ctrl(char),
    /// Two characters typed in turn, as Vim's `gg`.
    Seq(char, char),
}

impl Shortcut {
    /// Whether pressing `key` completes the shortcut alone: never for a sequence, which a
    /// [`Keymap`] completes across two keys.
    pub fn matches(self, key: KeyEvent) -> bool {
        let bare = !key.modifiers.intersects(COMMAND);
        key.kind != KeyEventKind::Release
            && match self {
                Self::Char(character) => bare && key.code == KeyCode::Char(character),
                Self::Code(code) => bare && key.code == code,
                Self::Ctrl(character) => {
                    key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char(character)
                }
                Self::Seq(..) => false,
            }
    }
}

/// A shortcut and the action it triggers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Command<A>(pub Shortcut, pub A);

/// A legend entry: a binding's keys, abbreviated, then what they do in a word or two.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hint {
    /// The keys, as the legend abbreviates them.
    pub keys: &'static str,
    /// What they do.
    pub label: &'static str,
}

/// Shortcuts that do one thing, declared once for the keymap, the help and the legend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding<A: 'static> {
    /// The shortcuts as help lists them, such as `j/k ↓/↑`.
    pub keys: &'static str,
    /// What they do, as help explains it.
    pub does: &'static str,
    /// The legend's entry, for the bindings worth one.
    pub hint: Option<Hint>,
    /// The commands the keymap routes; none for a gesture it does not, such as the mouse wheel.
    pub commands: &'static [Command<A>],
}

impl<A> Binding<A> {
    /// `keys` that do `does` through `commands`, with no legend entry.
    pub const fn new(
        keys: &'static str,
        does: &'static str,
        commands: &'static [Command<A>],
    ) -> Self {
        Self {
            keys,
            does,
            hint: None,
            commands,
        }
    }

    /// The binding, with the legend entry `keys`, then `label`.
    pub const fn hint(self, keys: &'static str, label: &'static str) -> Self {
        Self {
            hint: Some(Hint { keys, label }),
            ..self
        }
    }
}

/// A titled group of bindings: one scope of the keymap, and one block of help.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section<A: 'static> {
    /// The scope in the app's words, such as a pane's name.
    pub title: &'static str,
    /// The bindings in force within the scope.
    pub bindings: &'static [Binding<A>],
}

/// What one key amounted to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolution<A> {
    /// No binding in force answers it.
    Ignored,
    /// It began a sequence, which the next key completes or abandons.
    Pending,
    /// It completed a command.
    Command(A),
}

/// Routes keys through the sections in force, holding the first key of a sequence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Keymap {
    /// The first key of a sequence under way.
    prefix: Option<char>,
}

impl Keymap {
    /// What `key` amounts to under `sections`, searched in order: the first command it
    /// completes, else the start of a sequence. A release never counts; the key after a
    /// sequence's first either completes it or is spent abandoning it.
    pub fn resolve<A: Copy>(&mut self, sections: &[Section<A>], key: KeyEvent) -> Resolution<A> {
        if key.kind == KeyEventKind::Release {
            return Resolution::Ignored;
        }
        let typed = match key.code {
            KeyCode::Char(character) if !key.modifiers.intersects(COMMAND) => Some(character),
            _ => None,
        };
        if let Some(first) = self.prefix.take() {
            return typed
                .and_then(|second| {
                    commands(sections).find(|command| command.0 == Shortcut::Seq(first, second))
                })
                .map_or(Resolution::Ignored, |command| {
                    Resolution::Command(command.1)
                });
        }
        if let Some(command) = commands(sections).find(|command| command.0.matches(key)) {
            return Resolution::Command(command.1);
        }
        if let Some(first) = typed.filter(|&first| {
            commands(sections)
                .any(|command| matches!(command.0, Shortcut::Seq(lead, _) if lead == first))
        }) {
            self.prefix = Some(first);
            return Resolution::Pending;
        }
        Resolution::Ignored
    }

    /// Abandons a sequence under way.
    pub fn cancel(&mut self) {
        self.prefix = None;
    }
}

/// Every command of `sections`, in order.
fn commands<A>(sections: &[Section<A>]) -> impl Iterator<Item = &'static Command<A>> {
    sections
        .iter()
        .flat_map(|section| section.bindings)
        .flat_map(|binding| binding.commands)
}

/// The footer legend for `sections`: each hint's keys, then its label, in order.
pub fn legend<A>(sections: &[Section<A>]) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for Hint { keys, label } in sections
        .iter()
        .flat_map(|section| section.bindings)
        .filter_map(|binding| binding.hint)
    {
        if !spans.is_empty() {
            spans.push(Span::raw("  "));
        }
        spans.extend([
            Span::styled(keys, KEY.fg()),
            Span::raw(" "),
            Span::styled(label, MUTED.fg()),
        ]);
    }
    spans
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use super::{Binding, Command, Keymap, Resolution, Section, Shortcut};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Act {
        Top,
        Bottom,
        Quit,
    }

    const TREE: Section<Act> = Section {
        title: "Tree",
        bindings: &[Binding::new(
            "gg/G",
            "jump to the first or last row",
            &[
                Command(Shortcut::Seq('g', 'g'), Act::Top),
                Command(Shortcut::Char('G'), Act::Bottom),
            ],
        )],
    };

    const APPLICATION: Section<Act> = Section {
        title: "Application",
        bindings: &[Binding::new(
            "q Ctrl-C",
            "quit",
            &[
                Command(Shortcut::Char('q'), Act::Quit),
                Command(Shortcut::Ctrl('c'), Act::Quit),
            ],
        )],
    };

    fn key(character: char, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(character), modifiers)
    }

    #[test]
    fn sequences_complete_only_where_declared() {
        let mut keymap = Keymap::default();
        let both = [TREE, APPLICATION];
        let plain = |character| key(character, KeyModifiers::NONE);
        assert_eq!(keymap.resolve(&both, plain('g')), Resolution::Pending);
        // Terminals that report releases report the first key's release before the second key.
        let mut release = plain('g');
        release.kind = KeyEventKind::Release;
        assert_eq!(keymap.resolve(&both, release), Resolution::Ignored);
        assert_eq!(
            keymap.resolve(&both, plain('g')),
            Resolution::Command(Act::Top)
        );
        // A key that completes no sequence is spent abandoning it.
        assert_eq!(keymap.resolve(&both, plain('g')), Resolution::Pending);
        assert_eq!(keymap.resolve(&both, plain('q')), Resolution::Ignored);
        assert_eq!(
            keymap.resolve(&both, plain('q')),
            Resolution::Command(Act::Quit)
        );
        assert_eq!(
            keymap.resolve(&both, key('q', KeyModifiers::CONTROL)),
            Resolution::Ignored
        );
        assert_eq!(
            keymap.resolve(&both, key('c', KeyModifiers::CONTROL)),
            Resolution::Command(Act::Quit)
        );
        assert_eq!(
            keymap.resolve(&[APPLICATION], plain('g')),
            Resolution::Ignored
        );
    }
}
