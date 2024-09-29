use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use log::{error, info, warn};
use std::net::SocketAddr;
use std::collections::HashMap;
use crate::buffer::BufferMessage;
use std::collections::HashSet;
use crate::tcp_stream::TCPStream;
use crate::message::{Message, BufMessage, MessageType};
use serde_json::Error as SerdeError;
use tokio::time::{self, Duration};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::Receiver as ReceiverMPSC;
use serde_json::to_string;

pub struct TcpManager {
    // Получает полученные от RPC сервера сообщения, отправленные напрямую на данный узел
    pub receiver_rpc: ReceiverMPSC<Message>,

    // Буффер всех полученных сообщений. Очищается автоматически.
    pub buffer: Arc<Mutex<HashSet<BufMessage>>>,

    // Broadcast очередь. Читают все входящие подключения
    pub sender: Sender<String>
}

impl TcpManager {
    // Создает новый объект TCP Manager
    pub fn new(buffer: Arc<Mutex<HashSet<BufMessage>>>, receiver_rpc: ReceiverMPSC<Message>, sender: Sender<String>) -> TcpManager {
        TcpManager {
            receiver_rpc,
            buffer,
            sender,
        }
    }

    // Запускает TCP Manager. Требует запуска в отдельном потоке
    pub async fn start_thread(&mut self) {
        info!("TCP Manager started.");

        // let mut failed_connections: Vec<SocketAddr> = Vec::new();

        loop {
            // Сообщения, полученные от RPC Server (local) -> Ретрансляция
            while let Some(message) = self.receiver_rpc.recv().await {
                match to_string(&message) {
                    Ok(json_str) => {

                        if let Err(e) = self.sender.send(json_str) {
                            eprintln!("Failed to send message: {}", e);
                            break;
                        }
                    }
                    Err(e) => eprintln!("Ошибка преобразования в строку: {}", e),
                }
            }
            // Сообщения, полученные от входящих соединений (external) -> Обработка
            
        }
    }
}
    
