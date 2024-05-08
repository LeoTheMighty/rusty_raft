use colored::Color;
use rand::{thread_rng, seq::SliceRandom};
use std::ops::Deref;

pub struct RandomColor(pub Color);

pub fn pick_random_color() -> Color {
    // List of available colors
    let colors = [
        Color::Green,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        Color::BrightRed,
        Color::BrightGreen,
        Color::BrightYellow,
        Color::BrightBlue,
        Color::BrightMagenta,
        Color::BrightCyan,
        Color::BrightWhite,
    ];

    // Create a random number generator
    let mut rng = thread_rng();

    // Pick a random color
    *colors.choose(&mut rng).unwrap()
}

impl Default for RandomColor {
    fn default() -> Self {
        RandomColor(pick_random_color())
    }
}

impl Deref for RandomColor {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
