use std::ops::Deref;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, CssProvider, MessageType, Orientation, ScrolledWindow, WindowPosition};
use gtk::gdk::{EventMask, Screen};
use crate::gui::common::{add_app_to_listbox, show_dialog};
use crate::gui::config::FINDEX_CONFIG;
use crate::gui::css::load_css;
use crate::gui::dbus::get_all;
use crate::gui::query::init_query;
use crate::gui::search_result::init_search_result;

mod dbus;
mod config;
mod css;
mod query;
mod common;
mod search_result;

pub struct FindexGUI {
    app: Application,
}

impl FindexGUI {
    pub fn init() -> Self {
        // deref to make sure it's evaluated
        let _ = FINDEX_CONFIG.deref();

        let app = Application::builder()
            .application_id("org.findex.gui")
            .build();

        app.connect_activate(Self::window);

        Self {
            app,
        }
    }

    fn window(app: &Application) {
        let mut window = ApplicationWindow::builder()
            .application(app)
            .window_position(WindowPosition::CenterAlways)
            .title("Findex")
            .resizable(false)
            .default_width(FINDEX_CONFIG.default_window_width)
            .decorated(FINDEX_CONFIG.decorate_window);

        if FINDEX_CONFIG.close_window_on_losing_focus {
            window = window
                .events(EventMask::FOCUS_CHANGE_MASK)
                .skip_pager_hint(true)
                .skip_taskbar_hint(true);
        }

        let window = window.build();
        window.style_context().add_class("findex-window");

        if FINDEX_CONFIG.close_window_on_losing_focus {
            window.connect_focus_out_event(|win, _event| {
                win.close();
                Inhibit(true)
            });
        }
        window.connect_key_press_event(|win, event| {
            let key_name = match event.keyval().name() {
                Some(name) => name,
                None => return Inhibit(false)
            };

            if key_name == "Escape" {
                win.close();
                return Inhibit(true);
            }

            Inhibit(false)
        });

        let screen = Screen::default().unwrap();
        let visual = screen.rgba_visual();

        if screen.is_composited() {
            if let Some(visual) = visual {
                window.set_visual(Some(&visual));
            }
        }

        match load_css() {
            Ok(provider) => {
                gtk::StyleContext::add_provider_for_screen(
                    &window.screen().unwrap(),
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
            Err(e) => {
                show_dialog(&window, &e.to_string(), MessageType::Warning, "Warning");

                // try to load css from /opt/findex/style.css
                let file = "/opt/findex/style.css";
                let file_path = std::path::Path::new(file);

                if file_path.exists() {
                    let css_provider = CssProvider::default().unwrap();
                    if let Err(e) = css_provider.load_from_path(file) {
                        show_dialog(
                            &window,
                            &(String::from("Failed to load fallback stylesheet: ") + &e.to_string()),
                            MessageType::Error,
                            "Error",
                        );
                    } else {
                        gtk::StyleContext::add_provider_for_screen(
                            &window.screen().unwrap(),
                            &css_provider,
                            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                        );
                    }
                }
            }
        }

        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .parent(&window)
            .build();
        container.style_context().add_class("findex-container");
        let apps = match get_all() {
            Ok(a) => a,
            Err(e) => {
                show_dialog(
                    &window,
                    &(String::from("Failed to get all apps list: ") + &e.to_string()),
                    MessageType::Error,
                    "Error",
                );
                app.quit();
                return;
            }
        };

        let search_box = init_query();
        let list_box = init_search_result();
        let scw = ScrolledWindow::builder()
            .min_content_height(FINDEX_CONFIG.min_content_height)
            .max_content_height(FINDEX_CONFIG.max_content_height)
            .propagate_natural_height(true)
            .build();
        scw.add(&list_box);
        scw.style_context().add_class("findex-results-scroll");

        container.add(&search_box);
        container.add(&scw);

        window.show_all();

        for app in &apps {
            add_app_to_listbox(&list_box, app);
        }
        if !apps.is_empty() {
            let first_row = list_box.row_at_index(0).unwrap();
            list_box.select_row(Some(&first_row));
        }
    }

    pub fn run(&self) {
        self.app.run();
    }
}