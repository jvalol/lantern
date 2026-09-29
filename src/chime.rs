//! The sound of getting out. See `specs/0005-getting-out.md`.
//!
//! Generated rather than loaded, the way marble's thud is. A sine rising from
//! 220 Hz to 660 Hz over three quarters of a second, a fifth and an octave up,
//! fading in and out so it opens rather than strikes. Nothing here needs an
//! audio device, so it can be checked without one.

use rodio::source::{chirp, Source};
use rodio::SampleRate;
use std::time::Duration;

const LENGTH: Duration = Duration::from_millis(750);
const START_HZ: f32 = 220.0;
const END_HZ: f32 = 660.0;
const SAMPLE_RATE: u32 = 48_000;
const VOLUME: f32 = 0.5;

pub fn chime() -> impl Source + Send + 'static {
    let rate = SampleRate::new(SAMPLE_RATE).expect("the sample rate is not zero");

    chirp(rate, START_HZ, END_HZ, LENGTH)
        .fade_in(Duration::from_millis(120))
        .fade_out(LENGTH)
        .amplify(VOLUME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_runs_out() {
        // it has to stop on its own: nothing tells it to
        let samples = chime().count();
        let expected = (SAMPLE_RATE as f32 * LENGTH.as_secs_f32()) as usize;

        assert!(
            samples.abs_diff(expected) < SAMPLE_RATE as usize / 10,
            "{} samples where {} were due",
            samples,
            expected
        );
    }

    #[test]
    fn it_makes_a_sound() {
        let loudest = chime().fold(0.0f32, |loudest, sample| loudest.max(sample.abs()));

        assert!(loudest > 0.05, "the loudest it gets is {}", loudest);
        assert!(loudest <= 1.0, "it clips at {}", loudest);
    }

    #[test]
    fn it_opens_rather_than_strikes() {
        // the fade in means it does not start at full volume
        let first: Vec<f32> = chime().take(SAMPLE_RATE as usize / 100).collect();
        let start = first.iter().fold(0.0f32, |l, s| l.max(s.abs()));
        let whole = chime().fold(0.0f32, |l, s| l.max(s.abs()));

        assert!(start < whole * 0.5, "it starts at {} of {}", start, whole);
    }

    #[test]
    fn it_rises() {
        // count zero crossings in the first tenth and the last, which is
        // frequency without needing to know what a chirp is
        let all: Vec<f32> = chime().collect();
        let tenth = all.len() / 10;
        let crossings = |window: &[f32]| {
            window
                .windows(2)
                .filter(|pair| (pair[0] < 0.0) != (pair[1] < 0.0))
                .count()
        };

        let early = crossings(&all[..tenth]);
        let late = crossings(&all[all.len() - tenth..]);

        assert!(late > early, "it goes from {} crossings to {}", early, late);
    }
}
