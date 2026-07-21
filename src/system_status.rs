use subprocess::{Exec, Redirection};

#[derive(Clone, Debug)]
pub struct SystemStatus {
    pub internet: String,
    pub updates: String,
    pub gpu: String,
    pub session: String,
    pub flatpak: String,
    pub firewall: String,
    pub snapshots: String,
    pub bluetooth: String,
    pub disk: String,
    pub reboot_required: bool,
}

pub fn collect() -> SystemStatus {
    SystemStatus {
        internet: if has_internet() { "Conectado".into() } else { "Sin conexión".into() },
        updates: pending_updates(),
        gpu: detect_gpu(),
        session: std::env::var("XDG_SESSION_TYPE")
            .map(|v| if v.eq_ignore_ascii_case("wayland") { "Wayland" } else { "X11" }.into())
            .unwrap_or_else(|_| "No disponible".into()),
        flatpak: if command_exists("flatpak") { "Instalado".into() } else { "No está instalado".into() },
        firewall: service_status(&["firewalld", "ufw"]),
        snapshots: snapshot_status(),
        bluetooth: if command_exists("bluetoothctl") || std::path::Path::new("/sys/class/bluetooth").exists() {
            "Disponible".into()
        } else { "No disponible".into() },
        disk: disk_free(),
        reboot_required: std::path::Path::new("/run/reboot-required").exists(),
    }
}

pub fn command_exists(command: &str) -> bool { which::which(command).is_ok() }

pub fn has_internet() -> bool {
    Exec::cmd("ping").args(&["-c", "1", "-W", "2", "1.1.1.1"])
        .stdout(Redirection::Null).stderr(Redirection::Null).join().is_ok_and(|s| s.success())
}

fn pending_updates() -> String {
    if !command_exists("checkupdates") { return "No disponible".into(); }
    match Exec::cmd("checkupdates").stdout(Redirection::Pipe).stderr(Redirection::Null).capture() {
        Ok(out) => {
            let count = out.stdout_str().lines().filter(|line| !line.trim().is_empty()).count();
            if count == 0 { "Sistema al día".into() } else { format!("{count} disponibles") }
        }
        Err(_) => "No disponible".into(),
    }
}

pub fn detect_gpu() -> String {
    let output = Exec::cmd("lspci").stdout(Redirection::Pipe).stderr(Redirection::Null).capture();
    let Ok(output) = output else { return "GPU no detectada".into() };
    let stdout = output.stdout_str();
    let line = stdout.lines().find(|line| {
        let lower = line.to_lowercase();
        lower.contains("vga compatible controller") || lower.contains("3d controller") || lower.contains("display controller")
    });
    line.map(|line| line.splitn(3, ':').nth(2).unwrap_or(line).trim().to_owned())
        .unwrap_or_else(|| "GPU no detectada".into())
}

fn service_status(names: &[&str]) -> String {
    if !command_exists("systemctl") { return "Desconocido".into(); }
    for name in names {
        if Exec::cmd("systemctl").args(&["is-active", "--quiet", name]).join().is_ok_and(|s| s.success()) {
            return "Activo".into();
        }
    }
    "No detectado".into()
}

fn snapshot_status() -> String {
    if command_exists("timeshift") || command_exists("snapper") || std::path::Path::new("/.snapshots").exists() {
        "Configurados".into()
    } else { "No configurados".into() }
}

fn disk_free() -> String {
    match Exec::cmd("df").args(&["-h", "/"]).stdout(Redirection::Pipe).capture() {
        Ok(out) => out.stdout_str().lines().nth(1).and_then(|line| line.split_whitespace().nth(3))
            .map(|free| format!("{free} libres")).unwrap_or_else(|| "No disponible".into()),
        Err(_) => "No disponible".into(),
    }
}

pub fn guepardos_version() -> Option<String> {
    for path in ["/etc/guepardos-release", "/etc/os-release", "/etc/lsb-release"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            if path.ends_with("guepardos-release") && !content.trim().is_empty() { return Some(content.trim().to_owned()); }
            if let Some(value) = content.lines().find_map(|line| line.strip_prefix("VERSION_ID=")) {
                return Some(value.trim_matches('"').to_owned());
            }
        }
    }
    None
}
