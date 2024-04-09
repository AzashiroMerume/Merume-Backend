use jsonwebtoken::{DecodingKey, EncodingKey};

#[derive(Clone)]
pub struct FirebaseConfig {
    pub token_encoding_key: EncodingKey,
    pub token_decoding_key: DecodingKey,
    pub service_account: String,
}
