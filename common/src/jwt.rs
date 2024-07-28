use ::chrono::Duration;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub tracing_id: String,
    pub task_id: i32,
    pub exp: usize,
}

pub struct Jwt {
    secret: String,
}

impl Jwt {
    pub fn new(secret: String) -> Self {
        Jwt { secret }
    }

    pub fn encode(&self, tracing_id: &str, id: i32) -> Result<String, jsonwebtoken::errors::Error> {
        let expiary = chrono::Utc::now() + Duration::minutes(30);
        let claims = Claims {
            tracing_id: tracing_id.to_string(),
            task_id: id,
            exp: expiary.timestamp() as usize,
        };
        let encoding_key = EncodingKey::from_secret(self.secret.as_ref());
        let token = encode(&Header::default(), &claims, &encoding_key)?;
        Ok(token)
    }

    pub fn verify(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_ref());
        let validation = Validation::new(Algorithm::HS256);
        let token_data: TokenData<Claims> = decode(token, &decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn check_jwt() {
        let jwt = Jwt::new("super-secret-token".to_string());
        let token = jwt.encode("1234", 1).unwrap();
        let cliams = jwt.verify(&token).unwrap();
        assert_eq!(cliams.task_id, 1);
        assert_eq!(cliams.tracing_id, "1234");
    }
}
