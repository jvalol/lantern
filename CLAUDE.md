# lantern

A dark maze and two lamps to light it with. The seventh game on `blitzkit`. The
dependency is the published crate, overridden by the engine checkout at
`../blitzkit` when built inside this project folder.

## Build and test

Requires Rust 1.87 or newer, the engine's MSRV.

```
cargo build
cargo test
cargo run
cargo clippy
cargo fmt
```

## How work happens here

Behavior changes are spec driven:

1. **Write the spec first.** Copy `specs/TEMPLATE.md` to `specs/NNNN-short-name.md`
   and fill it in. Say what the game does, not how the code does it.
2. **Make the acceptance criteria testable.** Each one names the test that proves
   it, or goes under "Verified by hand" when it needs a window.
3. **Write the tests, then the code.** `cargo test` passes before a commit.
4. **Update the spec when behavior changes.** A spec that disagrees with the game
   is a bug in the spec.

## Layout

- `src/main.rs` — nothing yet. The game is a spec and a rulebook so far.
- `src/maze.rs` — the grid, the walls, and carving one route between any two
  cells.
- `src/lamps.rs` — the two lamps, and putting them down and taking them up.

## Why two lamps

blitzkit spec 0022 casts shadows from at most two point lights, because each one
is six passes over the scene. That is a ceiling chosen for cost and it could
move, so `lamps::LAMPS` names it here rather than reaching for the engine's
constant.

Two is right for the game anyway. One is a torch and no decision. Three is
enough to light ahead and behind and still hold a spare, and the choosing stops.

This is the first game to use point lights at all. Until now only the `cubes`
example did, so spec 0022 has never had a game behind it.
