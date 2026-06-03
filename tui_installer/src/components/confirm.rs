use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::components::form::{FormField, FormFieldRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmChoiceAction {
    Noop,
    Submit(bool),
    Cancel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConfirmChoice {
    selected_yes: bool,
}

impl ConfirmChoice {
    pub fn handle_key(&mut self, key: KeyEvent) -> ConfirmChoiceAction {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => ConfirmChoiceAction::Submit(true),
            KeyCode::Char('n') | KeyCode::Char('N') => ConfirmChoiceAction::Submit(false),
            KeyCode::Char(' ') | KeyCode::Up | KeyCode::Down | KeyCode::Tab => {
                self.selected_yes = !self.selected_yes;
                ConfirmChoiceAction::Noop
            }
            KeyCode::Left | KeyCode::Esc => ConfirmChoiceAction::Cancel,
            KeyCode::Right | KeyCode::Enter => ConfirmChoiceAction::Submit(self.selected_yes),
            _ => ConfirmChoiceAction::Noop,
        }
    }

    pub fn field(&self, label: impl Into<String>, hint: impl Into<String>) -> FormField {
        FormField::new(
            label,
            if self.selected_yes {
                "[ yes ] / no"
            } else {
                "yes / [ no ]"
            },
            Some(hint),
            FormFieldRole::Choice,
        )
    }
}
