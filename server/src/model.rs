use password_auth::{generate_hash, verify_password};
use sqlx::{query, query_as};

use crate::message::Message;

pub async fn load_users(conn: &sqlx::MySqlPool) -> Vec<(i32, String, i32, String, i32)> {
    query!(
        r#"
        SELECT
            c.user_id_1 AS id_1,
            u1.username AS name_1,
            c.user_id_2 AS id_2,
            u2.username AS name_2,
            c.chat_id
        FROM
            chats c
        JOIN
            users u1 ON c.user_id_1 = u1.user_id
        JOIN
            users u2 ON c.user_id_2 = u2.user_id;
        "#
    )
    .fetch_all(conn)
    .await
    .unwrap()
    .into_iter()
    .map(|r| (r.id_1, r.name_1, r.id_2, r.name_2, r.chat_id))
    .collect()
}

pub async fn load_messages(id: i32, conn: &sqlx::MySqlPool) -> Vec<Message> {
    query_as!(
        Message,
        r#"
        SELECT
            chat_id,
            sender_id,
            content
        FROM
            messages
        WHERE
            sender_id = ?;
        "#,
        id
    )
    .fetch_all(conn)
    .await
    .unwrap()
}

pub async fn login(name: &str, password: &str, conn: &sqlx::MySqlPool) -> Result<i32, String> {
    let user = match query!(
        "SELECT user_id, password_hash FROM users WHERE username = ?",
        &name
    )
    .fetch_one(conn)
    .await
    {
        Err(_) => return Err("user does not exist".to_owned()),
        Ok(r) => r,
    };

    if let Ok(_) = verify_password(&password, &user.password_hash) {
        Ok(user.user_id)
    } else {
        Err("invalid password".to_owned())
    }
}

pub async fn register(name: &str, password: &str, conn: &sqlx::MySqlPool) -> Result<(), ()> {
    match query!("SELECT * FROM users WHERE username = ?", &name)
        .fetch_one(conn)
        .await
    {
        Ok(_) => return Err(()),
        _ => (),
    };
    let password_hash = generate_hash(password);

    query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        &name,
        &password_hash,
    )
    .execute(conn)
    .await
    .unwrap();

    query!(
        r#"
        INSERT INTO chats (username_1, username_2) 
        SELECT DISTINCT ?, username 
        FROM users 
        WHERE username <> ?
        "#,
        name,
        name
    )
    .execute(conn)
    .await
    .unwrap();

    Ok(())
}

pub async fn new_message(message: &Message, conn: &sqlx::MySqlPool) -> Result<(), ()> {
    query!(
        "INSERT INTO messages (content, sender_id, chat_id) VALUES (?, ?, ?)",
        message.get_content(),
        message.sender_id,
        message.chat_id
    )
    .execute(conn)
    .await
    .unwrap();

    Ok(())
}
