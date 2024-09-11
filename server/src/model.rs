use password_auth::{generate_hash, verify_password};
use sqlx::{query, query_as, MySql, Pool};

use crate::message::Message;

#[derive(Debug)]
pub struct Model {
    pool: Pool<MySql>,
}

impl Model {
    pub async fn new(db: &str) -> Self {
        let pool = sqlx::mysql::MySqlPool::connect(format!("mysql://root:p@db:3306/{db}").as_str())
            .await
            .unwrap();
        Self { pool }
    }
    pub async fn load_lonely(&self) -> Vec<(i32, String)> {
        query!(
            r#"
            SELECT
                u.*
            FROM
                users u
              LEFT JOIN chats c ON u.user_id = c.user_id_1
              OR u.user_id = c.user_id_2
            WHERE
                c.chat_id IS NULL;
            "#
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.user_id, r.username))
        .collect()
    }

    pub async fn load_chats(&self) -> Vec<(i32, String, i32, String, i32)> {
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
        .fetch_all(&self.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.id_1, r.name_1, r.id_2, r.name_2, r.chat_id))
        .collect()
    }

    pub async fn load_messages(&self, id: i32) -> Vec<Message> {
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
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }
    pub async fn chats(&self, id: i32) -> Vec<i32> {
        query!(
            r#"
            SELECT
                chat_id
            FROM
                chats c
            WHERE
                user_id_1 = ?
                OR user_id_2 = ?;
            "#,
            id,
            id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|c| (c.chat_id))
        .collect()
    }

    pub async fn login(&self, name: &str, password: &str) -> Result<i32, String> {
        let user = match query!(
            "SELECT user_id, password_hash FROM users WHERE username = ?",
            &name
        )
        .fetch_one(&self.pool)
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
    pub async fn all(&self, id: i32) -> String {
        query!(
            r#"
            SELECT
              u.user_id,
              u.username
            FROM
              users u
              LEFT JOIN chats c ON (
                u.user_id = c.user_id_1
                OR u.user_id = c.user_id_2
              )
              AND (
                c.user_id_1 = ?
                OR c.user_id_2 = ?
              )
              LEFT JOIN blocked b ON (
                u.user_id = b.user_id
                OR u.user_id = b.blocked_user_id
              )
              AND (
                b.user_id = ?
                OR b.blocked_user_id = ?
              )
            WHERE
              c.chat_id IS NULL
              AND b.id IS NULL
              AND u.user_id <> ?;
            "#,
            id,
            id,
            id,
            id,
            id
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
        .iter()
        .map(|item| format!("{};{}", item.user_id, item.username))
        .collect::<Vec<String>>()
        .join(";")
    }
    pub async fn blocked(&self, id: i32) -> String {
        query!(
            r#"
            SELECT
              u.*
            FROM
              users u
              JOIN blocked b ON u.user_id = b.blocked_user_id
            WHERE
              b.user_id = ?;
            "#,
            id,
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
        .iter()
        .map(|item| format!("{};{}", item.user_id, item.username))
        .collect::<Vec<String>>()
        .join(";")
    }

    pub async fn register(&self, name: &str, password: &str) -> Result<i32, ()> {
        match query!("SELECT * FROM users WHERE username = ?", &name)
            .fetch_one(&self.pool)
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
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id();
        Ok(id as i32)
    }
    pub async fn connect(&self, id: i32, other: i32) -> i32 {
        let chat_id = query!(
            "INSERT INTO chats (user_id_1, user_id_2) VALUES (?, ?)",
            id,
            other
        )
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i32;

        chat_id
    }
    pub async fn block(&self, id: i32, other: i32) {
        query!(
            "INSERT INTO blocked (user_id, blocked_user_id) VALUES (?, ?)",
            id,
            other
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn unblock(&self, id: i32, other: i32) {
        query!(
            "DELETE FROM blocked WHERE user_id = ? AND blocked_user_id = ?;",
            id,
            other
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }

    pub async fn new_message(&self, message: &Message) -> i32 {
        query!(
            "INSERT INTO messages (content, sender_id, chat_id, status) VALUES (?, ?, ?, ?)",
            message.get_content(),
            message.sender_id,
            message.chat_id,
            message.status
        )
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i32
    }

    pub async fn set_seen(&self, chat_id: i32, user_id: i32) {
        query!(
            "UPDATE messages SET status = 2 WHERE chat_id = ? AND sender_id = ?",
            chat_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn delete(&self, message_id: i32) {
        query!(
            "UPDATE messages SET status = 3, content = '' WHERE message_id = ?",
            message_id
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
    pub async fn update(&self, message: &Message) {
        query!(
            "UPDATE messages SET status = 4, content = ? WHERE message_id = ?",
            message.content,
            message.message_id
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }

    pub async fn clear(&self) {
        query!("DELETE FROM messages;")
            .execute(&self.pool)
            .await
            .unwrap();
        query!("ALTER TABLE messages AUTO_INCREMENT = 1;")
            .execute(&self.pool)
            .await
            .unwrap();
        query!("DELETE FROM chats;")
            .execute(&self.pool)
            .await
            .unwrap();
        query!("ALTER TABLE chats AUTO_INCREMENT = 1;")
            .execute(&self.pool)
            .await
            .unwrap();
        query!("DELETE FROM blocked;")
            .execute(&self.pool)
            .await
            .unwrap();
        query!("ALTER TABLE blocked AUTO_INCREMENT = 1;")
            .execute(&self.pool)
            .await
            .unwrap();
        query!("DELETE FROM users;")
            .execute(&self.pool)
            .await
            .unwrap();
        query!("ALTER TABLE users AUTO_INCREMENT = 1;")
            .execute(&self.pool)
            .await
            .unwrap();
    }
}
