use crate::config::{APP_ID, VERSION};
use crate::{RESPREFIX, check_regular_file, first_steps, package_installer, quick_actions, system_status, utils};

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib::GString;
use gtk::prelude::*;
use gtk::{Builder, HeaderBar, Window, glib};
use tracing::error;

#[derive(Clone, Debug)]
pub struct HelloWindow {
    pub builder: gtk::Builder,
    pub window: gtk::Window,
    preferences: serde_json::Value,
}

// SAFETY: GTK UI access is kept on the GTK main thread. The global is initialized
// once during startup and then only used from GTK signal callbacks.
unsafe impl Send for HelloWindow {}
unsafe impl Sync for HelloWindow {}

impl HelloWindow {
    #[expect(clippy::too_many_lines, reason = "GTK window composition")]
    pub fn new(
        application: &gtk::Application,
        preferences: serde_json::Value,
        _best_locale: &str,
    ) -> Self {
        if let Some(icon_theme) = gtk::IconTheme::default() {
            icon_theme.add_resource_path(&format!("{RESPREFIX}/data/img"));
        }

        let provider = gtk::CssProvider::new();
        provider.load_from_resource(&format!("{RESPREFIX}/ui/style.css"));
        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let builder = Builder::new();
        let main_window = gtk::ApplicationWindow::new(application);
        main_window.set_title("GuepardOS Welcome");
        main_window.set_default_size(1100, 760);
        main_window.set_position(gtk::WindowPosition::Center);
        main_window.style_context().add_class("welcome-root");

        let header = HeaderBar::new();
        header.set_title(Some("GuepardOS Welcome"));
        header.set_subtitle(Some("Basado en Arch Linux"));
        header.set_show_close_button(true);
        let about_btn = gtk::Button::from_icon_name(Some("help-about"), gtk::IconSize::Button);
        about_btn.set_tooltip_text(Some("Acerca de GuepardOS Welcome"));
        about_btn.connect_clicked(|_| {
            if let Some(window) = crate::G_HELLO_WINDOW.get() {
                window.show_about_dialog();
            }
        });
        header.pack_end(&about_btn);
        main_window.set_titlebar(Some(&header));

        let scrolled = gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        let root = gtk::Box::new(gtk::Orientation::Vertical, 24);
        root.set_margin_top(28);
        root.set_margin_bottom(28);
        root.set_margin_start(28);
        root.set_margin_end(28);
        root.style_context().add_class("welcome-root");
        scrolled.add(&root);
        main_window.add(&scrolled);

        root.pack_start(&create_hero(&preferences), false, false, 0);
        root.pack_start(&create_status_card(), false, false, 0);
        root.pack_start(&create_profiles(&main_window.clone().upcast::<Window>()), false, false, 0);
        root.pack_start(&create_quick_actions(&preferences, &main_window.clone().upcast::<Window>()), false, false, 0);
        root.pack_start(&create_first_steps(&preferences), false, false, 0);
        root.pack_start(&create_footer(&preferences), false, false, 0);

        let window = main_window.upcast::<Window>();
        window.show_all();

        HelloWindow { builder, window, preferences }
    }

    pub fn show_about_dialog(&self) {
        let logo_path = format!("/usr/share/icons/hicolor/scalable/apps/{APP_ID}.svg");
        let dialog = gtk::AboutDialog::builder()
            .transient_for(&self.window)
            .modal(true)
            .program_name(GString::from_string_unchecked("GuepardOS Welcome".to_owned()))
            .comments(GString::from_string_unchecked(
                "Aplicación de bienvenida para preparar GuepardOS.".to_owned(),
            ))
            .version(VERSION)
            .authors(vec!["GuepardOS team".to_owned(), "Vladislav Nepogodin".to_owned()])
            .translator_credits("translator-credits")
            .copyright("2021-2026 GuepardOS team")
            .license_type(gtk::License::Gpl30)
            .website("https://github.com/GuepardOS/guepardos-welcome")
            .website_label("GitHub")
            .build();

        if let Ok(logo) = Pixbuf::from_file(logo_path) {
            dialog.set_logo(Some(&logo));
        }

        dialog.run();
        dialog.hide();
    }

    pub fn switch_locale(&self, _use_locale: &str) {}

