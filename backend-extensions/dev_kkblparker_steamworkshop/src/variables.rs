use crate::ini_storage;
use axum::http::StatusCode;
use shared::{models::server_variable::ServerVariable, response::ApiResponse};

/// Where a server's Workshop mod list actually lives - eggs vary, so this is resolved per-server
/// from a handful of hidden variables (see `resolve`) rather than assumed.
enum Storage {
    /// e.g. Arma 3's `MODIFICATIONS` variable: a semicolon-separated list that mixes `@<id>`
    /// Workshop tokens with other tokens (CDLC names, manually uploaded mod folders) the
    /// extension must leave untouched.
    Variable {
        variable_uuid: uuid::Uuid,
        /// Every token, verbatim - only `@<digits>` ones are touched on write.
        tokens: Vec<String>,
    },
    /// e.g. Project Zomboid's `Zomboid/Server/<name>.ini`: a plain file on the server (not an
    /// egg variable) with a `WorkshopItems=` key holding plain (no `@`) semicolon-separated IDs,
    /// and optionally a second key (e.g. `Mods=`) for the separate, manually-ordered list of
    /// mod.info IDs those Workshop items actually provide - the Steam Web API has no way to
    /// resolve one from the other, so that second key is exposed as a raw, user-edited value
    /// rather than something this extension tries to compute (see `load_order`/`set_load_order`).
    IniFile {
        path: String,
        items_key: String,
        mods_key: Option<String>,
        /// The file's full current content, so a write can preserve everything else untouched.
        content: String,
    },
}

pub struct WorkshopConfig {
    pub appid: String,
    storage: Storage,
}

fn missing() -> ApiResponse {
    ApiResponse::error("steam workshop is not supported for this server's egg")
        .with_status(StatusCode::BAD_REQUEST)
}

fn workshop_token(id: &str) -> String {
    format!("@{id}")
}

fn token_to_id(token: &str) -> Option<&str> {
    let id = token.strip_prefix('@')?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
}

impl WorkshopConfig {
    /// The server's current Steam Workshop mod list (plain IDs, no `@` prefix), in the order
    /// they appear in the underlying storage.
    pub fn mod_ids(&self) -> Vec<String> {
        match &self.storage {
            Storage::Variable { tokens, .. } => tokens
                .iter()
                .filter_map(|token| token_to_id(token))
                .map(str::to_string)
                .collect(),
            Storage::IniFile {
                items_key, content, ..
            } => ini_list(content, items_key),
        }
    }

    /// Whether this egg exposes a second, separately-ordered "which mods to actually load" list
    /// distinct from the Workshop ID list (Project Zomboid's `Mods=`; Arma 3 has no equivalent -
    /// its single `MODIFICATIONS` list serves both purposes).
    pub fn load_order_supported(&self) -> bool {
        matches!(
            &self.storage,
            Storage::IniFile {
                mods_key: Some(_),
                ..
            }
        )
    }

    pub fn load_order(&self) -> Option<String> {
        match &self.storage {
            Storage::IniFile {
                mods_key: Some(key),
                content,
                ..
            } => ini_storage::get_key(content, key),
            _ => None,
        }
    }
}

