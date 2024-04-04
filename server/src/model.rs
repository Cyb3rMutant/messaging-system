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
            message_id,
            chat_id,
            sender_id,
            content,
            status
        FROM
            messages
        WHERE
            chat_id IN (SELECT chat_id FROM chats WHERE user_id_1 = ? OR user_id_2 = ?);
        "#,
        id,
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

pub async fn register(
    name: &str,
    password: &str,
    conn: &sqlx::MySqlPool,
) -> Result<(i32, Vec<(i32, i32)>), ()> {
    match query!("SELECT * FROM users WHERE username = ?", &name)
        .fetch_one(conn)
        .await
    {
        Ok(_) => return Err(()),
        _ => (),
    };
    let password_hash = generate_hash(password);

    let id = query!(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        &name,
        &password_hash,
    )
    .execute(conn)
    .await
    .unwrap()
    .last_insert_id();

    query!(
        r#"
        INSERT INTO chats (user_id_1, user_id_2) 
        SELECT DISTINCT ?, user_id 
        FROM users 
        WHERE user_id <> ?
        "#,
        id,
        id
    )
    .execute(conn)
    .await
    .unwrap();

    let friends = query!(
        "SELECT user_id_2, chat_id FROM chats where user_id_1 = ?",
        id
    )
    .fetch_all(conn)
    .await
    .unwrap();

    Ok((
        id as i32,
        friends
            .into_iter()
            .map(|f| (f.user_id_2, f.chat_id))
            .collect(),
    ))
}

pub async fn new_message(message: &Message, conn: &sqlx::MySqlPool) -> i32 {
    query!(
        "INSERT INTO messages (content, sender_id, chat_id, status) VALUES (?, ?, ?, ?)",
        message.get_content(),
        message.sender_id,
        message.chat_id,
        message.status
    )
    .execute(conn)
    .await
    .unwrap()
    .last_insert_id() as i32
}

pub async fn set_seen(chat_id: i32, user_id: i32, conn: &sqlx::MySqlPool) {
    query!(
        "UPDATE messages SET status = 2 WHERE chat_id = ? AND sender_id = ?",
        chat_id,
        user_id
    )
    .execute(conn)
    .await
    .unwrap();
}
pub async fn delete(message_id: i32, conn: &sqlx::MySqlPool) {
    query!(
        "UPDATE messages SET status = 3, content = '' WHERE message_id = ?",
        message_id
    )
    .execute(conn)
    .await
    .unwrap();
}
pub async fn update(message: &Message, conn: &sqlx::MySqlPool) {
    query!(
        "UPDATE messages SET status = 4, content = ? WHERE message_id = ?",
        message.content,
        message.message_id
    )
    .execute(conn)
    .await
    .unwrap();
}

pub async fn clear(conn: &sqlx::MySqlPool) {
    query!("DELETE FROM messages;").execute(conn).await.unwrap();
    query!("ALTER TABLE messages AUTO_INCREMENT = 1;")
        .execute(conn)
        .await
        .unwrap();
    query!("DELETE FROM chats;").execute(conn).await.unwrap();
    query!("ALTER TABLE chats AUTO_INCREMENT = 1;")
        .execute(conn)
        .await
        .unwrap();
    query!("DELETE FROM blocked;").execute(conn).await.unwrap();
    query!("ALTER TABLE blocked AUTO_INCREMENT = 1;")
        .execute(conn)
        .await
        .unwrap();
    query!("DELETE FROM users;").execute(conn).await.unwrap();
    query!("ALTER TABLE users AUTO_INCREMENT = 1;")
        .execute(conn)
        .await
        .unwrap();
}
