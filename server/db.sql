CREATE DATABASE IF NOT EXISTS `messaging`;
USE `messaging`;

-- Create the 'users' table
CREATE TABLE users (
    user_id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL
);

-- Create the 'chats' table
CREATE TABLE chats (
    chat_id INT AUTO_INCREMENT PRIMARY KEY,
    user_id_1 INT NOT NULL,
    user_id_2 INT NOT NULL,
    A INT DEFAULT NULL,
    B INT DEFAULT NULL,
    FOREIGN KEY (user_id_1) REFERENCES users(user_id),
    FOREIGN KEY (user_id_2) REFERENCES users(user_id),
    CHECK (user_id_1 <> user_id_2) -- Ensure users aren't chatting with themselves
);

-- Create the 'messages' table
CREATE TABLE messages (
    message_id INT AUTO_INCREMENT PRIMARY KEY,
    chat_id INT NOT NULL,
    sender_id INT NOT NULL,
    content TEXT NOT NULL,
    status INT NOT NULL DEFAULT 1, -- Status values (1 = sent, 2 = seen, 3 = deleted, 4 = updated)
    FOREIGN KEY (chat_id) REFERENCES chats(chat_id),
    FOREIGN KEY (sender_id) REFERENCES users(user_id)
);

-- Create the 'blocked' table
CREATE TABLE blocked (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    blocked_user_id INT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (blocked_user_id) REFERENCES users(user_id),
    CHECK (user_id <> blocked_user_id) -- Ensure users aren't blocking themselves
);

-- Optional: Create indexes to optimize certain queries
CREATE INDEX idx_user_blocked ON blocked (user_id, blocked_user_id);
CREATE INDEX idx_chat_participants ON chats (user_id_1, user_id_2);
CREATE INDEX idx_message_status ON messages (status);
