/*
    Принимает входящие подключения
    Позволяет другим узлам подключаться к текущему узлу.
    OxionProtocol 2024. All rights reserved. 
*/

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, broadcast};
use std::sync::Arc;
use log::{error, info, warn};
use crate::message::{Message, BufMessage, MessageType};
use std::collections::{HashSet, HashMap};
use serde_json::Error as SerdeError;
use std::net::SocketAddr;
use tokio::sync::broadcast::Sender;

pub struct TCPStream {
    // Ссылка для создания читателя broadcast очереди.
    writer_link: Sender<String>,
}

impl TCPStream {
    /*
        Создает новый объект TCP Stream.
    */
    pub fn new(writer_link: Sender<String>) -> TCPStream {
        Self {
            writer_link,
        }
    }

    /*
        Функция для запуска в отдельном потоке.
        Запускает TCP Stream для принятия входящий соединений.
        Создает отдельную асинхронную задачу для каждого подключения.
    */ 
    pub async fn start_thread(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("0.0.0.0:31313").await?;
        info!("Listening new connections on port 31313");

        loop {
            // Ожидает новое подключение, как только оно прихожит, то принимает его
            let (mut socket, addr) = listener.accept().await?;
            info!("New connection: {:?}", addr);
            
            // Создаем нового читателя broadcast канала для получения всех сообщений на данное подключение
            let mut read_local = self.writer_link.subscribe();

            tokio::spawn(async move {
                let mut buffer = [0; 1024];
    
                loop {
                    tokio::select! {
                        result = socket.read(&mut buffer) => {
                            match result {
                                Ok(0) => {
                                    break;
                                }
                                Ok(n) => {
                                    let message: Result<Message, SerdeError> = serde_json::from_slice(&buffer[..n]);
                                    println!("Received from {:?}: {}", addr, String::from_utf8_lossy(&buffer[..n]));

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
                                        },
                                        Err(e) => {
                                            error!("Failed to parse message; error = {:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to read data from {:?}: {}", addr, e);
                                    break;
                                }
                            }
                        }
                        result = read_local.recv() => {
                            match result {
                                Ok(msg) => {
                                    if let Err(e) = socket.write_all(msg.as_bytes()).await {
                                        eprintln!("Failed to write data to {:?}: {}", addr, e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to receive broadcast message: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}
