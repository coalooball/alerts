use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_token: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<User>,
    pub session_token: Option<String>,
}

pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn hash_password(password: &str) -> Result<String> {
        hash(password, DEFAULT_COST).map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        // Debug logging
        log::debug!("Verifying password '{}' against hash '{}'", password, &hash[..std::cmp::min(hash.len(), 30)]);
        
        let result = verify(password, hash).map_err(|e| anyhow::anyhow!("Failed to verify password: {}", e))?;
        log::debug!("Verification result: {}", result);
        Ok(result)
    }

    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<Option<User>> {
        #[derive(sqlx::FromRow)]
        struct UserRow {
            id: Uuid,
            username: String,
            email: String,
            password_hash: String,
            is_active: bool,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let user = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, email, password_hash, is_active, created_at, updated_at 
             FROM users WHERE username = $1 AND is_active = true"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(user_row) = user {
            log::info!("Found user: {}, hash starts with: {}", user_row.username, &user_row.password_hash[..20]);
            if Self::verify_password(password, &user_row.password_hash)? {
                log::info!("Password verification successful for user: {}", user_row.username);
                return Ok(Some(User {
                    id: user_row.id,
                    username: user_row.username,
                    email: user_row.email,
                    is_active: user_row.is_active,
                    created_at: user_row.created_at,
                    updated_at: user_row.updated_at,
                }));
            } else {
                log::warn!("Password verification failed for user: {}", user_row.username);
            }
        } else {
            log::warn!("No user found with username: {}", username);
        }

        Ok(None)
    }

    pub async fn create_session(&self, user_id: Uuid) -> Result<UserSession> {
        let session_token = generate_session_token();
        let expires_at = Utc::now() + Duration::hours(24); // 24 hours

        #[derive(sqlx::FromRow)]
        struct SessionRow {
            id: Uuid,
            user_id: Uuid,
            session_token: String,
            expires_at: chrono::DateTime<chrono::Utc>,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let session = sqlx::query_as::<_, SessionRow>(
            "INSERT INTO user_sessions (user_id, session_token, expires_at) 
             VALUES ($1, $2, $3) 
             RETURNING id, user_id, session_token, expires_at, created_at"
        )
        .bind(user_id)
        .bind(&session_token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(UserSession {
            id: session.id,
            user_id: session.user_id,
            session_token: session.session_token,
            expires_at: session.expires_at,
            created_at: session.created_at,
        })
    }

    pub async fn validate_session(&self, session_token: &str) -> Result<Option<User>> {
        #[derive(sqlx::FromRow)]
        struct SessionUserRow {
            user_id: Uuid,
            expires_at: chrono::DateTime<chrono::Utc>,
            id: Uuid,
            username: String,
            email: String,
            is_active: bool,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let session = sqlx::query_as::<_, SessionUserRow>(
            "SELECT us.user_id, us.expires_at, u.id, u.username, u.email, u.is_active, u.created_at, u.updated_at
             FROM user_sessions us
             JOIN users u ON us.user_id = u.id
             WHERE us.session_token = $1 AND us.expires_at > NOW() AND u.is_active = true"
        )
        .bind(session_token)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(session) = session {
            return Ok(Some(User {
                id: session.id,
                username: session.username,
                email: session.email,
                is_active: session.is_active,
                created_at: session.created_at,
                updated_at: session.updated_at,
            }));
        }

        Ok(None)
    }

    pub async fn invalidate_session(&self, session_token: &str) -> Result<()> {
        sqlx::query("DELETE FROM user_sessions WHERE session_token = $1")
            .bind(session_token)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM user_sessions WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}

fn generate_session_token() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    Uuid::new_v4().hash(&mut hasher);
    Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
    
    format!("{:x}", hasher.finish())
}