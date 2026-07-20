use subprocess::{Exec, Redirection};

#[derive(Clone, Debug)]
pub struct SystemStatus {
    pub internet: String,
    pub updates: String,
    pub gpu: String,
    pub session: String,
    pub flatpak: String,
}

pub fn collect() -> SystemStatus {
    SystemStatus {
        internet: if has_internet() { "Conectado".into() } else { "Sin conexión".into() },
        updates: pending_updates(),
        gpu: detect_gpu(),
        session: std::env::var("XDG_SESSION_TYPE")
            .map(|v| if v.eq_ignore_ascii_case("wayland") { "Wayland" } else { "X11" }.into())
            .unwrap_or_else(|_| "Desconocida".into()),
        flatpak: if command_exists("flatpak") { "Instalado".into() } else { "No instalado".into() },
    }
}

pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

fn has_internet() -> bool {
    Exec::cmd("ping")
        .args(&["-c", "1", "-W", "2", "1.1.1.1"])
        .stdout(Redirection::Null)
        .stderr(Redirection::Null)
        .join()
        .is_ok_and(|status| status.success())
}

fn pending_updates() -> String {
    if command_exists("checkupdates") {
        return match Exec::cmd("checkupdates").stdout(Redirection::Pipe).capture() {
            Ok(out) => {
                let count = out.stdout_str().lines().filter(|line| !line.trim().is_empty()).count();
                if count == 0 { "Sistema al día".into() } else { format!("{count} pendientes") }
            },
            Err(_) => "No disponible".into(),
        };
    }
    "No disponible".into()
}

fn detect_gpu() -> String {
    let lspci = Exec::cmd("lspci").stdout(Redirection::Pipe).capture();
    let Ok(output) = lspci else {
        return "Desconocida".into();
    };
    let text = output.stdout_str().to_lowercase();
    if text.contains("nvidia") {
        "NVIDIA".into()
    } else if text.contains("amd") || text.contains("advanced micro devices") || text.contains("radeon") {
        "AMD".into()
    } else if text.contains("intel") {
        "Intel".into()
    } else {
        "Desconocida".into()
    }
}

pub fn guepardos_version() -> Option<String> {
    for path in ["/etc/guepardos-release", "/etc/os-release", "/etc/lsb-release"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            if path.ends_with("guepardos-release") {
                let value = content.trim();
                if !value.is_empty() {
                    return Some(value.to_owned());
                }
            }
            for line in content.lines() {
                if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                    return Some(value.trim_matches('"').to_owned());
                }
            }
        }
    }
    None
}
