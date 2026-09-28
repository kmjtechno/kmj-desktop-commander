use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TokenClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub server: String,
    pub iat: u64,
    pub nbf: u64,
    pub exp: u64,
    pub jti: String,
    pub scopes: Vec<String>,
}

pub fn mint(claims: &TokenClaims, secret: &str) -> Result<String, String> {
    validate_claims(claims, claims.iat, &claims.server)?;
    let payload = serde_json::to_vec(claims).map_err(|e| e.to_string())?;
    let encoded = URL_SAFE_NO_PAD.encode(payload);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| "invalid signing secret")?;
    mac.update(encoded.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{encoded}.{signature}"))
}

pub fn verify_bearer(
    header: Option<&str>,
    secret: &str,
    now: u64,
    server: &str,
) -> Result<TokenClaims, String> {
    let token = header
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or("missing bearer token")?;
    let (encoded, signature) = token.split_once('.').ok_or("malformed token")?;
    let sig = URL_SAFE_NO_PAD
        .decode(signature)
        .map_err(|_| "malformed signature")?;
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| "invalid signing secret")?;
    mac.update(encoded.as_bytes());
    mac.verify_slice(&sig).map_err(|_| "invalid signature")?;
    let payload = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| "malformed claims")?;
    let claims: TokenClaims =
        serde_json::from_slice(&payload).map_err(|_| "malformed claims")?;
    validate_claims(&claims, now, server)?;
    Ok(claims)
}

fn validate_claims(claims: &TokenClaims, now: u64, server: &str) -> Result<(), String> {
    if claims.iss != "kmj-commander" || claims.aud != "kmj-vps" || claims.server != server {
        return Err("token binding denied".into());
    }
    if claims.sub.is_empty() || claims.jti.len() < 16 || claims.scopes.is_empty() {
        return Err("incomplete token claims".into());
    }
    if claims.exp <= claims.iat || claims.exp.saturating_sub(claims.iat) > 300 {
        return Err("token lifetime denied".into());
    }
    if now < claims.nbf || now > claims.exp {
        return Err("token expired or not active".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims() -> TokenClaims {
        TokenClaims {
            iss: "kmj-commander".into(),
            sub: "chatgpt".into(),
            aud: "kmj-vps".into(),
            server: "lab".into(),
            iat: 100,
            nbf: 100,
            exp: 400,
            jti: "0123456789abcdef".into(),
            scopes: vec!["cloudos:read".into()],
        }
    }

    #[test]
    fn signed_token_round_trip() {
        let token = mint(&claims(), "01234567890123456789012345678901").unwrap();
        assert_eq!(
            verify_bearer(
                Some(&format!("Bearer {token}")),
                "01234567890123456789012345678901",
                200,
                "lab"
            )
            .unwrap()
            .sub,
            "chatgpt"
        );
    }

    #[test]
    fn wrong_secret_denied() {
        let token = mint(&claims(), "01234567890123456789012345678901").unwrap();
        assert!(
            verify_bearer(
                Some(&format!("Bearer {token}")),
                "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
                200,
                "lab"
            )
            .is_err()
        );
    }

    #[test]
    fn expired_denied() {
        let token = mint(&claims(), "01234567890123456789012345678901").unwrap();
        assert!(
            verify_bearer(
                Some(&format!("Bearer {token}")),
                "01234567890123456789012345678901",
                401,
                "lab"
            )
            .is_err()
        );
    }
}
