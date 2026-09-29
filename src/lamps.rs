//! The two lamps: where they are, and putting them down and taking them up.
//! See `specs/0001-the-two-lamps.md`.
//!
//! Two because blitzkit spec 0022 casts from two and refuses a third. A lamp is
//! either in hand or standing in a cell, never both.

/// blitzkit::lighting::MAX_SHADOWING_POINT_LIGHTS. Named here so the reason
/// survives if the engine's number ever moves.
pub const LAMPS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lamp {
    Carried,
    Standing(usize),
}

#[derive(Debug, Clone)]
pub struct Lamps {
    lamps: [Lamp; LAMPS],
}

impl Lamps {
    /// Both in hand, at the start.
    pub fn new() -> Self {
        Self {
            lamps: [Lamp::Carried; LAMPS],
        }
    }

    pub fn carried(&self) -> usize {
        self.lamps.iter().filter(|l| **l == Lamp::Carried).count()
    }

    /// Where the lamps that are lighting the maze are standing.
    pub fn standing(&self) -> Vec<usize> {
        self.lamps
            .iter()
            .filter_map(|lamp| match lamp {
                Lamp::Standing(cell) => Some(*cell),
                Lamp::Carried => None,
            })
            .collect()
    }

    /// Puts one down where you are. Nothing happens with none in hand.
    pub fn put_down(&mut self, cell: usize) -> bool {
        for lamp in self.lamps.iter_mut() {
            if *lamp == Lamp::Carried {
                *lamp = Lamp::Standing(cell);
                return true;
            }
        }
        false
    }

    /// Takes up the lamp in this cell. You have to be standing at it.
    pub fn take_up(&mut self, cell: usize) -> bool {
        for lamp in self.lamps.iter_mut() {
            if *lamp == Lamp::Standing(cell) {
                *lamp = Lamp::Carried;
                return true;
            }
        }
        false
    }
}

impl Default for Lamps {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_only_two() {
        let mut lamps = Lamps::new();

        assert!(lamps.put_down(3));
        assert!(lamps.put_down(9));
        assert!(!lamps.put_down(11), "a third to put down does not exist");
        assert_eq!(lamps.standing().len(), LAMPS);
    }

    #[test]
    fn a_lamp_stays_where_it_is_put() {
        let mut lamps = Lamps::new();
        lamps.put_down(7);

        assert_eq!(lamps.standing(), vec![7]);
        assert_eq!(lamps.carried(), 1, "and the other is still in hand");
    }

    #[test]
    fn a_lamp_is_taken_from_where_it_is() {
        let mut lamps = Lamps::new();
        lamps.put_down(7);

        assert!(!lamps.take_up(8), "not from a cell away");
        assert!(lamps.take_up(7));
        assert_eq!(lamps.carried(), LAMPS);
    }

    #[test]
    fn carrying_both_lights_nothing_behind() {
        let lamps = Lamps::new();

        assert_eq!(lamps.carried(), LAMPS);
        assert!(lamps.standing().is_empty(), "so nothing is lit but here");
    }

    #[test]
    fn two_lamps_can_stand_in_one_place() {
        // allowed, and a waste: the whole game is not doing this
        let mut lamps = Lamps::new();
        lamps.put_down(4);
        lamps.put_down(4);

        assert_eq!(lamps.standing(), vec![4, 4]);
        assert_eq!(lamps.carried(), 0);
    }
}
