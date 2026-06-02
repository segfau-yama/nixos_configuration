use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::terminal::Frame;

#[derive(Debug, Clone)]
pub struct FormSection {
    pub title: String,
    pub fields: Vec<FormField>,
    pub active_field: Option<usize>,
    pub text_editing: bool,
}

impl FormSection {
    pub fn new(
        title: impl Into<String>,
        fields: Vec<FormField>,
        active_field: Option<usize>,
        text_editing: bool,
    ) -> Self {
        Self {
            title: title.into(),
            fields,
            active_field,
            text_editing,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub label: String,
    pub value: String,
    pub hint: Option<String>,
    pub role: FormFieldRole,
}

impl FormField {
    pub fn new(
        label: impl Into<String>,
        value: impl Into<String>,
        hint: Option<impl Into<String>>,
        role: FormFieldRole,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            hint: hint.map(Into::into),
            role,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFieldRole {
    Text,
    Choice,
    Toggle,
    Log,
    ReadOnly,
}

pub fn render_form_section(frame: &mut Frame, area: Rect, section: &FormSection) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled(":: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} form", section.title),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    frame.render_widget(title, layout[0]);

    if section.fields.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "- no fields",
            Style::default().fg(Color::DarkGray),
        )));
        frame.render_widget(empty, layout[1]);
        return;
    }

    let field_heights = section.fields.iter().map(field_height).collect::<Vec<_>>();
    let (start, end) = visible_field_range(
        &field_heights,
        layout[1].height,
        section.active_field.unwrap_or(0),
    );
    let constraints = field_heights[start..end]
        .iter()
        .copied()
        .map(Constraint::Length)
        .collect::<Vec<_>>();

    let field_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(layout[1]);

    for (area_index, index) in (start..end).enumerate() {
        let field = &section.fields[index];
        render_field(
            frame,
            field_areas[area_index],
            field,
            section.active_field == Some(index),
            section.text_editing,
        );
    }
}

fn visible_field_range(heights: &[u16], available_height: u16, active: usize) -> (usize, usize) {
    if heights.is_empty() {
        return (0, 0);
    }

    if heights.iter().sum::<u16>() <= available_height {
        return (0, heights.len());
    }

    let active = active.min(heights.len().saturating_sub(1));
    let mut start = active;
    let mut used = heights[active];
    while start > 0 && used.saturating_add(heights[start - 1]) <= available_height {
        start -= 1;
        used = used.saturating_add(heights[start]);
    }

    let mut end = active + 1;
    while end < heights.len() && used.saturating_add(heights[end]) <= available_height {
        used = used.saturating_add(heights[end]);
        end += 1;
    }

    (start, end)
}

fn render_field(
    frame: &mut Frame,
    area: Rect,
    field: &FormField,
    is_active: bool,
    text_editing: bool,
) {
    use ratatui::widgets::{Block, Borders};

    let input_height = if field.role == FormFieldRole::Log {
        area.height
            .saturating_sub(if field.hint.is_some() { 1 } else { 0 })
    } else {
        3
    };
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(input_height),
            Constraint::Length(if field.hint.is_some() { 1 } else { 0 }),
        ])
        .split(area);

    let title = field_title(field, is_active, text_editing);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style(field.role, is_active));
    let input_area = block.inner(layout[0]);

    let value = display_value(&field.value);
    let cursor_offset = field.value.chars().count() as u16;
    let paragraph = if field.role == FormFieldRole::Log {
        Paragraph::new(value.clone())
            .style(value_style(&value, field.role, is_active))
            .wrap(Wrap { trim: false })
            .block(block)
    } else {
        Paragraph::new(Line::from(Span::styled(
            value.clone(),
            value_style(&value, field.role, is_active),
        )))
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: false })
        .block(block)
    };
    frame.render_widget(paragraph, layout[0]);

    if is_active && field.role == FormFieldRole::Text && text_editing {
        let cursor_x = input_area
            .x
            .saturating_add(cursor_offset.min(input_area.width.saturating_sub(1)));
        frame.set_cursor_position((cursor_x, input_area.y));
    }

    if let Some(hint) = &field.hint {
        let hint = Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("note ", Style::default().fg(Color::DarkGray)),
            Span::styled("- ", Style::default().fg(Color::DarkGray)),
            Span::styled(hint.clone(), Style::default().fg(Color::DarkGray)),
        ]));
        frame.render_widget(hint, layout[1]);
    }
}

fn field_title(field: &FormField, is_active: bool, text_editing: bool) -> Line<'static> {
    let role = match field.role {
        FormFieldRole::Text => "input",
        FormFieldRole::Choice => "choice",
        FormFieldRole::Toggle => "toggle",
        FormFieldRole::Log => "log",
        FormFieldRole::ReadOnly => "info",
    };

    let marker = if is_active { ">> " } else { "" };
    let editing = if is_active && field.role == FormFieldRole::Text && text_editing {
        " <editing>"
    } else if is_active {
        " <selected>"
    } else {
        ""
    };

    Line::from(vec![
        Span::styled(marker, Style::default().fg(Color::Yellow)),
        Span::styled(
            field.label.clone(),
            title_style(is_active).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" [{role}]{editing} "),
            Style::default().fg(if is_active {
                Color::Yellow
            } else {
                Color::DarkGray
            }),
        ),
    ])
}

fn display_value(value: &str) -> String {
    if value.trim().is_empty() {
        "<empty>".to_string()
    } else {
        value.to_string()
    }
}

fn border_style(role: FormFieldRole, is_active: bool) -> Style {
    if is_active {
        return Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
    }

    let color = match role {
        FormFieldRole::Text => Color::Blue,
        FormFieldRole::Choice => Color::Cyan,
        FormFieldRole::Toggle => Color::Magenta,
        FormFieldRole::Log => Color::Green,
        FormFieldRole::ReadOnly => Color::DarkGray,
    };

    Style::default().fg(color)
}

fn title_style(is_active: bool) -> Style {
    if is_active {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    }
}

fn value_style(value: &str, role: FormFieldRole, is_active: bool) -> Style {
    if value == "<empty>" {
        let mut style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC);
        if is_active {
            style = style.add_modifier(Modifier::UNDERLINED | Modifier::REVERSED);
        }
        return style;
    }

    let base = match (role, value) {
        (FormFieldRole::Toggle, "true") => Style::default().fg(Color::Green),
        (FormFieldRole::Toggle, "false") => Style::default().fg(Color::Red),
        (FormFieldRole::Choice, "custom") => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        (FormFieldRole::Log, _) => Style::default().fg(Color::Gray),
        (FormFieldRole::ReadOnly, _) => Style::default().fg(Color::Gray),
        _ => Style::default().fg(Color::Cyan),
    };

    if is_active {
        base.add_modifier(Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED)
    } else {
        base
    }
}

fn field_height(field: &FormField) -> u16 {
    if field.role == FormFieldRole::Log {
        let line_count = field.value.lines().count().max(1) as u16;
        let hint_height = if field.hint.is_some() { 1 } else { 0 };
        return line_count.saturating_add(2).clamp(6, 18) + hint_height;
    }

    if field.hint.is_some() { 4 } else { 3 }
}
