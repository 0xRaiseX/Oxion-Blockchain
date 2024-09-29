/*
    Основной main файл. Выполянет инициализация, подключение и запуск блокчейна.
    Выполняет все основные сценарии. Управляет всем жизненынм циклом блокчейна.
    Поддерживает работу потоков и управлет ими. Содержит основной исполняемый цикл.
*/

mod block;
mod blockchain;
mod pos;
mod transaction;
mod middleware;
mod node;
mod server;
mod consensys;

use pos::PoS;
use std::sync::Arc;
use std::fs;
use log::LevelFilter;
use serde::Deserialize;

use crossbeam::channel;
use std::collections::BinaryHeap;
use tokio::sync::Mutex;
use std::collections::HashSet;
use tokio::sync::mpsc;
use crate::transaction::Mempool;
use crate::blockchain::Blockchain;

#[derive(Deserialize)]
struct Config {
    log_level: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config: Config = {
        let config_str = fs::read_to_string("config.json").expect("Failed to read config file");
        serde_json::from_str(&config_str).expect("Failed to parse config file")
    };

    let (tx, rx) = mpsc::channel(10);

    // Очередь для отправки полученных сообщений через RPC на другие узлы.
    // let (sender_rpc, _receiver_rpc) = channel::unbounded();
    // Запускает TCP Server и TCP Connect. Управляется TCP Manager.
    let _tcp_server = tokio::spawn(async {
        let _ = tcp_module::module::activate(rx).await;
    });

    let log_level = match config.log_level.to_lowercase().as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .filter_module("actix", LevelFilter::Off)
        .filter_module("actix_web", LevelFilter::Off)
        .init();

    // let (sender, _receiver) = channel::unbounded();

    let pos = Arc::new(Mutex::new(PoS::new()));

    let mut mempool = Arc::new(Mutex::new(Mempool::new()));

    let chain_vector = Arc::new(Mutex::new(vec!(Blockchain::create_genesis_block())));

    let mut blockchain = Blockchain::new(Arc::clone(&mempool), Arc::clone(&chain_vector));
    let _blockchain = tokio::spawn(async move {
        let _ = blockchain.start_thread().await;
    });

    {
        let mut pos = pos.lock().await;
        pos.add_participant(String::from("Alice"), 100);
        pos.add_participant(String::from("Bob"), 50);
    }

    let _ = server::start_rpc_server(Arc::clone(&mempool), tx, Arc::clone(&chain_vector)).await;

    Ok(())
}
