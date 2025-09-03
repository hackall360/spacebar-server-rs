use serde::Deserialize;

/// Schema representing a login request body.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
    pub undelete: Option<bool>,
    pub captcha_key: Option<String>,
    pub login_source: Option<String>,
    pub gift_code_sku_id: Option<String>,
}

impl LoginRequest {
    /// Validate the request according to length constraints.
    pub fn validate(&self) -> Result<(), String> {
        let len = self.password.chars().count();
        if !(1..=72).contains(&len) {
            return Err("password length must be between 1 and 72 characters".into());
        }
        Ok(())
    }
}
