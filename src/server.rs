// Основной модуль серверной части. Выполняет работу с клиентом и запросами, отправленные на текущий узел
// 03.08.2024 OXI Ecosystem  All Right Reserved

use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use actix_web::web::Data;
// use crossbeam::channel::Sender;

use crate::transaction::Transaction;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::{Value, to_value};
use log::info;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use tcp_module::message::Message;
use tcp_module::message::MessageType;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use base64::{encode, decode};
use crate::transaction::Mempool;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::block::Block;

#[derive(Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Vec<Value>>,
    id: u64,
}

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

pub struct RPCServer {
    mempool: Arc<Mutex<Mempool>>,
    send_to_nodes_link: Sender<Message>,
    chain: Arc<Mutex<Vec<Block>>>
}

impl RPCServer {
    pub fn new(mempool: Arc<Mutex<Mempool>>, send_to_nodes_link: Sender<Message>, chain: Arc<Mutex<Vec<Block>>>) -> RPCServer {
        RPCServer {
            mempool,
            send_to_nodes_link,
            chain
        }
    }

    async fn add_transaction(&self, params: Option<Vec<Value>>) -> RpcResponse {
        // Формируем params для удобной работы с параметрами
        let array: Vec<Value> = match params {
            Some(arr) => arr,
            None => {
                return RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: Some(json!("Params required")),
                    error: None,
                }
            }
        };

        let public_key_bytes = base64::decode(&array.get(0).and_then(Value::as_str).unwrap_or_default().to_string()).expect("Invalid public key encoding");
        let signature_bytes = base64::decode(&array.get(5).and_then(Value::as_str).unwrap_or_default().to_string()).expect("Invalid signature encoding");
        
        let addr = array.get(0).and_then(Value::as_str).unwrap_or_default().to_string();
        let to = array.get(1).and_then(Value::as_str).unwrap_or_default().to_string();
        let amount = array.get(2).and_then(Value::as_u64).unwrap_or_default() as u128;
        let timestamp = array.get(3).and_then(Value::as_u64).unwrap_or_default() as u128;
        let fee = array.get(4).and_then(Value::as_u64).unwrap_or_default() as u64;

        let message_from = format!("{}{}{}{}{}", addr, to, amount, timestamp, fee);
        let message_bytes = message_from.as_bytes();

        let public_key = PublicKey::from_bytes(&public_key_bytes).expect("Invalid public key");
        let signature = Signature::from_bytes(&signature_bytes).expect("Invalid signature");

        let is = public_key.verify(message_bytes, &signature).is_ok();

        if is {
            let transaction = Transaction::new(addr, to, amount, fee, timestamp);

            let mut mempool_lock = self.mempool.lock().await;

            match mempool_lock.add_transaction(transaction.clone()) {
                true => {
                    let message_to_nodes = Message::new(MessageType::Transaction, to_value(&transaction).unwrap());
                    self.send_to_nodes_link.send(message_to_nodes).await;
                    
                    return RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: Some(json!("Transaction added")),
                        error: None,
                    }
                }
                false => {
                    return RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        result: Some(json!("Транзакция уже добавлена.")),
                        error: None,
                    }
                }
            }
        } else {
            return RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: Some(json!("Signature failed.")),
                error: None,
            }
        }
    }

    async fn get_block(&self, params: Option<Vec<Value>>) -> RpcResponse {
        let array: Vec<Value> = match params {
            Some(arr) => arr,
            None => {
                return RpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: Some(json!("Params required")),
                    error: None,
                }
            }
        };

        let index_block = array.get(0).and_then(Value::as_u64).unwrap_or_default() as usize;

        let chain = self.chain.lock().await;

        if let Some(value) = chain.get(index_block) {
            return RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: Some(json!(value)),
                error: None,
            };
        } else {
            return RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: Some(RpcError { code: 0, message: "Индекс не найден".to_string() }),
            }
        }
    }

    async fn say_hello_handler(&self, _params: Option<Vec<Value>>, mempool: Arc<Mutex<BinaryHeap<Transaction>>>) -> RpcResponse {
        let guard = mempool.lock().await;
        RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(json!(guard.iter().cloned().collect::<Vec<Transaction>>())),
            error: None,
        }
    }

    async fn some_other_handler(&self, _params: Option<Vec<Value>>) -> RpcResponse {
        RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(json!("This is another method!")),
            error: None,
        }
    }
}

async fn rpc_handler(req_body: String, server: Data<Arc<Mutex<RPCServer>>>) -> impl Responder {
    let request: RpcRequest = match serde_json::from_str(&req_body) {
        Ok(request) => request,
        Err(e) => {
            let response = RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: None,
                result: None,
                error: Some(RpcError {
                    code: 0,
                    message: e.to_string(),
                }),
            };
            return HttpResponse::BadRequest().json(response);
        }
    };

    let server = server.lock().await;

    let mut response = match request.method.as_str() {
        // Main methods
        "sendTransaction" => server.add_transaction(request.params).await,
        "getBlock" => server.get_block(request.params).await,
        // Test methods
        // "say_hello" => server.say_hello_handler(request.params, mempool_arc).await,
        // "some_other_method" => server.some_other_handler(request.params).await,
        _ => RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: Some(RpcError {
                code: 0,
                message: "Method not found".to_string(),
            }),
        }, 
    };
    response.id = Some(request.id);
    HttpResponse::Ok().json(response)
}


pub async fn start_rpc_server(mempool: Arc<Mutex<Mempool>>, send_to_nodes_link: Sender<Message>, chain: Arc<Mutex<Vec<Block>>>)-> std::io::Result<()> {
    info!("RPC Server Starting on 8080");

    let server = Arc::new(Mutex::new(RPCServer::new(mempool, send_to_nodes_link, chain)));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(Arc::clone(&server)))
            .route("/rpc", web::post().to(rpc_handler))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}



    // fn build_responce(result: Option<serde_json::Value>, error: Option<RpcError>) 

    // async fn mine_transactions() -> RpcResponse {
    //     // let validator_address = {
    //     //     let pos = pos.lock().unwrap();
    //     //     pos.select_validator().map(|v| v.address.clone())
    //     // };

    //     let message;

    //     if let Some(validator_address) = validator_address {
    //         // let mut blockchain = blockchain.lock().unwrap();
    //         // blockchain.mine_pending_transactions(validator_address);
    //         message = "Block mined";
    //     } else {
    //         message = "No validators available";
    //     }

    //     RpcResponse {
    //         jsonrpc: "2.0".to_string(),
    //         id: None,
    //         result: Some(message.into()),
    //         error: None,
    //     }
    // }

    // async fn get_blockchain() -> RpcResponse {
    //     RpcResponse {
    //         jsonrpc: "2.0".to_string(),
    //         id: None,
    //         result: Some(json!(blockchain_copy)),
    //         error: None,
    //     }
    // }

    // 03.08.2024 OXI Ecosystem  All Right Reserved

/*
    Основной модуль серверной части. Выполняет работу с клиентом: 
    1. Принимает запросы RPC 
    2. Проверяет валидность запроса
    3. Проверяет подпись, данные, структуру запроса 
    4. Отправляет пользователю ответ о состоянии его запроса 

    Может читать данные из блокчейна и отправлять транзакции в очередь мемпула.
*/