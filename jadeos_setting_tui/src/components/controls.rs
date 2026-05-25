use ratatui::{
    style::Stylize,
    text::{Span, Span as TextSpan},
};

pub fn controls_line() -> Vec<Span<'static>> {
    vec![
        "q".red().bold(),
        TextSpan::raw(" quit   "),
        "Enter".green().bold(),
        TextSpan::raw(" edit/confirm   "),
        "Right".green().bold(),
        TextSpan::raw(" next   "),
        "Esc".yellow().bold(),
        TextSpan::raw(" stop edit/back   "),
        "Left".yellow().bold(),
        TextSpan::raw(" back   "),
        "Tab/Up/Down".cyan().bold(),
        TextSpan::raw(" field   "),
        "Type/Backspace".magenta().bold(),
        TextSpan::raw(" auto edit (text field)   "),
        "Space".blue().bold(),
        TextSpan::raw(" toggle/select"),
    ]
}
