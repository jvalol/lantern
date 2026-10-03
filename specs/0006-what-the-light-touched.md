# 0006 What the light touched

**Status:** implemented
**Date:** 2026-09-29

## Goal

A hundred cells of dark maze is more than anyone holds in their head. What goes
wrong is not getting lost, it is walking a corridor for the third time without
knowing it. Somewhere to put what you have already seen.

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
you are in it or not, and what it lights is written down. Which is the reason
to build the map on light rather than footsteps. The two candles were already a
decision about where to see, and now about what to keep.

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

**It is drawn in the bottom right**, as quads: a card the size of the maze, a
cell on it for every cell of the maze, the recorded ones bright, the walls
between recorded cells as thin ones, a mark for you and which way you face, and
a mark for each candle you have set down. Nothing for the way out.

The card and the unlit cells are a reversal. This spec used to say nothing
under it and no frame, on the grounds that a map of five cells inside an
outline of 256 is mostly empty rectangle. That was right about the rectangle
and wrong about what it costs. A map of only what you have seen has no shape
until you have seen a lot. Three pale cells with no extent and no boundary do
not read as a map. Jake called them a mark on the wall, 2026-10-01. The empty
rectangle is the map saying how much there is left.

So it now says two things it did not: how big the maze is, and where in it you
are standing. It still says nothing about its walls, its corridors, the way out,
or anything else you have not lit. Those two facts are worth a map that reads as
one, and neither of them shortens the walk.

The card carries the padding, so the grid has a margin and the lit cells never
touch the edge. The unlit cells are drawn one per cell rather than as one flat
tone, which costs nothing now and leaves room for the unexplored part to differ
cell by cell later.

**A wall lies on the edge between two cells**, half a cell out from the middle
of either. A quad sits on its middle and so does a cell, so a wall drawn at the
cell's own place runs through the middle of it. A cell walled north and west
came out as a plus sign. Standing at a dead end the map showed a cross. The
candle marks and your own arrow had the same fault in the other direction, half
a cell into the corner of their cells, which is most of why the arrow never
read clearly.

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
- And lies on the edge between them, not through either. — `minimap::tests::a_wall_lies_on_the_edge_between_two_cells`
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
Anything you have not lit beyond the bare fact that a cell is there: the way
out, the braziers you have not reached, which of the dark is corridor and which
is wall. Scrolling or zooming it. A label.
