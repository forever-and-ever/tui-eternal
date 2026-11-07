tui-eternal: house look and conduct for ratatui apps; `DESIGN.md` owns the design language

## gate
- `./check.py check` after edits; `./check.py verify` runs same gate without writes
- Release only after `cargo semver-checks` passes against last published version

## invariants
- Public API change: migrate adopters `../memview` and `../slskd-1337` in same campaign, deleting their local copies

## routes
- Changing palette roles, chrome, key notation or help conduct → `DESIGN.md`