fn ini_list(content: &str, key: &str) -> Vec<String> {
    ini_storage::get_key(content, key)
        .map(|value| {
            value
                .split(';')
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Resolves where a server's Workshop mod list lives from its egg's hidden variables:
///
/// - `WORKSHOP_APPID` (always required) - the Steam app ID Workshop items belong to. Often
///   DIFFERENT from whatever app ID steamcmd uses to install the dedicated server binaries -
///   e.g. Arma 3's dedicated server is app 233780, but its Workshop content belongs to the base
///   GAME, app 107410 (confirmed against the actual egg's entrypoint script). Project Zomboid is
///   the same shape: server app 380870, Workshop app 108600 (confirmed live via
///   GetPublishedFileDetails' `consumer_app_id` on real, current PZ Workshop items).
/// - `WORKSHOP_STORAGE` (optional, defaults to `variable`) - `variable` or `ini_file`.
///
/// `variable` mode additionally needs `WORKSHOP_MODS_VARIABLE` (the *name* of the egg variable
/// holding the semicolon list, e.g. `MODIFICATIONS`).
///
/// `ini_file` mode additionally needs `WORKSHOP_INI_DIR` (directory relative to the server root,
/// e.g. `Zomboid/Server`), `WORKSHOP_INI_NAME_VARIABLE` (the name of the egg variable whose
/// current value is the file's base name, e.g. `SERVER_NAME` - PZ names its ini after the
/// server), `WORKSHOP_INI_ITEMS_KEY` (e.g. `WorkshopItems`), and optionally
/// `WORKSHOP_INI_MODS_KEY` (e.g. `Mods`).
pub(crate) async fn resolve(
    state: &shared::State,
    server: &mut shared::models::server::Server,
) -> Result<WorkshopConfig, ApiResponse> {
    if !server
        .egg
        .features
        .iter()
        .any(|feature| feature == "steam_workshop")
    {
        return Err(missing());
    }

    let variables =
        ServerVariable::all_by_server_uuid_egg_uuid(&state.database, server.uuid, server.egg.uuid)
            .await
            .map_err(ApiResponse::from)?;

    let var = |name: &str| {
        variables
            .iter()
            .find(|v| v.variable.env_variable == name)
            .map(|v| v.value.clone())
    };

    let appid = var("WORKSHOP_APPID").ok_or_else(missing)?;
    let storage_mode = var("WORKSHOP_STORAGE").unwrap_or_else(|| "variable".to_string());

    let storage = match storage_mode.as_str() {
        "ini_file" => {
            let dir = var("WORKSHOP_INI_DIR").ok_or_else(missing)?;
            let name_variable = var("WORKSHOP_INI_NAME_VARIABLE").ok_or_else(missing)?;
            let name_value = var(&name_variable).ok_or_else(missing)?;
            let items_key = var("WORKSHOP_INI_ITEMS_KEY").ok_or_else(missing)?;
            let mods_key = var("WORKSHOP_INI_MODS_KEY").filter(|value| !value.is_empty());

            let path = format!("{}/{name_value}.ini", dir.trim_end_matches('/'));
            let content = ini_storage::read(state, server, &path)
                .await
                .map_err(ApiResponse::from)?;

            Storage::IniFile {
                path,
                items_key,
                mods_key,
                content,
            }
        }
        _ => {
            let mods_variable_name = var("WORKSHOP_MODS_VARIABLE").ok_or_else(missing)?;
            let mods_variable = variables
                .iter()
                .find(|v| v.variable.env_variable == mods_variable_name)
                .ok_or_else(missing)?;

            let tokens = mods_variable
                .value
                .split(';')
                .map(str::trim)
                .filter(|token| !token.is_empty())
                .map(str::to_string)
                .collect();

            Storage::Variable {
                variable_uuid: mods_variable.variable.uuid,
                tokens,
            }
        }
    };

    Ok(WorkshopConfig { appid, storage })
}

/// Adds a Workshop item's ID to the mod list, preserving every other token/key exactly as-is.
pub(crate) async fn add_mod(
    state: &shared::State,
    server: &mut shared::models::server::Server,
    user_uuid: uuid::Uuid,
    config: &mut WorkshopConfig,
    id: &str,
) -> Result<(), anyhow::Error> {
    match &mut config.storage {
        Storage::Variable { tokens, .. } => {
            let token = workshop_token(id);
            if !tokens.iter().any(|existing| existing == &token) {
                tokens.push(token);
            }
        }
        Storage::IniFile {
            items_key, content, ..
        } => {
            let mut ids = ini_list(content, items_key);
            if !ids.iter().any(|existing| existing == id) {
                ids.push(id.to_string());
            }
            *content = ini_storage::set_key(content, items_key, &ids.join(";"));
        }
    }

    persist(state, server, user_uuid, config).await
}

/// Removes a Workshop item's ID from the mod list, leaving everything else as-is.
pub(crate) async fn remove_mod(
    state: &shared::State,
    server: &mut shared::models::server::Server,
    user_uuid: uuid::Uuid,
    config: &mut WorkshopConfig,
    id: &str,
) -> Result<(), anyhow::Error> {
    match &mut config.storage {
        Storage::Variable { tokens, .. } => {
            let token = workshop_token(id);
            tokens.retain(|existing| existing != &token);
        }
        Storage::IniFile {
            items_key, content, ..
        } => {
            let ids: Vec<String> = ini_list(content, items_key)
                .into_iter()
                .filter(|existing| existing != id)
                .collect();
            *content = ini_storage::set_key(content, items_key, &ids.join(";"));
        }
    }

    persist(state, server, user_uuid, config).await
}

/// Overwrites the separate "which mods to load" list (see `WorkshopConfig::load_order_supported`)
/// with a raw, user-provided value - this extension has no reliable way to derive a Workshop
/// item's internal mod.info ID(s) from the Steam Web API, so unlike the Workshop ID list itself,
/// this one is edited as free text rather than per-item add/remove.
pub(crate) async fn set_load_order(
    state: &shared::State,
    server: &mut shared::models::server::Server,
    user_uuid: uuid::Uuid,
    config: &mut WorkshopConfig,
    value: &str,
) -> Result<(), anyhow::Error> {
    match &mut config.storage {
        Storage::IniFile {
            mods_key: Some(key),
            content,
            ..
        } => {
            *content = ini_storage::set_key(content, key, value);
        }
        _ => return Err(anyhow::anyhow!("this egg has no separate load-order list")),
    }

    persist(state, server, user_uuid, config).await
}

async fn persist(
    state: &shared::State,
    server: &mut shared::models::server::Server,
    user_uuid: uuid::Uuid,
    config: &WorkshopConfig,
) -> Result<(), anyhow::Error> {
    match &config.storage {
        Storage::Variable {
            variable_uuid,
            tokens,
        } => {
            let value = if tokens.is_empty() {
                String::new()
            } else {
                format!("{};", tokens.join(";"))
            };

            ServerVariable::create(&state.database, server.uuid, *variable_uuid, &value).await?;
        }
        Storage::IniFile { path, content, .. } => {
            ini_storage::write(state, server, user_uuid, path, content).await?;
        }
    }

    Ok(())
}