    pub fn set_autostart(&self, autostart: bool) {
        let autostart_path = utils::fix_path(self.preferences["autostart_path"].as_str().unwrap());
        let desktop_path = self.preferences["desktop_path"].as_str().unwrap().to_owned();
        let config_dir = Path::new(&autostart_path).parent().unwrap();
        if !config_dir.exists() && let Err(e) = fs::create_dir_all(config_dir) {
            error!("Could not create autostart directory: {e}");
            return;
        }
        if autostart && !check_regular_file(&autostart_path) {
            if let Err(e) = std::os::unix::fs::symlink(desktop_path, &autostart_path) {
                error!("Could not enable autostart: {e}");
            }
        } else if !autostart && check_regular_file(&autostart_path)
            && let Err(e) = std::fs::remove_file(&autostart_path)
        {
            error!("Could not disable autostart: {e}");
        }
    }

    pub fn open_uri(&self, uri: &str) {
        if let Err(uri_err) = gtk::show_uri_on_window(Some(&self.window), uri, 0) {
            error!("Failed to open uri: {uri_err}");
        }
    }

    pub fn set_stack_child_visible(&self, _child_name: &str) {}

    pub fn get_preferences(&self, entry: &str) -> &serde_json::Value {
        &self.preferences[entry]
    }
}

fn create_hero(preferences: &serde_json::Value) -> gtk::Box {
    let hero = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    hero.set_valign(gtk::Align::Center);

    let logo = gtk::Image::new();
    let logo_path = format!("{}/org.guepardos.welcome.svg", preferences["logo_path"].as_str().unwrap());
    if Path::new(&logo_path).exists() {
        logo.set_from_file(Some(&logo_path));
    } else {
        logo.set_from_icon_name(Some("computer-symbolic"), gtk::IconSize::Dialog);
        logo.set_tooltip_text(Some("Logo temporal de GuepardOS Welcome"));
    }
    logo.set_pixel_size(88);
    hero.pack_start(&logo, false, false, 0);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let title = label("Bienvenido a GuepardOS", 0.0, Some("hero-title"));
    title.set_markup("<span size=\"xx-large\" weight=\"bold\">Bienvenido a GuepardOS</span>");
    let subtitle = label("Tu sistema, preparado a tu manera", 0.0, Some("hero-subtitle"));
    subtitle.set_markup("<span size=\"large\">Tu sistema, preparado a tu manera</span>");
    let release = system_status::guepardos_version()
        .map(|version| format!("Basado en Arch Linux · Rolling Release · {version}"))
        .unwrap_or_else(|| "Basado en Arch Linux · Rolling Release".to_owned());
    text.pack_start(&title, false, false, 0);
    text.pack_start(&subtitle, false, false, 0);
    text.pack_start(&label(&release, 0.0, Some("muted")), false, false, 0);
    hero.pack_start(&text, true, true, 0);
    hero
}

fn create_status_card() -> gtk::Box {
    let card = card("Estado inicial del sistema");
    let grid = gtk::Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(12);
    card.pack_start(&grid, false, false, 0);

    let labels = Rc::new(vec![
        ("internet", status_pill("Internet", "Comprobando...")),
        ("updates", status_pill("Actualizaciones", "Comprobando...")),
        ("gpu", status_pill("GPU", "Comprobando...")),
        ("session", status_pill("Sesión", "Comprobando...")),
        ("flatpak", status_pill("Flatpak", "Comprobando...")),
    ]);

    for (index, (_, widget)) in labels.iter().enumerate() {
        grid.attach(widget, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }

    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let _ = tx.send_blocking(system_status::collect());
    });
    glib::MainContext::default().spawn_local(async move {
        if let Ok(status) = rx.recv().await {
            for (key, widget) in labels.iter() {
                let value = match *key {
                    "internet" => &status.internet,
                    "updates" => &status.updates,
                    "gpu" => &status.gpu,
                    "session" => &status.session,
                    "flatpak" => &status.flatpak,
                    _ => "",
                };
                widget.set_text(&format!("{}: {value}", widget.widget_name()));
            }
        }
    });

    card
}

