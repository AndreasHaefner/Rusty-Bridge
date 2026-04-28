use uuid::Uuid;
use sqlx::PgPool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AccountType {
    Anonymous,
    Registered, 
}

impl AccountType {
    pub fn from_status(status: &str) -> Self {
        match status {
            "registered" => Self::Registered,
            _ => Self::Anonymous,
        }
    }
}



#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,             
    pub username: String,      
    pub account_type: AccountType,
}



pub struct UserManager {
    db: PgPool,
}

impl UserManager {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
    pub async fn verify_session(&self, user_id: Uuid, token: &str) -> Result<Option<User>, sqlx::Error> {
    
        let result = sqlx::query!(
            r#"
            SELECT u.id, u.username, u.status
            FROM users u
            JOIN sessions s ON u.id = s.user_id
            WHERE s.token = $1 AND s.user_id = $2 AND s.expires_at > NOW()
            "#,
            token,
            user_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(result.map(|row| User {
            id: row.id,
            username: row.username,
            account_type: AccountType::from_status(&row.status),
        }))
    
    }   

    
    pub async fn create_anonymous_user(&self) -> Result<User, sqlx::Error> {
        let rec = sqlx::query!(
            "INSERT INTO users (username, status) VALUES ($1, $2) RETURNING id",
            "Gast", "anonymous"
        )
        .fetch_one(&self.db)
        .await?;

        Ok(User {
            id: rec.id,
            username: "Guest".to_string(),
            account_type: AccountType::Anonymous,
        })
    }
    pub async fn create_session(&self, user_id: Uuid, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO sessions (token, user_id, expires_at) VALUES ($1, $2, NOW() + INTERVAL '7 days')",
        token,
        user_id
    )
    .execute(&self.db)
    .await?;
    Ok(())
}
}