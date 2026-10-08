use crate::models::User;
use sqlx::SqlitePool;

pub async fn create_user(pool: &SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    // TODO: Implement user creation
    Ok(())
}

pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>, sqlx::Error> {
    // TODO: Implement get user by email
    Ok(None)
}

pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, sqlx::Error> {
    // TODO: Implement get user by id
    Ok(None)
}

pub async fn update_user(pool: &SqlitePool, user: &User) -> Result<(), sqlx::Error> {
    // TODO: Implement update user
    Ok(())
}
