use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstanceInfo {
    id: String,
    name: String,
    description: Option<String>,
    image: Option<String>,
    correspondence_email: Option<String>,
    correspondence_user_id: Option<String>,
    front_page: Option<String>,
    tos_page: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PingResponse {
    ping: &'static str,
    instance: InstanceInfo,
}

async fn handler(State(state): State<AppState>) -> Json<PingResponse> {
    let general = &state.config.general;
    let resp = PingResponse {
        ping: "pong!",
        instance: InstanceInfo {
            id: general.instance_id.clone(),
            name: general.instance_name.clone(),
            description: general.instance_description.clone(),
            image: general.image.clone(),
            correspondence_email: general.correspondence_email.clone(),
            correspondence_user_id: general.correspondence_user_id.clone(),
            front_page: general.front_page.clone(),
            tos_page: general.tos_page.clone(),
        },
    };
    Json(resp)
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handler))
}
