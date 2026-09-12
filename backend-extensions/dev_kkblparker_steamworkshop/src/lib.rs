//! Steam Workshop — a Calagopus backend extension.
//!
//! Unlike the Arma Reforger and Minecraft Workshop extensions, Steam Workshop
//! content has no direct download URL (verified live against Steam's own
//! API: `file_url` comes back empty for both a well-known Arma 3 mod and a
//! well-known Garry's Mod addon) - retrieving it requires `steamcmd
//! +workshop_download_item`.
//!
//! So this extension is a **mod list manager**, not an instant installer -
//! but NOT a reinstall trigger either. Inspecting the actual, current
//! `ghcr.io/parkervcp/games:arma3` image's `entrypoint.sh` (pulled and
//! extracted directly from GHCR) showed that eggs built on it already scan a
//! semicolon-separated mods variable (e.g. `MODIFICATIONS`) for `@<id>`
//! tokens and run `steamcmd +workshop_download_item` for each one on EVERY
//! server start when the egg's "Automatic Updates" variable is enabled - not
//! just on install. So this extension just edits that variable directly
//! (see `variables.rs` - it only touches `@<digits>` tokens, leaving any
//! CDLC names or manually-uploaded mod folders already in the list alone),
//! and `/workshop/apply` triggers a normal **restart** (the same power
//! action the server's own Restart button sends), not a reinstall.
//!
//! Searching the Workshop needs a Steam Web API key (admin-configured, see
//! `settings`/`config`) - `IPublishedFileService/QueryFiles` isn't keyless,
//! unlike `GetPublishedFileDetails` (fetching a known item's own details).
//!
//! Uses its own distinct permission group (`steam_workshop`), same reasoning
//! as the Minecraft extension: reusing another extension's group name would
//! clobber its permission definitions in the global registry.

mod config;
mod routes;
mod settings;
mod steam;
mod variables;

use indexmap::IndexMap;
use shared::{
    State,
    extensions::{
        Extension, ExtensionPermissionsBuilder, ExtensionRouteBuilder,
        settings::ExtensionSettingsDeserializer,
    },
    permissions::PermissionGroup,
};
use std::sync::Arc;

#[derive(Default)]
pub struct ExtensionStruct;

#[async_trait::async_trait]
impl Extension for ExtensionStruct {
    async fn settings_deserializer(&self, _state: State) -> ExtensionSettingsDeserializer {
        Arc::new(settings::SteamWorkshopSettingsDeserializer)
    }

    async fn initialize_permissions(
        &mut self,
        _state: State,
        builder: ExtensionPermissionsBuilder,
    ) -> ExtensionPermissionsBuilder {
        builder.add_server_permission_group(
            "steam_workshop",
            PermissionGroup {
                description: "Permissions that control the ability to manage this server's Steam Workshop mod list.",
                permissions: IndexMap::from([
                    (
                        "read",
                        "Allows browsing the Steam Workshop and viewing the server's mod list.",
                    ),
                    (
                        "manage",
                        "Allows adding/removing mods from the list and applying changes (restarting the server).",
                    ),
                ]),
            },
        )
    }

    async fn initialize_router(
        &mut self,
        state: State,
        builder: ExtensionRouteBuilder,
    ) -> ExtensionRouteBuilder {
        builder
            .add_client_server_api_router(|router| router.merge(routes::client::router(&state)))
            .add_admin_api_router(|router| router.merge(routes::admin::router(&state)))
    }
}
