/* 
    Содержит структуры для передачи сообщений между узлами сети.
*/

use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};
use serde_json::Value;

// Тип сообщения для общения узлов
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MessageType {
    Transaction,
    Block,
    Status,
    Connect,
}

// Сообщение для общения узлов
#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub timestamp: u128,
    pub data: Value,
    pub hash: String,
}

// Сообщение для буфера, содержающее только уникальный хеш сообщения и время его создания 
#[derive(Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct BufMessage {
    pub timestamp: u128,
    pub hash: String,
}

impl Message {
    // Создает новый объект сообщения
    pub fn new(message_type: MessageType, content: Value) -> Message {
        let timestamp = Message::current_timestamp();
        let hash = Message::calculate_hash(&message_type, &content, timestamp);
        
        Message {
            message_type,
            timestamp: timestamp,
            data: content,
            hash: hash,
        }
    }

    // Расчет хеша сообщения
    fn calculate_hash(message_type: &MessageType, data: &Value, timestamp: u128) -> String {
        let data_str = serde_json::to_string(message_type).unwrap();
        let message_type_str = serde_json::to_string(message_type).unwrap();
        let input = format!("{}{}{}", message_type_str, data_str, timestamp);
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Получение текущей временной метки
    fn current_timestamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    }

    /// Преобразование сообщения в строку JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Создание сообщения из строки JSON
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap()
    }
}

impl BufMessage {
    // Создает новое сообщение для буфера из обычного сообщения
    pub fn new(message: &Message) -> BufMessage {
        BufMessage {
            timestamp: message.timestamp,
            hash: message.hash.clone(),
        }
    }
}