use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const PRODUCT: &str = "KMJ_DESKTOP_COMMANDER";
const PROTOCOL: &str = "KSLP-v1";
const RENEWAL_WINDOW_SECONDS: u64 = 86_400;
const OFFLINE_GRACE_SECONDS: u64 = 259_200;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedEntitlement {
    pub payload: String,
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EntitlementClaims {
    pub protocol: String,
    pub license_id: String,
    pub customer_id: String,
    pub product: String,
    pub plan: String,
    pub device_id: String,
    pub enabled_features: Vec<String>,
    pub server_profiles: u32,
    pub devices: u32,
    pub autopilot_runs_per_month: u32,
    pub issued_at: u64,
    pub not_before: u64,
    pub lease_expiry: u64,
    pub sequence: u64,
    pub kid: String,
    pub token_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntitlementState {
    Active,
    RenewalDue,
    Grace,
    Restricted,
}

#[derive(Clone, Debug, Serialize)]
pub struct VerifiedEntitlement {
    pub claims: EntitlementClaims,
    pub state: EntitlementState,
    pub next_refresh_after_seconds: u64,
}

pub fn verify(
    artifact: &SignedEntitlement,
    public_key: &str,
    expected_device: &str,
    now: u64,
) -> Result<VerifiedEntitlement, String> {
    let key_bytes = URL_SAFE_NO_PAD
        .decode(public_key)
        .map_err(|_| "invalid public key encoding")?;
    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "invalid Ed25519 public key length")?;
    let key = VerifyingKey::from_bytes(&key_array).map_err(|_| "invalid Ed25519 public key")?;

    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&artifact.signature)
        .map_err(|_| "invalid signature encoding")?;
    let signature = Signature::from_slice(&signature_bytes).map_err(|_| "invalid signature")?;
    key.verify(artifact.payload.as_bytes(), &signature)
        .map_err(|_| "entitlement signature rejected")?;

    let payload = URL_SAFE_NO_PAD
        .decode(&artifact.payload)
        .map_err(|_| "invalid entitlement payload encoding")?;
    let claims: EntitlementClaims =
        serde_json::from_slice(&payload).map_err(|_| "invalid entitlement payload")?;

    validate_claims(&claims, expected_device, now)?;

    let state = if now <= claims.lease_expiry.saturating_sub(RENEWAL_WINDOW_SECONDS) {
        EntitlementState::Active
    } else if now <= claims.lease_expiry {
        EntitlementState::RenewalDue
    } else if now <= claims.lease_expiry.saturating_add(OFFLINE_GRACE_SECONDS) {
        EntitlementState::Grace
    } else {
        EntitlementState::Restricted
    };

    let next_refresh_after_seconds = match state {
        EntitlementState::Active => claims
            .lease_expiry
            .saturating_sub(now)
            .saturating_sub(RENEWAL_WINDOW_SECONDS)
            .clamp(21_600, 604_800),
        EntitlementState::RenewalDue => 21_600,
        EntitlementState::Grace => 3_600,
        EntitlementState::Restricted => 21_600,
    };

    Ok(VerifiedEntitlement {
        claims,
        state,
        next_refresh_after_seconds,
    })
}

fn validate_claims(
    claims: &EntitlementClaims,
    expected_device: &str,
    now: u64,
) -> Result<(), String> {
    if claims.protocol != PROTOCOL || claims.product != PRODUCT {
        return Err("entitlement product or protocol mismatch".into());
    }
    if claims.device_id != expected_device {
        return Err("entitlement device binding mismatch".into());
    }
    if claims.license_id.is_empty()
        || claims.customer_id.is_empty()
        || claims.plan.is_empty()
        || claims.kid.is_empty()
        || claims.token_id.is_empty()
        || claims.sequence == 0
    {
        return Err("incomplete entitlement claims".into());
    }
    if claims.not_before > now.saturating_add(300)
        || claims.issued_at > now.saturating_add(300)
        || claims.lease_expiry <= claims.not_before
    {
        return Err("invalid entitlement time bounds".into());
    }
    Ok(())
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn artifact(now: u64, expiry: u64) -> (SignedEntitlement, String) {
        let claims = EntitlementClaims {
            protocol: PROTOCOL.into(),
            license_id: "lic-test".into(),
            customer_id: "customer-test".into(),
            product: PRODUCT.into(),
            plan: "free".into(),
            device_id: "device-1".into(),
            enabled_features: vec!["manual_operations".into()],
            server_profiles: 3,
            devices: 1,
            autopilot_runs_per_month: 50,
            issued_at: now,
            not_before: now,
            lease_expiry: expiry,
            sequence: 1,
            kid: "test-key".into(),
            token_id: "token-test".into(),
        };
        let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).unwrap());
        let signing = SigningKey::from_bytes(&[7_u8; 32]);
        let signature = URL_SAFE_NO_PAD.encode(signing.sign(payload.as_bytes()).to_bytes());
        let public_key = URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes());
        (SignedEntitlement { payload, signature }, public_key)
    }

    #[test]
    fn valid_entitlement_is_verified_offline() {
        let (artifact, key) = artifact(1_000_000, 2_000_000);
        let verified = verify(&artifact, &key, "device-1", 1_000_001).unwrap();
        assert_eq!(verified.state, EntitlementState::Active);
        assert_eq!(verified.claims.plan, "free");
    }

    #[test]
    fn wrong_device_is_denied() {
        let (artifact, key) = artifact(1_000_000, 2_000_000);
        assert!(verify(&artifact, &key, "device-2", 1_000_001).is_err());
    }

    #[test]
    fn tampering_is_denied() {
        let (mut artifact, key) = artifact(1_000_000, 2_000_000);
        artifact.payload.push('A');
        assert!(verify(&artifact, &key, "device-1", 1_000_001).is_err());
    }

    #[test]
    fn expired_entitlement_enters_grace_without_network() {
        let (artifact, key) = artifact(1_000_000, 1_100_000);
        let verified = verify(&artifact, &key, "device-1", 1_100_001).unwrap();
        assert_eq!(verified.state, EntitlementState::Grace);
    }

    #[test]
    fn long_expired_entitlement_is_restricted_not_destroyed() {
        let (artifact, key) = artifact(1_000_000, 1_100_000);
        let verified = verify(
            &artifact,
            &key,
            "device-1",
            1_100_000 + OFFLINE_GRACE_SECONDS + 1,
        )
        .unwrap();
        assert_eq!(verified.state, EntitlementState::Restricted);
    }
}
