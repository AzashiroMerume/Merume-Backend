use chrono::{TimeDelta, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn generate_refresh_jwt_token(user_id: &str, jwt_secret: &str) -> Result<String, String> {
    let iat = Utc::now();
    let month = match TimeDelta::try_days(30) {
        Some(month) => month,
        None => return Err(format!("Failed to calculate time interval")),
    };
    let exp = iat + month;

    let claims = Claims {
        sub: user_id.to_owned(),
        iat: iat.timestamp(),
        exp: exp.timestamp(),
    };

    let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
    match encode(&Header::default(), &claims, &encoding_key) {
        Ok(token) => Ok(token),
        Err(e) => Err(format!("Error generating JWT token: {:?}", e)),
    }
}

pub fn verify_refresh_jwt_token(token: &str, jwt_secret: &str) -> Result<String, ErrorKind> {
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    match decode::<Claims>(token, &decoding_key, &Validation::default()) {
        Ok(decoded) => Ok(decoded.claims.sub),
        Err(e) => Err(e.into_kind()),
    }
}
