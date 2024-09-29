/*
    Подключается к узлам в сети
    Добавляет активное соединение в переменную connections
*/
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use log::{error, info};
use crate::message::Message;
use std::collections::HashSet;
use std::env;
use std::net::SocketAddr;
use std::collections::HashMap;
use serde_json::Error as SerdeError;
use crate::message::MessageType;

pub struct TCPConnect {}

impl TCPConnect {
    pub fn new() -> TCPConnect {
        Self {}
    }

    pub async fn connect_peers(&self) {
        // Ограничить подключение к самому себе
        match env::var("IS_MAIN") {
            Ok(value) => 
            {   
                println!("Значение переменной окружения: {}", value);
                if value == "MAIN" {
                      return;
                } 
            },
            Err(e) => println!("Не удалось прочитать переменную окружения {}", e),
        }

        // Подключаемся к ноде N (становимся активным участником сети)
        let mut stream = match TcpStream::connect("main_node:31313").await {
            Ok(stream) => {
                println!("Successfully connected to server");
                stream
            },
            Err(e) => {
                println!("Failed to connect to TCP server on 31313 {}", e);
                return;
            },
        };

        loop {
            let mut buffer = vec![0; 1024];

            match stream.read(&mut buffer).await {
                Ok(n) if n == 0 => {
                    return;
                },
                Ok(n) => {
                    let message: Result<Message, SerdeError> = serde_json::from_slice(&buffer[..n]);

                    match message {
                        Ok(message) => {
                            match message.message_type {
                                MessageType::Connect => {
                                    info!("Получено сообщения с запросом на подключение");
                                    // Если подключений > 3, то перекинуть основное подключение к ним, т.к. это загрузочный узел (использовать HashSet) 
                                },
                                MessageType::Transaction => {
                                    info!("Получено сообщение с транзакцией");
                                },
                                MessageType::Block => {
                                    info!("Получено сообщение с блоком");
                                },
                                MessageType::Status => {
                                    info!("Получено сообщение со статусом");
                                    continue;
                                },
                            } 

                            let response = Message::new(MessageType::Status, serde_json::json!({"status": "ok"}));
                            let response_data = serde_json::to_vec(&response).unwrap();
                            if let Err(e) = stream.write_all(&response_data).await {
                                eprintln!("Failed to send response; error = {:?}", e);
                            }
                        },
                        Err(e) => {
                            error!("Failed to parse message; error = {:?}", e);
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to read from socket; err = {:?}", e);
                }
            }
        }
    }
}