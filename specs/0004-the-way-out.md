# 0004 The way out

**Status:** draft
**Date:** 2026-09-29

## Goal

Everything calls it the way out and there is nothing there. The exit is the cell
furthest from the start, which is a number, not a place: no door, no opening, no
mark. You walk into an alcove that looks like every other dead end and the
readout changes. The first thing anyone asks on reaching it is whether that was
it.

## Behavior

**The way out is on the edge of the maze.** It was whichever cell happened to be
furthest from the start, which is usually somewhere in the middle, and a cell in
the middle has nothing to be a door in. It is now the furthest cell that has an
outer wall to cut, so there is always an outside for it to lead to.

This was supposed to cost some of the walk and it barely does. Over forty
mazes the way out averages seventy steps where the furthest cell of all
averages seventy three, and the shortest of the forty is forty four against
forty six. The edge of a sixteen by sixteen maze is most of its far half.

Those numbers are lower than the ones this game used to report, because spec
0003 found `distances_from` walking depth first and counting the route it
wandered in on.

**It is a hole in the outer wall.** The side facing out is opened like any other
side, so the wall that would have stood there is never built. Past it is
nothing, and nothing is drawn black, which is what a door at night looks like
from inside.

**Light comes in.** A pale patch on the floor across the doorway, brighter than
a hint and a different colour, lying where the opening is rather than at the
cell's centre. It is what makes the door visible from down the corridor instead
of only once you are standing in it, and it is there whether or not you have
asked for a hint.

**Reaching the cell is still what finishes it.** The door is somewhere to walk
out of, not a thing to touch. Nothing about the threshold is a trigger.

## Acceptance criteria

- The way out is on the edge of the maze. — `maze::way_out_tests::the_way_out_is_on_the_edge`
- It has an outward side, and that side is open. — `maze::way_out_tests::the_door_is_open`
- No wall stands in the doorway. — `walls::tests::the_doorway_has_no_wall`
- It is still a walk, not a doorstep. — `maze::way_out_tests::it_is_still_a_long_way_off`
- Nothing else opens off the board. — `maze::tests::a_wall_is_solid`
- You can still get to it. — `maze::way_out_tests::there_is_still_a_way_there`
- The door faces out of the maze, not into it. — `maze::way_out_tests::the_door_faces_outwards`
- The threshold lies in the doorway, not in the middle of the cell. — `way_out::tests::the_threshold_lies_in_the_doorway`
- It is wide enough to fill the opening. — `way_out::tests::it_is_as_wide_as_the_door`
- It is brighter than the strongest hint, and a different colour. — `way_out::tests::it_does_not_read_as_a_hint`

### Verified by hand

- Coming down a corridor towards it, the patch of light is visible before the
  doorway is, which is the whole point of it.
- Standing in the doorway and looking out, there is black rather than a wall.
- Nobody has to ask whether that was the end.

## Out of scope

Anything outside the door: ground, sky, a view. Walking out of it leading
anywhere. More than one way out.
