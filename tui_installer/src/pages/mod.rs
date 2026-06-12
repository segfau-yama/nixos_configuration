use std::collections::HashMap;

use crate::{
    app::{AppSnapshot, Screen},
    component::Component,
    components::{
        Popup,
        form::{FormField, FormFieldRole},
    },
};

mod device_select;
mod done;
mod github_login;
mod hardware_detect;
mod host_select;
mod locale_select;
mod partition_config;
mod summary;
mod user_menu;
mod welcome;

pub trait InstallerPage: Component {
    fn screen(&self) -> Screen;
    fn sync(&mut self, snapshot: &AppSnapshot);
    fn popup(&self) -> Option<Popup> {
        None
    }
}

pub(crate) fn form_field(
    label: impl Into<String>,
    value: impl Into<String>,
    hint: Option<String>,
    role: FormFieldRole,
) -> FormField {
    FormField::new(label, value, hint, role)
}

pub(crate) fn status_field(message: Option<&str>) -> FormField {
    form_field(
        "status",
        message.unwrap_or("ready"),
        None,
        FormFieldRole::ReadOnly,
    )
}

pub fn build_pages() -> HashMap<Screen, Box<dyn InstallerPage>> {
    let mut pages = HashMap::new();

    for page in vec![
        welcome::page(),
        github_login::page(),
        device_select::page(),
        partition_config::config_page(),
        partition_config::confirm_page(),
        host_select::host_page(),
        host_select::hostname_page(),
        hardware_detect::detect_page(),
        hardware_detect::gpu_page(),
        hardware_detect::cpu_page(),
        hardware_detect::boot_page(),
        locale_select::keyboard_page(),
        locale_select::locale_page(),
        locale_select::timezone_page(),
        locale_select::ssh_page(),
        user_menu::menu_page(),
        user_menu::preset_password_page(),
        user_menu::custom_basic_page(),
        user_menu::custom_type_page(),
        user_menu::custom_programs_page(),
        user_menu::custom_password_page(),
        user_menu::result_page(),
        summary::page(),
        done::page(),
    ] {
        pages.insert(page.screen(), page);
    }

    pages
}
