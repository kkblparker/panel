use axum::http::StatusCode;
use shared::{models::server_variable::ServerVariable, response::ApiResponse};

pub struct WorkshopConfig {
    pub appid: String,
    mods_variable_uuid: uuid::Uuid,
    /// The mods variable's raw value, split on `;` - includes non-Workshop tokens (CDLC/manually
    /// uploaded mod folder names) verbatim so they survive a round-trip untouched. Workshop items
    /// are the tokens matching `@<digits>`.
    tokens: Vec<String>,
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
    /// Every `@<digits>` token currently in the mods variable - i.e. the server's current Steam
    /// Workshop mod list, in the same order they appear in the underlying egg variable.
    pub fn mod_ids(&self) -> Vec<String> {
        self.tokens
            .iter()
            .filter_map(|token| token_to_id(token))
            .map(str::to_string)
            .collect()
    }

    fn serialize(&self) -> String {
        if self.tokens.is_empty() {
            String::new()
        } else {
            format!("{};", self.tokens.join(";"))
        }
    }
}

/// Resolves the two hidden variables an egg must declare to support this extension:
///
/// - `WORKSHOP_APPID` - the Steam app ID Workshop items belong to (this is often DIFFERENT from
///   whatever app ID steamcmd uses to install the dedicated server binaries - e.g. Arma 3's
///   dedicated server is app 233780, but its Workshop content is associated with the base GAME,
///   app 107410, confirmed against the actual egg's own entrypoint script, which hardcodes
///   `GAME_ID=107410` for `+workshop_download_item` while using `STEAMCMD_APPID` for the server).
/// - `WORKSHOP_MODS_VARIABLE` - the *name* of whichever other variable on this egg holds the
///   semicolon-separated mod list steamcmd actually reads at startup (e.g. `MODIFICATIONS` for
///   Arma 3) - a level of indirection so this extension works with any steamcmd egg's own
///   convention without hardcoding one variable name.
pub(crate) async fn resolve(
    state: &shared::State,
    server: &shared::models::server::Server,
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

    let appid = variables
        .iter()
        .find(|v| v.variable.env_variable == "WORKSHOP_APPID")
        .map(|v| v.value.clone())
        .ok_or_else(missing)?;

    let mods_variable_name = variables
        .iter()
        .find(|v| v.variable.env_variable == "WORKSHOP_MODS_VARIABLE")
        .map(|v| v.value.clone())
        .ok_or_else(missing)?;

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

    Ok(WorkshopConfig {
        appid,
        mods_variable_uuid: mods_variable.variable.uuid,
        tokens,
    })
}

/// Adds a Workshop item's `@<id>` token to the mods variable, preserving every other token
/// (CDLC names, manually uploaded mod folders, other Workshop items) exactly as they were.
pub(crate) async fn add_mod(
    state: &shared::State,
    server_uuid: uuid::Uuid,
    config: &mut WorkshopConfig,
    id: &str,
) -> Result<(), anyhow::Error> {
    let token = workshop_token(id);
    if !config.tokens.iter().any(|existing| existing == &token) {
        config.tokens.push(token);
    }

    persist(state, server_uuid, config).await
}

/// Removes a Workshop item's `@<id>` token from the mods variable, leaving everything else as-is.
pub(crate) async fn remove_mod(
    state: &shared::State,
    server_uuid: uuid::Uuid,
    config: &mut WorkshopConfig,
    id: &str,
) -> Result<(), anyhow::Error> {
    let token = workshop_token(id);
    config.tokens.retain(|existing| existing != &token);

    persist(state, server_uuid, config).await
}

async fn persist(
    state: &shared::State,
    server_uuid: uuid::Uuid,
    config: &WorkshopConfig,
) -> Result<(), anyhow::Error> {
    ServerVariable::create(
        &state.database,
        server_uuid,
        config.mods_variable_uuid,
        &config.serialize(),
    )
    .await?;

    Ok(())
}
