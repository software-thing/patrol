use std::sync::OnceLock;

use anyhow::anyhow;
use chrono::{Duration, Utc};
use jsonwebtoken::{self, Algorithm, Validation};
use poem::{http::StatusCode, web::Redirect, Endpoint, IntoResponse, Request, Response};
use redis::{aio::ConnectionManager, AsyncCommands, ExpireOption};
use serde::{Deserialize, Serialize};

use crate::{crypto, keys, models::users};

pub const PATROL_COOKIE: &'static str = "patrol";

static VALIDATION: OnceLock<Validation> = OnceLock::new();

async fn is_logged_in(req: &mut Request) -> bool {
    let validation = VALIDATION.get_or_init(|| {
        let mut validation = Validation::new(Algorithm::RS384);
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

        true
    } else {
        false
    }
}

pub async fn token_middleware<E: Endpoint>(next: E, mut req: Request) -> poem::Result<Response> {
    if is_logged_in(&mut req).await {
        next.call(req).await.map(|res| res.into_response())
    } else {
        Ok(Redirect::see_other("/patrol/login").into_response())
    }
}

pub async fn not_logged_in_middleware<E: Endpoint>(
    next: E,
    mut req: Request,
) -> poem::Result<Response> {
    if is_logged_in(&mut req).await {
        Ok(Redirect::see_other("/patrol/account").into_response())
    } else {
        next.call(req).await.map(|res| res.into_response())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    /// Issuer
    pub iss: String,
    /// Token ID
    pub jti: String,

    /// Username
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
    pub exp: u64,
}

impl Claims {
    pub async fn new(
        redis: &ConnectionManager,
        user: users::Model,
        roles: Vec<String>,
    ) -> anyhow::Result<Self> {
        let month_from_now = Utc::now() + Duration::days(30);

        let jti = crypto::id();
        let key = format!("token:{}:{}", user.username, jti);
        let exp = month_from_now.timestamp().unsigned_abs();

        redis.clone().set_ex(key, "", exp).await?;

        Ok(Self {
            iss: "patrol".to_string(),
            jti,

            sub: user.username.to_string(),
            fnm: user.first_name,
            lnm: user.last_name,
            pic: user.profile_picture,
            rls: roles,

            exp,
        })
    }
}

pub async fn new(claims: Claims) -> anyhow::Result<String> {
    let header = jsonwebtoken::Header {
        kid: Some("main".to_string()),

        ..jsonwebtoken::Header::new(Algorithm::RS384)
    };

    jsonwebtoken::encode(&header, &claims, keys::encoding_key().await?).map_err(|err| anyhow!(err))
}
