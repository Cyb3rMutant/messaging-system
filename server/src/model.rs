use password_auth::{generate_hash, verify_password};
use sqlx::query;

pub async fn login(name: &str, password: &str, conn: &sqlx::MySqlPool) -> Result<(), String> {
    let user = match query!("SELECT password_hash FROM users WHERE name = ?", &name)
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

pub async fn register(name: &str, password: &str, conn: &sqlx::MySqlPool) -> Result<(), ()> {
    match query!("SELECT * FROM users WHERE name = ?", &name)
        .fetch_one(conn)
        .await
    {
        Ok(_) => return Err(()),
        _ => (),
    };
    let password_hash = generate_hash(password);

    sqlx::query!(
        "INSERT INTO users (name, password_hash) VALUES (?, ?)",
        &name,
        &password_hash,
    )
    .execute(conn)
    .await
    .unwrap();

    Ok(())
}
