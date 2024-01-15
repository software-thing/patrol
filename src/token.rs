use std::sync::OnceLock;

use anyhow::anyhow;
use chrono::{Duration, Utc};
use jsonwebtoken::{self, Algorithm, Validation};
use poem::{Endpoint, Request};
use serde::{Deserialize, Serialize};

use crate::{keys, models::users};

pub const PATROL_COOKIE: &'static str = "patrol";

static VALIDATION: OnceLock<Validation> = OnceLock::new();

pub async fn token_middleware<E: Endpoint>(next: E, mut req: Request) -> poem::Result<E::Output> {
    let validation = VALIDATION.get_or_init(|| {
        let mut validation = Validation::new(Algorithm::RS384);
        validation.leeway = 5;
        validation.set_issuer(&["patrol"]);
        validation
    });

    if let Some(token) = req
        .cookie()
        .get(PATROL_COOKIE)
        .map(|cookie| cookie.value_str().to_string())
    {
        let claims: Claims =
            jsonwebtoken::decode(&token, keys::decoding_key().await.unwrap(), validation)
                .unwrap()
                .claims;
        req.extensions_mut().insert(claims);
    }

    // Call the next endpoint
    next.call(req).await
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    /// Issuer
    pub iss: String,

    /// User ID
    pub sub: String,
    /// First name
    pub fnm: String,
    /// Last name
    pub lnm: String,
    /// Profile picture
    pub pic: bool,
    /// Roles
    pub rls: Vec<String>,

    /// Expiration time
    pub exp: i64,
}

impl Claims {
    pub fn new(user: users::Model, roles: Vec<String>) -> Self {
        let year_from_now = Utc::now() + Duration::days(365);

        Self {
            iss: "patrol".to_string(),

            sub: user.id.to_string(),
            fnm: user.first_name,
            lnm: user.last_name,
            pic: user.profile_picture,
            rls: roles,

            exp: year_from_now.timestamp(),
        }
    }
}

pub async fn new(claims: Claims) -> anyhow::Result<String> {
    let header = jsonwebtoken::Header::new(Algorithm::RS384);

    jsonwebtoken::encode(&header, &claims, keys::encoding_key().await?).map_err(|err| anyhow!(err))
}
