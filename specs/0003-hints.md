# 0003 Hints

**Status:** draft
**Date:** 2026-09-29

## Goal

The maze is sixteen cells square, the way out is around a hundred steps off,
and since spec 0002 made it dark there is nothing to steer by between the
braziers. A way to ask for help, and to decide how much.

## Behavior

**One key, four settings, each more obvious than the last.**

- **Off.** Nothing. What you have now.
- **A whisper.** The next cell on the way out is marked, faintly.
- **A trail.** The next five are marked, brighter.
- **The whole way.** Every cell between you and the way out is marked.

One mechanism at four strengths rather than four different tells. Turning it
up is a decision about how much you want to be told, not about what kind of
help you get, and the step from one setting to the next is legible because it
is the same thing, more of it.

**The marks are drawn, not described.** A flat lozenge on the floor at a cell's
centre, coloured as an unclamped multiplier the way the flames are, so it glows
in a corridor with nothing lighting it instead of needing light to be seen.

**A mark is geometry, so a wall hides it.** Even at the whole way, what you can
see is what is in front of you. Turning it all the way up does not hand you a
map; it lights the route as far down it as you can see, which is further round
each corner than the setting below.

**Shortest means shortest.** `distances_from` took cells off the end of its
queue, which walks depth first and records the first way it wandered in on
rather than the shortest. In a perfect maze those are the same thing; spec 0001
braids half the dead ends open, and they are not. On seed 0 it read 148 steps
where the maze allows 84, which the readout had been reporting all along.

**The route is worked out from where you are.** The distance from every cell to
the way out is computed once, because the maze does not change, and the route
from anywhere is the walk down that. Go the wrong way on purpose and the marks
lead out from where you end up, rather than back to where they were drawn.

**The way out gets no mark.** Spec 0004 lights the threshold there, and two
discs on one cell, one white and one blue and both most of a cell across, is one
too many. The route still runs to it; only the mark on it is dropped.

**Standing on the way out, there is nothing left to show.**

## Acceptance criteria

- The route from the start arrives at the way out. — `hints::tests::the_route_arrives`
- Every step of it goes through an opening. — `hints::tests::every_step_goes_through_an_opening`
- It is as short as the maze allows, not merely a way. — `hints::tests::it_is_as_short_as_the_maze_allows`
- From the way out there is nothing to show. — `hints::tests::at_the_way_out_there_is_nothing_left`
- Off shows nothing. — `hints::tests::off_shows_nothing`
- Each setting shows at least as much as the one below. — `hints::tests::each_setting_shows_more_than_the_last`
- The whole way is the whole way. — `hints::tests::the_whole_way_is_the_whole_way`
- The way out is lit, not marked as well. — `hints::tests::the_way_out_is_not_marked_twice`
- And nothing else is dropped. — `hints::tests::a_mark_is_dropped_only_at_the_way_out`
- A whisper is one cell and a trail is five. — `hints::tests::a_whisper_is_one_and_a_trail_is_five`
- It leads out from wherever you are, not from where you began. — `hints::tests::it_leads_out_from_wherever_you_are`
- The key cycles, and comes back to off. — `lantern_game::tests::the_hint_key_cycles_back_to_off`
- Nothing is marked once you are out. — `lantern_game::tests::no_marks_once_you_are_out`
- A step through a door changes the distance by one at most. — `maze::distance_tests::a_step_changes_the_distance_by_one_at_most`
- It is the same distance back. — `maze::distance_tests::it_is_the_same_distance_back`

### Verified by hand

- A whisper is enough to get round one corner and no more, which is the point
  of it being the lowest setting that does anything.
- At the whole way, a long straight corridor shows a line of marks running off
  into the dark, and a corner still hides what is past it.
- The marks read as guidance rather than as something burning, so they are not
  mistaken for a candle somebody left.

## Out of scope

Marking where you have been. Marking the braziers, the candles you put down, or
anything but the way out. A map, of any kind.
