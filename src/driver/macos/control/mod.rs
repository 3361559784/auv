mod app;
mod ax;
pub(crate) mod common;
mod pointer;
mod screen;
mod text;
mod window;

pub(crate) use self::app::activate_app;
pub(crate) use self::ax::{focus_text_input, press_button};
pub(crate) use self::pointer::{click_point, scroll_point};
pub(crate) use self::screen::{click_screen_row, click_screen_text};
pub(crate) use self::text::{paste_text_preserve_clipboard, press_key, type_text};
pub(crate) use self::window::click_window_point;
