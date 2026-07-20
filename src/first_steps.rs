use crate::utils::{fix_path, read_json, write_json};
use serde::{Deserialize, Serialize};

pub const TASKS: &[(&str, &str)] = &[
    ("internet", "Conectarse a Internet"),
    ("updates", "Buscar actualizaciones"),
    ("wallpaper", "Elegir fondo de pantalla"),
    ("display", "Revisar pantalla y escalado"),
    ("apps", "Instalar aplicaciones"),
    ("backups", "Configurar copias de seguridad"),
    ("reboot", "Reiniciar después de actualizar, si es necesario"),
];

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FirstStepsState {
    pub completed: Vec<String>,
}

pub fn load(path: &str) -> FirstStepsState {
    let path = state_path(path);
    read_json(&path)
        .ok()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default()
}

pub fn save(path: &str, state: &FirstStepsState) {
    if let Ok(value) = serde_json::to_value(state) {
        let _ = write_json(&state_path(path), &value);
    }
}

pub fn set_completed(state: &mut FirstStepsState, id: &str, completed: bool) {
    if completed {
        if !state.completed.iter().any(|item| item == id) {
            state.completed.push(id.to_owned());
        }
    } else {
        state.completed.retain(|item| item != id);
    }
}

fn state_path(base_path: &str) -> String {
    let base = fix_path(base_path);
    base.strip_suffix(".json")
        .map(|prefix| format!("{prefix}-first-steps.json"))
        .unwrap_or_else(|| format!("{base}-first-steps.json"))
}
