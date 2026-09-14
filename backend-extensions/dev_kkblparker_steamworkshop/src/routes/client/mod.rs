use shared::State;
use utoipa_axum::router::OpenApiRouter;

mod apply;
mod installed;
mod load_order;
mod mods;

pub fn router(state: &State) -> OpenApiRouter<State> {
    OpenApiRouter::new()
        .nest("/steam-workshop/mods", mods::router(state))
        .nest("/steam-workshop/installed", installed::router(state))
        .nest("/steam-workshop/apply", apply::router(state))
        .nest("/steam-workshop/load-order", load_order::router(state))
        .with_state(state.clone())
}
