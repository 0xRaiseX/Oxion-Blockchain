/*
    Файл для связи с основным проектом
    В основной функции main вызывается функция activate(main local)
*/
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashSet;
use crate::tcp_stream::TCPStream;
use crate::tcp_manager::TcpManager;
use crate::message::Message;
use tokio::sync::mpsc::{Receiver, Sender};
use std::collections::HashMap;
use tokio::sync::broadcast;
use crate::buffer::BufferMessage;
use crate::tcp_connect::TCPConnect;

// Активирует модуль TCP соединений
// Требует запуска в отдельном потоке
pub async fn activate(receiver: Receiver<Message>) -> std::io::Result<()> {
    // // Создает буффер для разные потоков
    let buffer_set = Arc::new(Mutex::new(HashSet::new()));
    let buffer_set_clone = Arc::clone(&buffer_set);

    // // Запускает буффер на обновление данных каждые 5 минут
    let mut buffer_message = BufferMessage::new(buffer_set);
    let buffer_stream = tokio::spawn(async move {
        let _ = buffer_message.start().await;
    });

    // let (tx, mut rx) = mpsc::channel(10);

    let main_port = 31313;
    
    // Очередь для TCP Manager для клонирования сообщений на все подключенные узлы 
    let (write_tx, _read_rx) = broadcast::channel::<String>(16);

    let mut tcp_manager = TcpManager::new(buffer_set_clone, receiver, write_tx.clone());
    let _tcp_manager_stream = tokio::spawn(async move {
        let _ = tcp_manager.start_thread().await;
    });
    
    let mut tcp_connect = TCPConnect::new();
    let tcp_connect_stream = tokio::spawn(async move {
        let _ = tcp_connect.connect_peers().await;
    });

    let tcp_stream = TCPStream::new(write_tx.clone());
    let _ = tcp_stream.start_thread().await;

    Ok(())
}
