use password_auth::verify_password;

pub async fn login(name: &str, password: &str, conn: &sqlx::MySqlPool) -> Result<(), String> {
    let user = match sqlx::query!("SELECT password_hash FROM users WHERE name = ?", &name)
        .fetch_one(conn)
        .await
    {
        Err(_) => return Err("user does not exist".to_owned()),
        Ok(r) => r,
    };

    if let Ok(_) = verify_password(&password, &user.password_hash) {
        Ok(())
    } else {
        Err("invalid password".to_owned())
    }
}
