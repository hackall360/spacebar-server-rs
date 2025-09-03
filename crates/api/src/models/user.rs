use serde::Serialize;

/// Public user representation sent to clients.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MinimalPublicUser {
    pub avatar: Option<String>,
    pub discriminator: String,
    pub id: String,
    pub public_flags: i32,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge_ids: Option<Vec<String>>,
}
