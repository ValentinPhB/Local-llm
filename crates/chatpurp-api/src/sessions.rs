use chatpurp_core::{
    application::{Failure, Sessions},
    policy::{Directory, VerifiedIdentity},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Once},
    time::{SystemTime, UNIX_EPOCH},
};

pub const COOKIE: &str = "chatpurp_demo_session";
pub trait Clock: Send + Sync {
    fn now(&self) -> Result<u64, Failure>;
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Result<u64, Failure> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .map_err(|_| Failure::Session)
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    groups: Vec<String>,
    iss: String,
    aud: String,
    iat: u64,
    exp: u64,
}
pub struct SessionManager {
    pub directory: Arc<Directory>,
    key: [u8; 32],
    clock: Arc<dyn Clock>,
}
impl SessionManager {
    pub fn new(directory: Arc<Directory>, clock: Arc<dyn Clock>) -> Result<Self, Failure> {
        static PROVIDER: Once = Once::new();
        PROVIDER.call_once(|| {
            let _ = jsonwebtoken::crypto::aws_lc::DEFAULT_PROVIDER.install_default();
        });
        let mut key = [0; 32];
        aws_lc_rs::rand::fill(&mut key).map_err(|_| Failure::Session)?;
        Ok(Self {
            directory,
            key,
            clock,
        })
    }
    pub fn issue(&self, id: &str) -> Result<String, Failure> {
        let identity = self.directory.identity(id).ok_or(Failure::Session)?;
        let iat = self.clock.now()?;
        let claims = Claims {
            sub: identity.id.clone(),
            groups: identity.groups.clone(),
            iss: self.directory.issuer.clone(),
            aud: self.directory.audience.clone(),
            iat,
            exp: iat.checked_add(900).ok_or(Failure::Session)?,
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.key),
        )
        .map_err(|_| Failure::Session)
    }
}
impl Sessions for SessionManager {
    fn verify(&self, token: &str) -> Result<VerifiedIdentity, Failure> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        // Signature/issuer/audience restent vérifiés par la bibliothèque ; horloge injectable.
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        validation.set_issuer(&[&self.directory.issuer]);
        validation.set_audience(&[&self.directory.audience]);
        let claims = decode::<Claims>(token, &DecodingKey::from_secret(&self.key), &validation)
            .map_err(|_| Failure::Session)?
            .claims;
        let identity = self
            .directory
            .validate_claims(&claims.sub, &claims.groups)
            .ok_or(Failure::Session)?;
        if claims.exp.checked_sub(claims.iat) != Some(900) || self.clock.now()? >= claims.exp {
            return Err(Failure::Session);
        }
        Ok(identity)
    }
}
/// RFC 6265 cookie-octet ; aucun décodage URL ni confiance dans un autre cookie.
pub fn cookie_token<'a>(headers: impl Iterator<Item = &'a str>) -> Result<Option<String>, Failure> {
    let mut token = None;
    for header in headers {
        for part in header.split(';') {
            let (name, raw) = part.trim().split_once('=').ok_or(Failure::Session)?;
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
            {
                return Err(Failure::Session);
            }
            let value = if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
                &raw[1..raw.len() - 1]
            } else {
                raw
            };
            if !value.bytes().all(|b| {
                b == 0x21
                    || (0x23..=0x2b).contains(&b)
                    || (0x2d..=0x3a).contains(&b)
                    || (0x3c..=0x5b).contains(&b)
                    || (0x5d..=0x7e).contains(&b)
            }) {
                return Err(Failure::Session);
            }
            if name == COOKIE {
                if token.is_some() || value.is_empty() {
                    return Err(Failure::Session);
                }
                token = Some(value.into());
            }
        }
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    struct TestClock(AtomicU64);
    impl Clock for TestClock {
        fn now(&self) -> Result<u64, Failure> {
            let n = self.0.load(Ordering::SeqCst);
            if n == u64::MAX {
                Err(Failure::Session)
            } else {
                Ok(n)
            }
        }
    }
    fn setup() -> (SessionManager, Arc<TestClock>) {
        let clock = Arc::new(TestClock(AtomicU64::new(1000)));
        let dir =
            serde_json::from_str(include_str!("../../../config/demo-idp/directory.json")).unwrap();
        (
            SessionManager::new(Arc::new(dir), clock.clone()).unwrap(),
            clock,
        )
    }
    #[test]
    fn sessions_expire_exactly_and_restart_invalidates() {
        let (sessions, clock) = setup();
        let token = sessions.issue("oscar").unwrap();
        assert_eq!(sessions.verify(&token).unwrap().id(), "oscar");
        clock.0.store(1899, Ordering::SeqCst);
        assert!(sessions.verify(&token).is_ok());
        clock.0.store(1900, Ordering::SeqCst);
        assert!(sessions.verify(&token).is_err());
        assert!(setup().0.verify(&token).is_err());
        assert!(sessions.issue("unknown").is_err());
        clock.0.store(u64::MAX, Ordering::SeqCst);
        assert!(sessions.issue("oscar").is_err());
        assert!(sessions.verify(&token).is_err());
        clock.0.store(u64::MAX - 1, Ordering::SeqCst);
        assert!(sessions.issue("oscar").is_err());
    }
    #[test]
    fn signed_but_inconsistent_claims_are_rejected() {
        let (sessions, _) = setup();
        let token = sessions.issue("oscar").unwrap();
        let base = decode::<Claims>(&token, &DecodingKey::from_secret(&sessions.key), &{
            let mut v = Validation::new(Algorithm::HS256);
            v.validate_exp = false;
            v.validate_aud = false;
            v
        })
        .unwrap()
        .claims;
        let value = serde_json::to_value(base).unwrap();
        for (field, replacement) in [
            ("groups", serde_json::json!(["LAB_READERS"])),
            ("iss", serde_json::json!("other")),
            ("aud", serde_json::json!("other")),
            ("exp", serde_json::json!(1901)),
            ("iat", serde_json::json!(-1)),
            ("sub", serde_json::json!("alice")),
        ] {
            let mut altered = value.clone();
            altered[field] = replacement;
            let t = encode(
                &Header::new(Algorithm::HS256),
                &altered,
                &EncodingKey::from_secret(&sessions.key),
            )
            .unwrap();
            assert!(sessions.verify(&t).is_err(), "{field}");
        }
        let wrong_alg = encode(
            &Header::new(Algorithm::HS384),
            &value,
            &EncodingKey::from_secret(&sessions.key),
        )
        .unwrap();
        assert!(sessions.verify(&wrong_alg).is_err());
    }
    #[test]
    fn cookie_ambiguities_fail_closed() {
        assert_eq!(
            cookie_token(["other=ok; chatpurp_demo_session=abc.def.sig"].into_iter())
                .unwrap()
                .as_deref(),
            Some("abc.def.sig")
        );
        for values in [
            vec!["chatpurp_demo_session=a; chatpurp_demo_session=a"],
            vec!["chatpurp_demo_session=a", "chatpurp_demo_session=b"],
            vec!["chatpurp_demo_session="],
            vec!["other=\"unfinished"],
            vec!["other=two words"],
            vec!["broken"],
        ] {
            assert!(cookie_token(values.into_iter()).is_err());
        }
        assert!(cookie_token(std::iter::empty()).unwrap().is_none());
    }
    struct CountingClock {
        now: AtomicU64,
        calls: AtomicU64,
    }
    impl Clock for CountingClock {
        fn now(&self) -> Result<u64, Failure> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.now.load(Ordering::SeqCst))
        }
    }
    #[test]
    fn missing_malformed_or_inconsistent_signed_claims_never_read_the_clock() {
        let clock = Arc::new(CountingClock {
            now: AtomicU64::new(1000),
            calls: AtomicU64::new(0),
        });
        let directory = Arc::new(
            serde_json::from_str(include_str!("../../../config/demo-idp/directory.json")).unwrap(),
        );
        let sessions = SessionManager::new(directory, clock.clone()).unwrap();
        let valid = serde_json::json!({"sub":"oscar","groups":["OSCAR_PUBLIC_WELCOME_READERS","MCP_READERS"],"iss":"local-demo-idp","aud":"local-llm-lab-api","iat":1000,"exp":1900});
        let mut cases = Vec::new();
        for field in ["sub", "groups", "iss", "aud", "iat", "exp"] {
            let mut v = valid.clone();
            v.as_object_mut().unwrap().remove(field);
            cases.push(v);
        }
        for field in ["iat", "exp"] {
            for value in [
                serde_json::json!(-1),
                serde_json::json!(1.5),
                serde_json::json!("1000"),
                serde_json::Value::Null,
                serde_json::json!(true),
            ] {
                let mut v = valid.clone();
                v[field] = value;
                cases.push(v);
            }
        }
        for (field, value) in [
            ("exp", serde_json::json!(1899)),
            ("exp", serde_json::json!(1901)),
            ("exp", serde_json::json!(0)),
            ("sub", serde_json::json!("unknown")),
            ("groups", serde_json::json!([])),
            ("groups", serde_json::json!(["LAB_READERS"])),
            ("groups", serde_json::json!(["MCP_READERS", "MCP_READERS"])),
        ] {
            let mut v = valid.clone();
            v[field] = value;
            cases.push(v);
        }
        for value in cases {
            let t = encode(
                &Header::new(Algorithm::HS256),
                &value,
                &EncodingKey::from_secret(&sessions.key),
            )
            .unwrap();
            assert!(sessions.verify(&t).is_err());
        }
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
        let oscar = sessions.issue("oscar").unwrap();
        let alice = sessions.issue("alice").unwrap();
        clock.calls.store(0, Ordering::SeqCst);
        let parts: Vec<_> = oscar.split('.').collect();
        let forged = format!(
            "{}.{}.{}",
            parts[0],
            parts[1],
            alice.split('.').nth(2).unwrap()
        );
        assert!(sessions.verify(&forged).is_err());
        assert_eq!(clock.calls.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn every_validation_reads_clock_and_iat_is_not_not_before() {
        let (sessions, clock) = setup();
        let token = sessions.issue("oscar").unwrap();
        clock.0.store(4000, Ordering::SeqCst);
        assert!(sessions.verify(&token).is_err());
        clock.0.store(1000, Ordering::SeqCst);
        assert!(sessions.verify(&token).is_ok());
        clock.0.store(4000, Ordering::SeqCst);
        let future = sessions.issue("oscar").unwrap();
        clock.0.store(1000, Ordering::SeqCst);
        assert!(sessions.verify(&future).is_ok());
    }
}
