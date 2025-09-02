use anyhow::Result;
use url::Url;
use webauthn_rs::prelude::*;

/// WebAuthn helper functions.
pub struct WebAuthn;

impl WebAuthn {
    /// Initialise a WebAuthn instance.
    pub fn init(rp_id: &str, origin: &str, rp_name: &str) -> Result<Webauthn> {
        let url = Url::parse(origin)?;
        Ok(WebauthnBuilder::new(rp_id, &url)?
            .rp_name(rp_name)
            .build()?)
    }
}
