use subprocess::{Exec, Redirection};
use std::sync::atomic::{AtomicBool, Ordering};

static INSTALL_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageSource {
    Pacman,
    Unavailable,
}

#[derive(Clone, Copy, Debug)]
pub struct PackageItem {
    pub id: &'static str,
    pub label: &'static str,
    pub packages: &'static [&'static str],
    pub source: PackageSource,
}

#[derive(Clone, Copy, Debug)]
pub struct PackageProfile {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub items: &'static [PackageItem],
}

pub const ESSENTIALS: &[PackageItem] = &[
    PackageItem { id: "firefox", label: "Firefox", packages: &["firefox"], source: PackageSource::Pacman },
    PackageItem { id: "vlc", label: "VLC", packages: &["vlc"], source: PackageSource::Pacman },
    PackageItem { id: "libreoffice", label: "LibreOffice", packages: &["libreoffice-fresh"], source: PackageSource::Pacman },
    PackageItem { id: "7zip", label: "7-Zip", packages: &["7zip"], source: PackageSource::Pacman },
    PackageItem { id: "flatpak", label: "Flatpak", packages: &["flatpak"], source: PackageSource::Pacman },
];

pub const DEVELOPMENT: &[PackageItem] = &[
    PackageItem { id: "git", label: "Git", packages: &["git"], source: PackageSource::Pacman },
    PackageItem { id: "vscodium", label: "VSCodium", packages: &["vscodium"], source: PackageSource::Unavailable },
    PackageItem { id: "dotnet", label: ".NET SDK", packages: &["dotnet-sdk"], source: PackageSource::Pacman },
    PackageItem { id: "python", label: "Python", packages: &["python"], source: PackageSource::Pacman },
    PackageItem { id: "nodejs", label: "Node.js", packages: &["nodejs", "npm"], source: PackageSource::Pacman },
    PackageItem { id: "java", label: "Java", packages: &["jdk-openjdk"], source: PackageSource::Pacman },
    PackageItem { id: "docker", label: "Docker", packages: &["docker", "docker-compose"], source: PackageSource::Pacman },
    PackageItem { id: "dbeaver", label: "DBeaver", packages: &["dbeaver"], source: PackageSource::Pacman },
];

pub const GAMING: &[PackageItem] = &[
    PackageItem { id: "steam", label: "Steam", packages: &["steam"], source: PackageSource::Pacman },
    PackageItem { id: "heroic", label: "Heroic Games Launcher", packages: &["heroic-games-launcher"], source: PackageSource::Pacman },
    PackageItem { id: "lutris", label: "Lutris", packages: &["lutris"], source: PackageSource::Pacman },
    PackageItem { id: "mangohud", label: "MangoHud", packages: &["mangohud"], source: PackageSource::Pacman },
    PackageItem { id: "gamemode", label: "GameMode", packages: &["gamemode"], source: PackageSource::Pacman },
    PackageItem { id: "gamescope", label: "Gamescope", packages: &["gamescope"], source: PackageSource::Pacman },
];

pub const PROFILES: &[PackageProfile] = &[
    PackageProfile {
        id: "essentials",
        title: "Esenciales",
        description: "Instala utilidades recomendadas para un sistema de uso diario.",
        items: ESSENTIALS,
    },
    PackageProfile {
        id: "development",
        title: "Desarrollo",
        description: "Prepara GuepardOS para programación y creación de software.",
        items: DEVELOPMENT,
    },
    PackageProfile {
        id: "gaming",
        title: "Gaming",
        description: "Instala las herramientas principales para jugar en Linux.",
        items: GAMING,
    },
];

pub fn is_installed(item: &PackageItem) -> bool {
    item.packages.iter().all(|package| is_package_installed(package))
}

pub fn missing_packages(items: &[PackageItem]) -> Vec<&'static str> {
    let mut packages = Vec::new();
    for item in items.iter().filter(|item| item.source == PackageSource::Pacman) {
        for package in item.packages {
            if !is_package_installed(package) && !packages.contains(package) {
                packages.push(*package);
            }
        }
    }
    packages
}

pub fn install_packages(packages: &[&str]) -> Result<String, String> {
    if packages.is_empty() {
        return Ok("Todo lo seleccionado ya está instalado.".into());
    }
    if std::path::Path::new("/var/lib/pacman/db.lck").exists() {
        return Err("Pacman está bloqueado por otra operación. Espera a que termine y vuelve a intentarlo.".into());
    }
    if which::which("pacman").is_err() {
        return Err("Pacman no está disponible en este sistema.".into());
    }
    if which::which("pkexec").is_err() {
        return Err("pkexec no está disponible para solicitar permisos de administrador.".into());
    }
    if !has_internet() {
        return Err("No hay conexión a Internet. Conéctate antes de instalar paquetes.".into());
    }

    if INSTALL_RUNNING.swap(true, Ordering::SeqCst) {
        return Err("Ya hay una instalación en curso.".into());
    }

    let status = Exec::cmd("pkexec")
        .arg("pacman")
        .args(&["-S", "--needed"])
        .args(packages)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()
        .map_err(|e| {
            INSTALL_RUNNING.store(false, Ordering::SeqCst);
            e.to_string()
        })?;

    INSTALL_RUNNING.store(false, Ordering::SeqCst);

    if status.exit_status.success() {
        Ok(status.stdout_str())
    } else {
        Err(status.stdout_str())
    }
}

fn is_package_installed(package: &str) -> bool {
    Exec::cmd("pacman")
        .args(&["-Q", package])
        .stdout(Redirection::Null)
        .stderr(Redirection::Null)
        .join()
        .is_ok_and(|status| status.success())
}

fn has_internet() -> bool {
    Exec::cmd("ping")
        .args(&["-c", "1", "-W", "2", "1.1.1.1"])
        .stdout(Redirection::Null)
        .stderr(Redirection::Null)
        .join()
        .is_ok_and(|status| status.success())
}
