use actix_web::http::header::HeaderMap;
use anyhow::Context;
use base64::Engine;
use secrecy::SecretString;

pub struct Credentials {
    pub username: String,
    pub password: SecretString,
}

pub fn basic_auth(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header is missing.")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;

    let base64_encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme is not 'Basic'")?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials")?;

    let decoded_creds =
        String::from_utf8(decoded_bytes).context("The decode credential string is not UTF8")?;

    let mut creds = decoded_creds.splitn(2, ":");
    let username = creds
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();

    let password = creds
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: SecretString::from(password),
    })
}
