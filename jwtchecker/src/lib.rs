use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, Validation, Algorithm, DecodingKey};
use custom_error::custom_error;


custom_error!{pub JWTError
    InvalidToken    = "The token is invalid"
}

#[derive(Clone)]
pub struct JWTChecker {
    validation: Validation,
    key: DecodingKey
}

impl JWTChecker {
    pub fn new(rsa_pub: &str) -> Self {
        let key = jsonwebtoken::DecodingKey::from_rsa_pem(rsa_pub.as_bytes()).unwrap();
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;
        validation.validate_exp = false;
        Self { key, validation }
    }
    pub fn decode_header(&self, header: &str) -> Result<String, JWTError> {
        let token = header.replace("Bearer ", "");
        self.decode(&token)
    }
    pub fn decode(&self, token: &str) -> Result<String, JWTError> {
        match decode::<Claims>(token, &self.key, &self.validation) {
            Ok(token_message) => { Ok(token_message.claims.preferred_username) }
            Err(_) => { Err(JWTError::InvalidToken) }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
   sub: String,
   preferred_username: String
}
