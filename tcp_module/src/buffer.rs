/*
    BufferMessage используется в качестве буфера сообщений, полученных от других узлов
    Хранит сообщения 5 мин до его времени истечения, далее буфер автоматически очищается
*/
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::sync::Mutex;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::message::BufMessage;

pub struct BufferMessage {
    buffer: Arc<Mutex<HashSet<BufMessage>>>,
}

impl BufferMessage {
    // Создает новый буфер
    pub fn new(buffer: Arc<Mutex<HashSet<BufMessage>>>) -> BufferMessage {
        BufferMessage {
            buffer,
        }
    }
    /* 
        Функция, которая должна быть запущена в отдельном потоке. 
        Проверяет буффер каждые 5 секунд на наличие сообщений, у которых истек срок дейсвтия
        Удаляет сообщение, если оно находится в буфере больше, чем 5 минут
    */
    pub async fn start(&mut self) {
        const FIVE_MINUTES_MILLIS: u128 = 5 * 60 * 1000; 
        loop {
            sleep(Duration::from_secs(5)).await;

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();
                
            let mut buffer_guard = self.buffer.lock().await;
            buffer_guard.retain(|msg| timestamp - msg.timestamp <= FIVE_MINUTES_MILLIS);
        }
    }
}