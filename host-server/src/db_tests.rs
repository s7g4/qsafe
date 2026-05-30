#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::env;
    use dotenvy::dotenv;

    async fn setup_db() -> Database {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/qsafe_test".to_string());
        Database::new(&database_url, 5).await.expect("Failed to connect to test DB")
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let db = setup_db().await;
        
        // Use a unique username for each test run
        let username = format!("testuser_{}", Uuid::new_v4().to_string().replace("-", "")[0..8]);
        let email = format!("{}@example.com", username);
        let password_hash = "fake_hash_123";
        let pub_key = vec![1, 2, 3, 4];

        // Test create
        let user = db.create_user(&username, &email, password_hash, &pub_key).await.expect("Failed to create user");
        assert_eq!(user.username, username);
        assert_eq!(user.email, email);
        assert_eq!(user.password_hash, password_hash);

        // Test get
        let retrieved = db.get_user_by_username(&username).await.expect("DB error").expect("User not found");
        assert_eq!(retrieved.id, user.id);
    }
}
