use ratatui::{
    style::Stylize,
    text::{Span, Span as TextSpan},
};

pub fn controls_line() -> Vec<Span<'static>> {
    vec![
        "q".red().bold(),
        TextSpan::raw(" quit   "),
        "Enter/Right".green().bold(),
        TextSpan::raw(" confirm/next   "),
        "Esc/Left".yellow().bold(),
        TextSpan::raw(" back   "),
        "Tab/Up/Down".cyan().bold(),
        TextSpan::raw(" field   "),
        "Type/Backspace".magenta().bold(),
        TextSpan::raw(" edit   "),
        "Space".blue().bold(),
        TextSpan::raw(" toggle/select"),
    ]
}
