use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

const PRODUCT: &str = "KMJ_DESKTOP_COMMANDER";
const PROTOCOL: &str = "KSLP-v1";
const RENEWAL_WINDOW_SECONDS: u64 = 86_400;
const OFFLINE_GRACE_SECONDS: u64 = 259_200;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SignedEntitlement {
    pub payload: String,
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntitlementPlan {
    Free,
    Pro,
    Team,
    Enterprise,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommercialState {
    Active,
    RenewalDue,
    Grace,
    Restricted,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EntitlementLimits {
    pub server_profiles: u32,
    pub devices: u32,
    pub autopilot_runs_per_month: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EntitlementClaims {
    pub protocol: String,
    pub license_id: String,
    pub customer_id: String,
    pub product: String,
    pub plan: EntitlementPlan,
    pub activation_id: String,
    pub device_id: String,
    pub device_public_key_fingerprint: String,
    pub enabled_features: Vec<String>,
    pub limits: EntitlementLimits,
    pub commercial_state: CommercialState,
    pub issued_at: u64,
    pub not_before: u64,
    pub lease_expiry: u64,
    pub sequence: u64,
    pub kid: String,
    pub token_id: String,
    pub nonce: String,
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
    expected_device_public_key_fingerprint: &str,
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
    validate_claims(
        &claims,
        expected_device,
        expected_device_public_key_fingerprint,
        now,
    )?;

    let time_state = if now <= claims.lease_expiry.saturating_sub(RENEWAL_WINDOW_SECONDS) {
        CommercialState::Active
    } else if now <= claims.lease_expiry {
        CommercialState::RenewalDue
    } else if now <= claims.lease_expiry.saturating_add(OFFLINE_GRACE_SECONDS) {
        CommercialState::Grace
    } else {
        CommercialState::Restricted
    };
    // A signed server restriction is authoritative; local time may only make the state more restrictive.
    let effective = std::cmp::max(time_state, claims.commercial_state.clone());
    let state = match effective {
        CommercialState::Active => EntitlementState::Active,
        CommercialState::RenewalDue => EntitlementState::RenewalDue,
        CommercialState::Grace => EntitlementState::Grace,
        CommercialState::Restricted => EntitlementState::Restricted,
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

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
}

fn valid_feature(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_')
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_nonce(value: &str) -> bool {
    (16..=128).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn validate_claims(
    claims: &EntitlementClaims,
    expected_device: &str,
    expected_fingerprint: &str,
    now: u64,
) -> Result<(), String> {
    if claims.protocol != PROTOCOL || claims.product != PRODUCT {
        return Err("entitlement product or protocol mismatch".into());
    }
    if claims.device_id != expected_device
        || claims.device_public_key_fingerprint != expected_fingerprint
    {
        return Err("entitlement device binding mismatch".into());
    }
    for value in [
        &claims.license_id,
        &claims.customer_id,
        &claims.activation_id,
        &claims.device_id,
        &claims.kid,
        &claims.token_id,
    ] {
        if !valid_id(value) {
            return Err("invalid entitlement identifier".into());
        }
    }
    if !valid_fingerprint(&claims.device_public_key_fingerprint) {
        return Err("invalid device public-key fingerprint".into());
    }
    if !valid_nonce(&claims.nonce) || claims.sequence == 0 {
        return Err("invalid entitlement replay claims".into());
    }
    let mut features = HashSet::new();
    if claims
        .enabled_features
        .iter()
        .any(|f| !valid_feature(f) || !features.insert(f))
    {
        return Err("invalid or duplicate entitlement feature".into());
    }
    if claims.not_before > now.saturating_add(300)
        || claims.issued_at > now.saturating_add(300)
        || claims.lease_expiry <= claims.not_before
    {
        return Err("invalid entitlement time bounds".into());
    }
    if claims.commercial_state == CommercialState::Restricted && !claims.enabled_features.is_empty()
    {
        return Err("restricted entitlement cannot authorize features".into());
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

    const FP: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn artifact(now: u64, expiry: u64, state: CommercialState) -> (SignedEntitlement, String) {
        let claims = EntitlementClaims {
            protocol: PROTOCOL.into(),
            license_id: "lic-test".into(),
            customer_id: "customer-test".into(),
            product: PRODUCT.into(),
            plan: EntitlementPlan::Free,
            activation_id: "act-test".into(),
            device_id: "device-1".into(),
            device_public_key_fingerprint: FP.into(),
            enabled_features: if state == CommercialState::Restricted {
                vec![]
            } else {
                vec!["manual_operations".into()]
            },
            limits: EntitlementLimits {
                server_profiles: 3,
                devices: 1,
                autopilot_runs_per_month: 50,
            },
            commercial_state: state,
            issued_at: now,
            not_before: now,
            lease_expiry: expiry,
            sequence: 1,
            kid: "test-key".into(),
            token_id: "token-test".into(),
            nonce: "abcdefghijklmnop".into(),
        };
        let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).unwrap());
        let signing = SigningKey::from_bytes(&[7_u8; 32]);
        let signature = URL_SAFE_NO_PAD.encode(signing.sign(payload.as_bytes()).to_bytes());
        let public_key = URL_SAFE_NO_PAD.encode(signing.verifying_key().to_bytes());
        (SignedEntitlement { payload, signature }, public_key)
    }

    #[test]
    fn valid_entitlement_is_verified_offline() {
        let (a, k) = artifact(1_000_000, 2_000_000, CommercialState::Active);
        let v = verify(&a, &k, "device-1", FP, 1_000_001).unwrap();
        assert_eq!(v.state, EntitlementState::Active);
        assert_eq!(v.claims.plan, EntitlementPlan::Free);
    }
    #[test]
    fn wrong_device_is_denied() {
        let (a, k) = artifact(1_000_000, 2_000_000, CommercialState::Active);
        assert!(verify(&a, &k, "device-2", FP, 1_000_001).is_err());
    }
    #[test]
    fn wrong_fingerprint_is_denied() {
        let (a, k) = artifact(1_000_000, 2_000_000, CommercialState::Active);
        assert!(
            verify(
                &a,
                &k,
                "device-1",
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                1_000_001
            )
            .is_err()
        );
    }
    #[test]
    fn tampering_is_denied() {
        let (mut a, k) = artifact(1_000_000, 2_000_000, CommercialState::Active);
        a.payload.push('A');
        assert!(verify(&a, &k, "device-1", FP, 1_000_001).is_err());
    }
    #[test]
    fn expired_entitlement_enters_grace_without_network() {
        let (a, k) = artifact(1_000_000, 1_100_000, CommercialState::Active);
        assert_eq!(
            verify(&a, &k, "device-1", FP, 1_100_001).unwrap().state,
            EntitlementState::Grace
        );
    }
    #[test]
    fn long_expired_entitlement_is_restricted_not_destroyed() {
        let (a, k) = artifact(1_000_000, 1_100_000, CommercialState::Active);
        assert_eq!(
            verify(
                &a,
                &k,
                "device-1",
                FP,
                1_100_000 + OFFLINE_GRACE_SECONDS + 1
            )
            .unwrap()
            .state,
            EntitlementState::Restricted
        );
    }
    #[test]
    fn signed_restriction_cannot_be_relaxed_by_local_time() {
        let (a, k) = artifact(1_000_000, 2_000_000, CommercialState::Restricted);
        assert_eq!(
            verify(&a, &k, "device-1", FP, 1_000_001).unwrap().state,
            EntitlementState::Restricted
        );
    }
    #[test]
    fn unknown_claims_are_rejected() {
        let (a, k) = artifact(1_000_000, 2_000_000, CommercialState::Active);
        let mut raw = URL_SAFE_NO_PAD.decode(&a.payload).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), true.into());
        raw = serde_json::to_vec(&value).unwrap();
        let payload = URL_SAFE_NO_PAD.encode(raw);
        let signing = SigningKey::from_bytes(&[7_u8; 32]);
        let bad = SignedEntitlement {
            signature: URL_SAFE_NO_PAD.encode(signing.sign(payload.as_bytes()).to_bytes()),
            payload,
        };
        assert!(verify(&bad, &k, "device-1", FP, 1_000_001).is_err());
    }
}
