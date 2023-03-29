use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn generate_jwt_token(user_id: &str, jwt_secret: &str) -> Result<String, String> {
    let iat = Utc::now();
    let exp = iat + Duration::hours(24);

    let claims = Claims {
        sub: user_id.to_owned(),
        iat: iat.timestamp(),
        exp: exp.timestamp(),
    };

    let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
    match encode(&Header::default(), &claims, &encoding_key) {
        Ok(token) => Ok(token),
        Err(e) => Err(format!("Error generating JWT token: {:?}", e)),
    }
}

pub fn verify_token(token: &str, jwt_secret: &str) -> Result<Claims, String> {
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    match decode::<Claims>(token, &decoding_key, &Validation::default()) {
        Ok(decoded) => Ok(decoded.claims),
        Err(e) => Err(format!("Error verifying JWT token: {:?}", e)),
    }
}
