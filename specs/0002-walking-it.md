# 0002 Walking it

**Status:** draft
**Date:** 2026-09-29

## Goal

The maze from spec 0001 as somewhere to stand: walls you can walk into, a view
from inside, and the two lamps as actual light.

## Behavior

**A cell is a room's worth of floor.** Every wall spec 0001 left standing
becomes a slab between two cells. The maze is built once into one mesh, because
none of it moves.

**You are a sphere that slides.** blitzkit's `move_and_slide` from spec 0014,
against the walls as boxes. A corner you walk into turns you along it rather
than stopping you, which is what stops a dark maze being infuriating.

**The mouse looks and the arrows turn.** They are not the same thing and binding
the arrows to strafe made the game unplayable with one hand: you could not get
round a corner without reaching for the mouse. Left and right turn you, up and
down walk you, A and D strafe, and the mouse looks. The cursor is locked, per
spec 0013.

Turning by key is a rate rather than a distance, because a key is down or it is
not, so how far you turn is how long you hold it.

**A carried candle is stopped by walls like anything else.** Held straight out
it reaches far enough to stand inside a wall you are facing closely, which puts
the light on the far side of it. It is swept from the eye instead and stops at
the first wall, so in a corner you hold it against your chest.

Each candle is swept on its own. Sweeping the hand and then stepping the candles
out to either side of it does the test before the last move, so a candle clear
at the hand still ends up through a wall.

The sweep reports how far it got in world units, not what fraction of the reach
that was. The reach is longer than one unit, so reading it as a fraction places
a candle the whole way out from a wall it just hit, which is the only way one
ever got through.

**A candle is a solid you can see someone holding.** An open tube is
see-through and reads as a shell rather than wax, and a light with nothing
around it reads as a floating orb. A hand closes round the wax of the one you
carry.

**Space does both.** Standing where a candle is, it takes that one up;
otherwise it puts one down. Two keys for putting down and taking up meant
remembering which, for an action that is always one or the other depending on
where you are standing.

**The two lamps are `PointLight`s that cast.** `casts: true`, which spec 0022
turns into six shadow passes each. A lamp standing in the maze lights the
corridor it is in and throws every wall corner across the floor. A lamp in hand
lights where you are.

**Fixed lights that do not cast.** A handful, at most eight in total per spec
0020 counting the two that cast, so a corridor with no lamp in it is dim rather
than black. They are the reason the maze is navigable at all without spending a
lamp, and the reason spending one is worth it.

**The range is what makes light a resource.** A lamp reaches a few cells, not
the whole maze. Nothing about this is enforced by the engine; it is the number
that decides whether the game is a decision or a walk.

## Acceptance criteria

- A wall standing in the maze becomes a collider. — `walls::tests::every_wall_is_solid`
- An open side has no collider. — `walls::tests::an_open_side_is_walkable`
- The colliders sit where the maze says the walls are. — `walls::tests::the_walls_are_where_the_maze_put_them`
- A cell's floor is the size a cell is. — `walls::tests::a_cell_is_a_cell_wide`
- Walking into a wall does not pass through it. — `player::tests::a_wall_stops_you`
- Walking into a corner slides along it. — `player::tests::a_corner_turns_you`
- Walking is relative to where you look. — `player::tests::forward_is_where_you_are_looking`
- Turning changes where forward is, and does not move you. — `player::tests::turning_changes_where_forward_is`
- A wall stops a carried candle reaching through it. — `lights::tests::a_wall_stops_the_candle_reaching_through_it`
- With nothing in the way it is held at arm's length. — `lights::tests::nothing_in_the_way_holds_it_at_arms_length`
- Each candle is swept, not just the hand. — `lights::tests::every_candle_is_swept_not_just_the_hand`
- How far the sweep got is not what fraction of the reach that was. — `lights::tests::how_far_it_got_is_not_how_much_of_the_reach_that_was`
- Nowhere in the maze, facing any way, does a candle end up inside a wall. — `lights::tests::no_candle_ends_up_inside_a_wall_anywhere_in_the_maze`
- The wax is a closed solid rather than a tube. — `candle::tests::the_wax_is_closed`
- The flame stands clear of the wax along its whole body. — `candle::tests::the_flame_sits_above_the_wax`
- A hand closes round the wax. — `candle::tests::the_hand_is_wide_enough_to_hold_the_wax`
- Space puts one down, and takes it back. — `lantern_game::tests::space_puts_one_down_and_takes_it_back`
- Space takes up before it puts down. — `lantern_game::tests::space_takes_up_before_it_puts_down`
- A lamp put down lights the cell it stands in. — `lights::tests::a_standing_lamp_lights_its_cell`
- A carried lamp lights where you are. — `lights::tests::a_carried_lamp_follows_you`
- Both lamps cast, and nothing else does. — `lights::tests::only_the_two_cast`
- The lights handed to the engine never exceed eight. — `lights::tests::there_are_never_more_than_eight`

The last one is the one that bites silently: spec 0020 takes eight and the rest
are dropped, so a ninth light is not an error, it is a lamp that stops working.

### Verified by hand

- A corridor with no lamp is dim enough to want lighting and light enough to
  walk.
- A lamp set down throws the corridor's corners onto the walls and floor. This
  is spec 0022 doing the thing only the cubes example has asked of it.
- Walking away from a lamp, the light falls off and the maze closes back in.
- The two lamps in two places at once look different from both in hand, which
  is the whole mechanic being visible.

## Out of scope

Anything on the walls but a texture: no doors, no levers, no things to pick up.
A ceiling. Anything chasing you.
