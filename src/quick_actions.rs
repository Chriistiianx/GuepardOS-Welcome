use subprocess::Exec;

#[derive(Clone, Debug)]
pub struct QuickAction {
    pub id: &'static str,
    pub label: &'static str,
    pub commands: &'static [&'static [&'static str]],
}

pub const QUICK_ACTIONS: &[QuickAction] = &[
    QuickAction {
        id: "settings",
        label: "Abrir Ajustes",
        commands: &[&["systemsettings"], &["systemsettings6"], &["systemsettings5"]],
    },
    QuickAction {
        id: "updates",
        label: "Buscar actualizaciones",
        commands: &[&["plasma-discover", "--mode", "Update"], &["discover", "--mode", "Update"]],
    },
    QuickAction {
        id: "packages",
        label: "Abrir gestor de paquetes",
        commands: &[&["pamac-manager"], &["octopi"], &["plasma-discover"], &["discover"]],
    },
    QuickAction {
        id: "system-info",
        label: "Información del sistema",
        commands: &[&["kinfocenter"], &["systemsettings", "kcm_about-distro"]],
    },
    QuickAction { id: "display", label: "Configurar pantallas", commands: &[&["systemsettings", "kcm_kscreen"], &["kscreen-doctor"]] },
    QuickAction { id: "network", label: "Configurar red", commands: &[&["systemsettings", "kcm_networkmanagement"], &["nm-connection-editor"]] },
];

pub fn run_action(action: &QuickAction) -> Result<(), String> {
    for command in action.commands {
        if which::which(command[0]).is_ok() {
            return Exec::cmd(command[0])
                .args(&command[1..])
                .detached()
                .join()
                .map(|_| ())
                .map_err(|e| e.to_string());
        }
    }
    Err(format!("No se encontró una aplicación compatible para {}", action.label))
}

pub fn open_documentation(paths: &[String]) -> Result<(), String> {
    let Some(path) = paths.iter().find(|path| std::path::Path::new(path.as_str()).exists()) else {
        return Err("No se encontró documentación local de GuepardOS.".into());
    };
    for opener in ["dolphin", "kioclient5", "xdg-open"] {
        if which::which(opener).is_ok() {
            return Exec::cmd(opener)
                .arg(path)
                .detached()
                .join()
                .map(|_| ())
                .map_err(|e| e.to_string());
        }
    }
    Err("No se encontró un gestor de archivos compatible.".into())
}