fn create_profiles(parent: &Window) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 12);
    section.pack_start(&section_title("Perfiles rápidos"), false, false, 0);
    let grid = gtk::Grid::new();
    grid.set_column_spacing(14);
    grid.set_row_spacing(14);
    section.pack_start(&grid, false, false, 0);

    for (index, profile) in package_installer::PROFILES.iter().enumerate() {
        let item = gtk::Box::new(gtk::Orientation::Vertical, 10);
        item.style_context().add_class("profile-card");
        item.set_margin_top(2);
        item.set_margin_bottom(2);
        item.set_margin_start(2);
        item.set_margin_end(2);
        let title = label(profile.title, 0.0, None);
        title.set_markup(&format!("<b>{}</b>", profile.title));
        item.pack_start(&title, false, false, 0);
        item.pack_start(&label(profile.description, 0.0, Some("muted")), false, false, 0);
        let button = gtk::Button::with_label("Elegir componentes");
        button.style_context().add_class("suggested-action");
        let parent = parent.clone();
        let profile = *profile;
        button.connect_clicked(move |_| show_profile_dialog(&parent, profile));
        item.pack_end(&button, false, false, 0);
        item.set_size_request(260, 150);
        grid.attach(&item, index as i32, 0, 1, 1);
    }
    section
}

fn show_profile_dialog(parent: &Window, profile: package_installer::PackageProfile) {
    let dialog = gtk::Dialog::with_buttons(
        Some(profile.title),
        Some(parent),
        gtk::DialogFlags::MODAL,
        &[("Cancelar", gtk::ResponseType::Cancel), ("Instalar selección", gtk::ResponseType::Accept)],
    );
    dialog.set_default_size(560, 420);
    let content = dialog.content_area();
    content.set_spacing(12);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.pack_start(&label(profile.description, 0.0, Some("muted")), false, false, 0);

    let checks: Rc<RefCell<Vec<(gtk::CheckButton, package_installer::PackageItem)>>> =
        Rc::new(RefCell::new(Vec::new()));
    for item in profile.items {
        let row = gtk::Box::new(gtk::Orientation::Vertical, 4);
        row.style_context().add_class("install-row");
        row.set_margin_top(3);
        row.set_margin_bottom(3);
        row.set_margin_start(3);
        row.set_margin_end(3);
        let installed = package_installer::is_installed(item);
        let suffix = match (installed, item.source) {
            (true, _) => " · instalado",
            (false, package_installer::PackageSource::Unavailable) => " · fuente no disponible todavía",
            _ => "",
        };
        let check = gtk::CheckButton::with_label(&format!("{}{}", item.label, suffix));
        check.set_active(!installed && item.source == package_installer::PackageSource::Pacman);
        check.set_sensitive(!installed && item.source == package_installer::PackageSource::Pacman);
        row.pack_start(&check, false, false, 0);
        row.pack_start(&label(&format!("Paquetes: {}", item.packages.join(", ")), 0.0, Some("muted")), false, false, 0);
        content.pack_start(&row, false, false, 0);
        checks.borrow_mut().push((check, *item));
    }

    dialog.show_all();
    if dialog.run() == gtk::ResponseType::Accept {
        let selected: Vec<_> = checks
            .borrow()
            .iter()
            .filter_map(|(check, item)| check.is_active().then_some(*item))
            .collect();
        let packages = package_installer::missing_packages(&selected);
        confirm_and_install(parent, &packages);
    }
    dialog.destroy();
}

fn confirm_and_install(parent: &Window, packages: &[&str]) {
    let message = if packages.is_empty() {
        "Todo lo seleccionado ya está instalado.".to_owned()
    } else {
        format!("Se instalarán estos paquetes:\n\n{}", packages.join(", "))
    };
    let confirm = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Question,
        gtk::ButtonsType::OkCancel,
        &message,
    );
    let accepted = confirm.run() == gtk::ResponseType::Ok;
    confirm.destroy();
    if !accepted || packages.is_empty() {
        return;
    }

    let progress = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Info,
        gtk::ButtonsType::None,
        "Instalando paquetes. Puedes cancelar desde el diálogo de autenticación si lo necesitas.",
    );
    progress.show_all();
    let parent = parent.clone();
    let packages: Vec<String> = packages.iter().map(|package| (*package).to_owned()).collect();
    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let package_refs: Vec<&str> = packages.iter().map(String::as_str).collect();
        let _ = tx.send_blocking(package_installer::install_packages(&package_refs));
    });
    glib::MainContext::default().spawn_local(async move {
        if let Ok(result) = rx.recv().await {
            progress.destroy();
            let (kind, text) = match result {
                Ok(_) => (gtk::MessageType::Info, "Instalación finalizada.".to_owned()),
                Err(e) => (gtk::MessageType::Error, format!("No se pudo completar la instalación:\n\n{e}")),
            };
            let done = gtk::MessageDialog::new(
                Some(&parent),
                gtk::DialogFlags::MODAL,
                kind,
                gtk::ButtonsType::Ok,
                &text,
            );
            done.run();
            done.destroy();
        }
    });
}

