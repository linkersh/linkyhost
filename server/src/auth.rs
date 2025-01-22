use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: i32,
    pub exp: i64,
}

pub struct Auther {
    enc_key: EncodingKey,
    dec_key: DecodingKey,
    header: Header,
    valid: Validation,
}

impl Auther {
    pub fn new(secret: &str) -> anyhow::Result<Auther> {
        let enc_key = EncodingKey::from_base64_secret(secret)?;
        let dec_key = DecodingKey::from_base64_secret(secret)?;
        let header = Header::new(Algorithm::HS512);
        let mut valid = Validation::new(Algorithm::HS512);
        valid.validate_exp = true;

        Ok(Auther {
            enc_key,
            dec_key,
            header,
            valid,
        })
    }

    pub async fn hash_password(&self, pass: String) -> anyhow::Result<String> {
        let hash = tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
            let argon = Argon2::default();
            let salt = SaltString::generate(&mut OsRng);
            let hash = match argon.hash_password(pass.as_bytes(), &salt) {
                Ok(hash) => hash.to_string(),
                Err(_) => return Err(anyhow::Error::msg("failed to hash password")),
            };

            Ok(hash)
        })
        .await??;
        Ok(hash)
    }

    pub async fn verify_password(&self, pass: String, hash: String) -> anyhow::Result<bool> {
        let is_valid = tokio::task::spawn_blocking(move || -> anyhow::Result<bool> {
            let argon = Argon2::default();
            let password_hash = match PasswordHash::new(&hash) {
                Ok(v) => v,
                Err(_) => return Err(anyhow::Error::msg("failed to parse password")),
            };
            let valid_res = argon.verify_password(pass.as_bytes(), &password_hash);
            Ok(valid_res.is_ok())
        })
        .await??;
        Ok(is_valid)
    }

    pub fn sign_token(&self, user_id: i32) -> anyhow::Result<String> {
        let claims = UserClaims {
            exp: Utc::now().timestamp_millis() + 1000 * 60 * 60 * 24,
            sub: user_id,
        };

        let token = jsonwebtoken::encode(&self.header, &claims, &self.enc_key)?;
        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> anyhow::Result<UserClaims> {
        let claims: TokenData<UserClaims> =
            jsonwebtoken::decode(token, &self.dec_key, &self.valid)?;
        Ok(claims.claims)
    }
}
