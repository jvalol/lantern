# 0001 The two lamps

**Status:** draft
**Date:** 2026-09-29

## Goal

A dark maze and two lamps to light it with. The game is deciding where to leave
them.

## Behavior

**Two lamps.** blitzkit spec 0022 casts shadows from at most two point lights
and refuses a third, because each one is six passes over the scene. That is a
ceiling chosen for cost, not a hardware limit, and it could move.

Two is right here anyway. One lamp is a torch and no decision at all. Three is
enough to light ahead and behind and still hold one spare, which is both easier
and harder to read: you stop choosing, and you lose track of which lamp is
marking what.

**A lamp is put down, not carried.** Setting one down leaves it burning where it
stands. Picking it up takes it back. Both are done where you are, so moving a
lamp means walking it.

**Light is also memory.** A lamp left at a junction says you have been there.
So the two are always wanted in two places at once: ahead, where you cannot see,
and behind, where you will otherwise get lost. That tension is the game.

**The maze is dark otherwise.** No sun. A handful of fixed lights that do not
cast, at most eight per blitzkit spec 0020, so a corridor is dim rather than
black and the two that cast are the only ones that carve shape out of it.

**Half the dead ends are opened out.** Carving gives a maze with one route
between any two places, which is nothing but dead ends and the walk back from
them. Opening a second way out of half of them leaves about thirteen loops in a
sixteen by sixteen grid, and takes the exit from about 167 steps away to
somewhere between 95 and 139.

Loops are also what makes a lamp worth leaving. In a maze with one route,
a lamp behind you only ever says "you came from there". With loops it can say
"you have already been down this one", which is a different and more useful
thing.

**You are looking for the way out**, and the exit is not marked. Finding it
means having lit enough to know where you have not been.

## Acceptance criteria

- Two lamps, and no more. — `lamps::tests::there_are_only_two`
- Putting one down leaves it where you stood. — `lamps::tests::a_lamp_stays_where_it_is_put`
- Picking one up needs you to be at it. — `lamps::tests::a_lamp_is_taken_from_where_it_is`
- Carrying both leaves nothing lit behind. — `lamps::tests::carrying_both_lights_nothing_behind`
- The maze has a way out. — `maze::tests::every_maze_can_be_finished`
- Every cell can be reached. — `maze::tests::every_cell_can_be_reached`
- There are loops, so a wrong turn is not always a dead end. — `maze::tests::there_are_loops`
- Braiding leaves fewer dead ends than carving did. — `maze::tests::braiding_takes_out_about_half_the_dead_ends`
- The exit is not next to the start. — `maze::tests::the_exit_is_not_on_the_doorstep`
- Walls stop you. — `player::tests::a_wall_is_solid`

### Verified by hand

- A corridor with no lamp in it is dim enough to be worth lighting and not so
  dark it is unplayable.
- A lamp set down throws the corridor's shape onto the walls, which is spec
  0022 doing the thing nothing but the cubes example has asked of it.
- Leaving one behind and walking on feels like a cost.

## Out of scope

Anything to solve but the maze: no puzzles, no notes to find, no story. A third
lamp. Anything chasing you.