fn create_quick_actions(preferences: &serde_json::Value, parent: &Window) -> gtk::Box {
    let section = card("Acciones rápidas");
    let grid = gtk::Grid::new();
    grid.set_column_spacing(10);
    grid.set_row_spacing(10);
    section.pack_start(&grid, false, false, 0);
    let parent = parent.clone();
    for (index, action) in quick_actions::QUICK_ACTIONS.iter().enumerate() {
        let button = gtk::Button::with_label(action.label);
        let action = action.clone();
        let parent = parent.clone();
        button.connect_clicked(move |_| {
            if let Err(e) = quick_actions::run_action(&action) {
                show_message(&parent, gtk::MessageType::Warning, &e);
            }
        });
        grid.attach(&button, (index % 3) as i32, (index / 3) as i32, 1, 1);
    }

    let docs = gtk::Button::with_label("Documentación local");
    let paths: Vec<String> = preferences["documentation_paths"]
        .as_array()
        .map(|items| items.iter().filter_map(|v| v.as_str().map(ToOwned::to_owned)).collect())
        .unwrap_or_default();
    let parent = parent.clone();
    docs.connect_clicked(move |_| {
        if let Err(e) = quick_actions::open_documentation(&paths) {
            show_message(&parent, gtk::MessageType::Warning, &e);
        }
    });
    grid.attach(&docs, 0, 2, 1, 1);
    section
}

fn create_first_steps(preferences: &serde_json::Value) -> gtk::Box {
    let section = card("Primeros pasos");
    let save_path = preferences["save_path"].as_str().unwrap().to_owned();
    let state = Rc::new(RefCell::new(first_steps::load(&save_path)));
    for (id, title) in first_steps::TASKS {
        let check = gtk::CheckButton::with_label(title);
        check.set_active(state.borrow().completed.iter().any(|item| item == id));
        let save_path = save_path.clone();
        let state = state.clone();
        check.connect_toggled(move |check| {
            first_steps::set_completed(&mut state.borrow_mut(), id, check.is_active());
            first_steps::save(&save_path, &state.borrow());
        });
        section.pack_start(&check, false, false, 0);
    }
    section
}

fn create_footer(preferences: &serde_json::Value) -> gtk::Box {
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    footer.set_halign(gtk::Align::End);
    let label = gtk::Label::new(Some("Ejecutar GuepardOS Welcome al iniciar sesión"));
    label.style_context().add_class("muted");
    let autostart = gtk::Switch::new();
    let autostart_path = utils::fix_path(preferences["autostart_path"].as_str().unwrap());
    autostart.set_active(Path::new(&autostart_path).exists());
    autostart.connect_state_set(|switch, state| {
        if let Some(window) = crate::G_HELLO_WINDOW.get() {
            window.set_autostart(state);
        }
        switch.set_state(state);
        gtk::Inhibit(false)
    });
    footer.pack_start(&label, false, false, 0);
    footer.pack_start(&autostart, false, false, 0);
    footer
}

fn card(title: &str) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 12);
    card.set_margin_top(2);
    card.set_margin_bottom(2);
    card.set_margin_start(2);
    card.set_margin_end(2);
    card.style_context().add_class("status-card");
    card.pack_start(&section_title(title), false, false, 0);
    card
}

fn section_title(title: &str) -> gtk::Label {
    let label = label(title, 0.0, None);
    label.set_markup(&format!("<span size=\"large\" weight=\"bold\">{title}</span>"));
    label
}

fn status_pill(name: &str, value: &str) -> gtk::Label {
    let pill = label(&format!("{name}: {value}"), 0.0, Some("status-pill"));
    pill.set_widget_name(name);
    pill.set_size_request(220, 42);
    pill
}

fn label(text: &str, xalign: f32, class_name: Option<&str>) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(xalign);
    label.set_line_wrap(true);
    if let Some(class_name) = class_name {
        label.style_context().add_class(class_name);
    }
    label
}

fn show_message(parent: &Window, kind: gtk::MessageType, text: &str) {
    let dialog = gtk::MessageDialog::new(
        Some(parent),
        gtk::DialogFlags::MODAL,
        kind,
        gtk::ButtonsType::Ok,
        text,
    );
    dialog.run();
    dialog.destroy();
}
