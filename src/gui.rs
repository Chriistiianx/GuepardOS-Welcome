use crate::ui::{MessageType, UI};
use gtk::prelude::*;

pub struct Gui {
    window: gtk::Window,
}

impl Gui {
    pub fn new(window: gtk::Window) -> Self {
        Gui { window }
    }
}

impl UI for Gui {
    fn show_message(&self, message_type: MessageType, message: &str, title: String) {
        let dialog_msg_type = match message_type {
            MessageType::Info => gtk::MessageType::Info,
            MessageType::Warning => gtk::MessageType::Warning,
            MessageType::Error => gtk::MessageType::Error,
        };

        let dialog = gtk::MessageDialog::builder()
            .transient_for(&self.window)
            .message_type(dialog_msg_type)
            .text(message)
            .title(title)
            .modal(true)
            .buttons(gtk::ButtonsType::Ok)
            .build();
        dialog.connect_response(|dialog, _| dialog.close());

        dialog.show();
        gtk::glib::MainContext::default().block_on(dialog.run_future());
        dialog.close();
    }
}

pub fn run_command(command: &str, escalate: bool) -> bool {
    let cmd_formated = format!("{command}; read -p 'Press enter to exit'");
    let mut args: Vec<&str> = vec![];
    if escalate {
        args.extend_from_slice(&["-s", "pkexec /usr/share/guepardos-welcome/scripts/rootshell.sh"]);
    }
    args.push(cmd_formated.as_str());

    let exit_status = subprocess::Exec::cmd("/usr/share/guepardos-welcome/scripts/terminal-helper")
        .args(args.as_slice())
        .stdout(subprocess::Redirection::Pipe)
        .join()
        .unwrap();
    exit_status.success()
}
