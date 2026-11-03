# 0007 A picture of a candle left behind

**Status:** implemented
**Date:** 2026-11-03

## Goal

The frame lantern puts on its readme is held against a picture of what that
frame should be, so a change to the maze, the candle or the light it throws
is seen rather than shipped.

## Why it belongs here

The specs before this one say how the maze is carved, how a candle burns,
what the braziers do and what the minimap records. Every test of them is
arithmetic: a cell, a distance, a flame's remaining seconds. This game is
about what you can see from where you stand, and nothing here could see it.

The engine's spec 0076 made the rest one call.

**The staging was already here.** `--screenshot` puts a candle down, walks
away from it for `POSED_AWAY` seconds and turns round, because a dark maze
photographed from where you start is a picture of the dark.

**What was not here was the same maze twice.** It was carved from entropy,
so every regenerated screenshot was a different maze and a reference could
never have held against one.

## Behavior

**`LanternGame::new` takes `staged`, and a staged run carves from
`POSED_SEED`.** A real run carves from entropy, as it did. Only the
photographed path is pinned.

This changes what `refresh-screenshots` photographs, so lantern's committed
screenshot moves the next time it is taken.

**One `update`, at no time at all.** The pose walks on its own fixed
sixtieths inside `new`, so by the time the test has a renderer the player is
already turned round. The update lights the lamps from where they ended up,
and a `dt` of nought keeps the flame where it is.

**The picture is the 3D.** The engine's harness leaves the overlay empty, so
the readout and the minimap are not in shot.

**It is the engine's size**, 320 by 240, said once in `picture::SIZE`.

## Acceptance criteria

- A candle left behind matches its
  picture. — `lantern_game::tests::a_candle_left_behind_looks_right`

### Verified by hand

- **The picture itself.** The candle standing on the floor with its flame
  above it, the light falling off into the dark along the passage, and the
  walls lit where it reaches them. That falloff is the whole game and this
  is the only test that can see it.
- **A control.** The seed moved from 11 to 12 and the test failed on 23.2%
  of its pixels, which is what standing in a different corridor looks like.

## Out of scope

**Which seed.** Any of them carves a maze. This one is the first tried.

**The readout and the minimap.** Both are 2D and the engine's harness draws
neither. The minimap has its own tests.
