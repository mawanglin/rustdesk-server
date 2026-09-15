use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::env;

// 校验登录 token 的共享密钥，与 rustdesk-api 的 JWT key 用同一个来源
// （RUSTDESK_API_JWT_KEY）。两端一致才能通过客户端持有的 token 做身份互通。
pub static SECRET: Lazy<String> =
    Lazy::new(|| env::var("RUSTDESK_API_JWT_KEY").unwrap_or_else(|_| "".to_string()));

// JWT payload：登录用户 id + 过期时间
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: u32,
    pub exp: usize,
}

// generate_token 与 verify_token 供 hbbs 登录校验使用；generate 目前供测试/调试，
// 真实 token 由 rustdesk-api 登录后签发。注意：不要向 stdout 打印 SECRET。
pub fn generate_token(user_id: u32, exp: i64) -> Result<String, String> {
    let claims = Claims {
        user_id,
        exp: (chrono::Utc::now() + chrono::Duration::seconds(exp)).timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_ref()),
    )
    .map_err(|e| e.to_string())
}

pub fn verify_token(token: &str) -> Result<Claims, String> {
    let validation = Validation::new(Algorithm::HS256);
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET.as_ref()),
        &validation,
    );
    match decoded {
        Ok(token_data) => {
            let now = chrono::Utc::now().timestamp() as usize;
            if token_data.claims.exp > now {
                Ok(token_data.claims)
            } else {
                Err("Token status invalid or expired".to_string())
            }
        }
        Err(_) => Err("Invalid token".to_string()),
    }
}