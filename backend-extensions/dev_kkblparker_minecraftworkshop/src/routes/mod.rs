use utoipa_axum::router::OpenApiRouter;

mod installed;
mod mods;

pub fn router(state: &shared::State) -> OpenApiRouter<shared::State> {
    OpenApiRouter::new()
        .nest("/minecraft-workshop/mods", mods::router(state))
        .nest("/minecraft-workshop/installed", installed::router(state))
        .with_state(state.clone())
}
