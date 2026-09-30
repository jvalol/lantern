# 0006 What the light touched

**Status:** implemented
**Date:** 2026-09-29

## Goal

A hundred cells of dark maze is more than anyone holds in their head, and the
thing that goes wrong is not getting lost, it is walking a corridor for the
third time without knowing it. Somewhere to put what you have already seen.

## Behavior

**A map of the whole maze would delete the game**, so this is not one. It shows
only what your own light has fallen on, and it never shows the way out. Spec
0003's hints are foresight; this is memory. They do not overlap and neither
replaces the other.

**Your light, not anybody's.** A candle records: the ones in your hands as you
walk, and the ones you set down. The braziers do not, because they were burning
before you arrived and are not yours to spend. Standing in a brazier's cell with
a candle in hand records it like anywhere else, so there is no exception to
explain, only a rule about whose light it is.

**A candle you leave behind goes on recording.** It lights its corridor whether
you are in it or not, and what it lights is written down. This is the whole
reason to build the map on light rather than on footsteps: the two candles were
already a decision about where to see, and now they are a decision about what to
keep.

**Walking dark is walking unrecorded.** Both candles set down at the far end of
the maze and you cross it in the dark, and the map learns nothing from the
crossing. That is the cost, and it is the same cost the game already charges.

**How far a candle reaches, on the map.** A cell is recorded when a candle is
within `maze::CELLS` steps of it through open sides, `LAMP_RANGE / CELL` of
them, so the reach on the map comes from the reach in the world. Steps through
open sides rather than distance through the air: light goes round a corner
along a corridor and does not go through a wall, and the maze already knows
which is which.

This is an approximation. A cell three steps along a bending corridor is
recorded although a lamp would barely reach it, and the alternative is line of
sight, which costs more than the map is worth.

**What is written stays written.** Walking away does not forget, and picking a
candle back up does not unwrite what it lit.

**It is drawn in the bottom right**, as quads: one per recorded
cell, the walls between recorded cells as thin ones, a mark for you and which
way you face, and a mark for each candle you have set down. Nothing for the way
out.

Nothing under it. A panel behind it reads as another thing on the screen rather
than as the ground the map sits on, and the screen already has enough on it.

No frame and no label either. A map of five cells inside an outline of two
hundred and fifty six is mostly empty rectangle, and saying so in a line of text
beside it only adds a third thing to read. What makes it legible is being big
enough and being where you expect it, not being annotated.

**M shows and hides it**, the way H steps the hints.

## Acceptance criteria

- A cell your candle has lit is recorded. — `minimap::tests::a_lit_cell_is_recorded`
- A cell nothing has lit is not. — `minimap::tests::the_dark_is_not_recorded`
- Light does not record through a wall. — `minimap::tests::a_wall_stops_it_recording`
- It reaches as far on the map as a lamp reaches in the world. — `minimap::tests::it_reaches_as_far_as_a_lamp_does`
- A candle left behind goes on recording. — `minimap::tests::a_candle_left_behind_keeps_recording`
- A brazier's light is not yours and records nothing. — `minimap::tests::somebody_elses_light_records_nothing`
- What is recorded stays recorded. — `minimap::tests::what_is_written_stays_written`
- And picking a candle up does not unwrite it. — `minimap::tests::taking_a_candle_back_does_not_unwrite_it`
- The way out is never on it. — `minimap::tests::the_way_out_is_not_on_the_map`
- Only recorded cells are drawn. — `minimap::tests::only_what_is_recorded_is_drawn`
- A wall is drawn only between two recorded cells. — `minimap::tests::a_wall_needs_both_sides_recorded`
- You are where you are on it. — `minimap::tests::you_are_where_you_are`
- It sits clear of the text, whatever the window. — `minimap::tests::it_keeps_out_of_the_way_of_the_text`
- M shows it and hides it. — `lantern_game::tests::the_map_key_shows_and_hides_it`
- Carrying a candle writes down where you are standing. — `lantern_game::tests::carrying_a_candle_writes_where_you_are`

### Verified by hand

- Setting a candle down in a junction and walking on, the junction is still on
  the map when you come back to it.
- Crossing the maze with both candles behind you writes nothing, and it is
  obvious afterwards which crossing you made in the dark.
- The map never says which way is out.

## Out of scope

Line of sight. A map of where you have walked rather than where you have seen.
Anything on it that you have not lit: the way out, the braziers you have not
reached, the shape of the maze beyond your light. Scrolling or zooming it.
