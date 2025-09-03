use std::time::{SystemTime, UNIX_EPOCH};

use config::Config;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Validate CDN signature parameters for a given path.
pub fn has_valid_signature(
    path: &str,
    ex: &str,
    is: &str,
    hm: &str,
    ip: Option<&str>,
    user_agent: Option<&str>,
    config: &Config,
) -> bool {
    // Parse issued/expires timestamps (hex encoded).
    let issued = u64::from_str_radix(is, 16).ok();
    let expires = u64::from_str_radix(ex, 16).ok();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    if let (Some(i), Some(e)) = (issued, expires) {
        if e < now || i > now {
            return false;
        }
    } else {
        return false;
    }

    let mut mac = match HmacSha256::new_from_slice(config.security.cdn_signature_key.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(path.as_bytes());
    mac.update(is.as_bytes());
    mac.update(ex.as_bytes());

    if config.security.cdn_signature_include_ip {
        if let Some(ip) = ip {
            mac.update(ip.as_bytes());
        }
    }
    if config.security.cdn_signature_include_user_agent {
        if let Some(ua) = user_agent {
            mac.update(ua.as_bytes());
        }
    }

    let calc = mac.finalize().into_bytes();
    let expected = hex::decode(hm).unwrap_or_default();

    use subtle::ConstantTimeEq;
    calc.ct_eq(&expected).into()
}
