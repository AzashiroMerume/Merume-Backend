use chrono::Utc;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
    pub uid: String,
}

pub fn generate_access_jwt_token(
    firebase_user_id: &str,
    encoding_key: EncodingKey,
    service_account_email: String,
) -> Result<String, String> {
    let iat = Utc::now().timestamp();
    let exp = iat + 3600;

    let claims = Claims {
        iss: service_account_email.to_owned(),
        sub: service_account_email.to_owned(),
        aud: "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit".to_owned(),
        iat,
        exp,
        uid: firebase_user_id.to_owned(),
    };

    match encode::<Claims>(&Header::new(Algorithm::RS256), &claims, &encoding_key) {
        Ok(token) => Ok(token),
        Err(err) => Err(format!("Error generating JWT token: {:?}", err)),
    }
}

pub fn verify_access_jwt_token(
    token: &str,
    decoding_key: DecodingKey,
) -> Result<String, ErrorKind> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[
        "https://identitytoolkit.googleapis.com/google.identity.identitytoolkit.v1.IdentityToolkit",
    ]);
    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(decoded) => Ok(decoded.claims.uid),
        Err(err) => Err(err.into_kind()),
    }
}
