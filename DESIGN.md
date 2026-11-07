# Design language

Every house terminal app adopts all of tui-eternal and keeps no local rival: no
second palette, panel, terminal owner, recipe pick, keymap or help card. A
pattern joins the crate once two apps share it, and joining it migrates both.

## Palette

Eight roles, one color each, on a dark canvas. A role names what a color
means, never how it looks.

| Role | Means |
|---|---|
| `BG` | the canvas |
| `FG` | body text |
| `MUTED` | labels, units, explanations, and borders that do not hold the keys |
| `ACCENT` | whatever holds the keys or the eye: the focused panel, the selection, names, the badge |
| `HOT` | faults, alarms and consequences that destroy |
| `GOLD` | caution and waiting, and help's headings |
| `KEY` | keys, wherever a legend or help names them |
| `CURSOR` | the band behind the row under the cursor, in the panel that holds the keys |

A color outside these roles, such as a progress bar's fill, belongs to the app
that draws it.

## Chrome

- The top row is the header bar, in a `masthead()`: the app's `badge()`, then
  the app's state. Activity readouts sit flush right in it.
- A panel names itself top-left; its `tally()`, such as a count, sits
  top-right. Its border
  and name light in `ACCENT` while it holds the keys. Every other panel is
  `MUTED`.
- The bottom row is the legend: the hints of the bindings in force, most
  specific first, ending with `? help` and the way out.

## Activity

Each process picks one stock shimmer recipe at random, among those that draw in
the base glyphs and do not listen. A shimmer moves only while work is under
way and keeps the pace of that work.

## Keys and help

- An app declares its keys once, as sections of bindings. The keymap routes
  keys through the sections in force, the help card lists them, and the legend
  shows their hints.
- Sections run from most specific to least: the view in force, then its
  navigation, then the application.
- `?` raises help wherever no text field holds the keys; Esc, `?` or `q`
  dismisses it. The card ends with the app's notes: facts that no binding
  states.
- Ctrl-C quits from anywhere, help included.
- Notation: characters as typed (`j`, `G`, `/`); `Enter`, `Esc`, `Tab`,
  `Shift-Tab`, `Backspace`, `PgUp`, `PgDn`, `Home` and `End`; arrows as `↑ ↓ ←
  →`; `Ctrl-C`; sequences unspaced (`gg`, `zo`). `/` joins the keys for the
  actions a description names, in its order (`j/k`: down, then up); a space
  separates alternatives (`q Ctrl-C`).

## Terminal

A `Screen` holds the terminal in raw mode on the alternate screen with its
cursor hidden, and enables only the reports the app asks for. Dropping it, a
panic or a hangup restores the terminal exactly once, before any panic report
prints. After a hangup the restoring writes are best effort, never a second
panic.
