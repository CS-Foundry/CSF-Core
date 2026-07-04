use axum::http::Request;
use tower_governor::key_extractor::{KeyExtractor, PeerIpKeyExtractor};
use tower_governor::GovernorError;
use uuid::Uuid;

use super::jwt::verify_jwt;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RateLimitKey {
    User(Uuid),
    Ip(std::net::IpAddr),
}

#[derive(Clone)]
pub struct JwtOrIpKeyExtractor {
    fallback: PeerIpKeyExtractor,
}

impl JwtOrIpKeyExtractor {
    pub fn new() -> Self {
        Self {
            fallback: PeerIpKeyExtractor,
        }
    }
}

impl KeyExtractor for JwtOrIpKeyExtractor {
    type Key = RateLimitKey;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        if let Some(user_id) = extract_user_id(req) {
            return Ok(RateLimitKey::User(user_id));
        }

        self.fallback.extract(req).map(RateLimitKey::Ip)
    }
}

fn extract_user_id<T>(req: &Request<T>) -> Option<Uuid> {
    let header = req.headers().get(axum::http::header::AUTHORIZATION)?;
    let value = header.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?;
    let claims = verify_jwt(token).ok()?;
    Some(claims.claims.user_id)
}
