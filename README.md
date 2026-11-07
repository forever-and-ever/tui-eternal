# tui-eternal

One look and one conduct for ratatui apps: a palette of eight roles, rounded
chrome, a terminal owner that restores the terminal however the app ends, the
shimmer that dresses work under way, and keys declared once, so that routing a
key, the `?` help card that explains it and the footer legend that labels it
cannot disagree.

[DESIGN.md](DESIGN.md) states the design language. The crate documentation
carries a complete app loop.

- `palette`: `BG`, `FG`, `MUTED`, `ACCENT`, `HOT`, `GOLD`, `KEY` and `CURSOR`, with
  `PAGE`, body text on the canvas, and `SHIMMER`, the page as
  [libshimmer](https://crates.io/crates/libshimmer) blends against it.
- `panel`, `tally`, `masthead`, `badge`: rounded panels lit while they hold the
  keys, their top-right counts, the header bar's rim, and the app's name
  reversed out of the accent.
- `Screen`: raw mode and the alternate screen, with the reports an app asks
  for. Dropping it, a panic or a hangup restores the terminal exactly once.
- `Ember`: one random stock libshimmer recipe per process, and the shimmers it
  lights.
- `Section`, `Binding`, `Command`, `Shortcut`: an app's keys. `Keymap` routes
  them, two-key sequences included; `Help` is the scrolling card that `?`
  raises; `legend` is the footer.

## License

MIT-0
