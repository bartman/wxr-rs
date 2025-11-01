use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Claims {
    pub id: u32,
}

pub fn decode_token(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid token format".into());
    }
    let payload = parts[1];
    let decoded = general_purpose::URL_SAFE_NO_PAD.decode(payload)?;
    let claims: Claims = serde_json::from_slice(&decoded)?;
    Ok(claims)
}