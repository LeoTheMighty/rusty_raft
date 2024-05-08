use colored::Color;
use rand::{thread_rng, seq::SliceRandom};

pub fn pick_random_color() -> Color {
    // List of available colors
    let colors = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        Color::BrightBlack,
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