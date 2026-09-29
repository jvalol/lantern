# 0005 Getting out

**Status:** draft
**Date:** 2026-09-29

## Goal

Spec 0004 gave the maze a door. Walking through it changed a word in the corner
of the screen and left you standing on nothing in the dark, looking back at the
wall you came out of. The ending did not land as one.

## Behavior

**It ends when you walk through, not when you arrive.** It ended on reaching the
exit cell, which is a stride short of the doorway: by the time you walk out, it
already happened behind you. Crossing the line the wall stood on is an act, and
that is the thing to end on.

**There is a field outside.** Grass, on the same grid as the floor inside and
flush with it, running far enough that its edge is not something you find. Past
the door is somewhere to arrive rather than somewhere to fall off.

Every light is spoken for, six braziers and two candles against spec 0020's
eight, so the field is not lit. It is coloured as an unclamped multiplier the
way the flames are, brightest at the doorstep and falling away with distance,
which is the only thing making it look like moonlight.

It is drawn whether you are out on it or not. Seeing grass through the doorway
from inside is most of what makes the door worth walking to.

**The candles go out.** Out on the grass, the light you have been rationing all
game simply stops, and the braziers behind you keep burning because they were
never yours. This is the mechanic ending rather than the screen announcing that
it has.

**A sound.** A sine rising from 220 Hz to 660 Hz over three quarters of a
second, generated the way marble's thud is rather than shipped as a file, fading
in so it opens rather than strikes. It plays once, at the crossing. Walking
further out does not play it again.

**And a line, in the middle of the screen**, the way tessera ends.

## Acceptance criteria

- It ends on crossing the line, not on reaching the cell. — `lantern_game::tests::it_ends_when_you_walk_through_not_when_you_arrive`
- The candles go out, and the braziers do not. — `lantern_game::tests::the_candles_go_out_when_you_do`
- The line sits in the middle of the window. — `lantern_game::tests::the_ending_is_centred_in_the_window`
- No grass lies over the maze. — `field::tests::the_field_is_outside_the_maze`
- It is flush with the floor inside. — `field::tests::it_is_flush_with_the_floor_inside`
- It is brightest at the door. — `field::tests::it_is_brightest_at_the_door`
- And fades to nothing before its edge. — `field::tests::it_fades_into_the_night`
- The sound stops on its own. — `chime::tests::it_runs_out`
- It makes a sound, and does not clip. — `chime::tests::it_makes_a_sound`
- It opens rather than strikes. — `chime::tests::it_opens_rather_than_strikes`
- It rises. — `chime::tests::it_rises`

The last one counts zero crossings in the first tenth and the last, which is
frequency measured without the test having to know what a chirp is.

### Verified by hand

- Coming down the last corridor, there is grass through the doorway before
  there is a doorway.
- Stepping across, the candles go out and the field is still there, which is
  the point: you do not need them any more.
- Looking back from the field, the braziers are still burning in the dark.

## Out of scope

A sky, a horizon, or anything standing in the field. Walking far enough to find
its edge. Anything after this: the way out is the end.
