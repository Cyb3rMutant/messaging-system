use password_auth::{generate_hash, verify_password};
use sqlx::{query, query_as};

use crate::message::Message;

pub async fn load_users(conn: &sqlx::MySqlPool) -> Vec<(String, String, i32)> {
    query!("SELECT * FROM chats;")
        .fetch_all(conn)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.username_1, r.username_2, r.chat_id))
        .collect()
}

pub async fn load_messages(name: &str, conn: &sqlx::MySqlPool) -> Vec<Message> {
    query_as!(
        Message,
        r#"
        SELECT
            m.sender_username AS sender,
            CASE
                WHEN m.sender_username = c.username_1 THEN c.username_2
                ELSE c.username_1
            END AS receiver,
            m.content
        FROM
            messages m
            JOIN chats c ON m.chat_id = c.chat_id
        WHERE
            m.sender_username = ?;
        "#,
        name
    )
    .fetch_all(conn)
    .await
    .unwrap()
}

pub async fn login(name: &str, password: &str, conn: &sqlx::MySqlPool) -> Result<(), String> {
    let user = match query!("SELECT password_hash FROM users WHERE username = ?", &name)
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
    let (sender, receiver) = message.get_users();
    let chat_id = query!(
        r#"
        SELECT chat_id 
        FROM chats 
        WHERE (? = username_1 AND ? = username_2) 
        OR (? = username_2 AND ? = username_1)
        "#,
        sender,
        receiver,
        sender,
        receiver
    )
    .fetch_one(conn)
    .await
    .unwrap()
    .chat_id;

    query!(
        "INSERT INTO messages (content, sender_username, chat_id) VALUES (?, ?, ?)",
        message.get_content(),
        sender,
        chat_id
    )
    .execute(conn)
    .await
    .unwrap();

    Ok(())
}
