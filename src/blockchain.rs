// src/blockchain.rs
use crate::block::Block;
use crate::transaction::Transaction;
use serde::Deserialize;
use serde::Serialize;
use crate::transaction::Mempool;
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use std::time::{SystemTime, UNIX_EPOCH};
use log::info;

#[derive(Clone,)]
pub struct Blockchain {
    pub chain: Arc<Mutex<Vec<Block>>>,
    pub pending_transactions: Vec<Transaction>,
    mempool: Arc<Mutex<Mempool>>,
}

impl Blockchain {
    pub fn new(mempool: Arc<Mutex<Mempool>>, chain: Arc<Mutex<Vec<Block>>>) -> Self {
        Blockchain {
            chain: chain,
            pending_transactions: Vec::new(),
            mempool: mempool,
        }
    }
    // Основной цикл блокчейна
    pub async fn start_thread(&mut self) {
        info!("Blockchain started.");
       
        let mut time_block_loop = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();

        loop {
            let transaction; 
            
            {
                let mut mempool_lock = self.mempool.lock().await;
                transaction = mempool_lock.get_highest_fee_transaction();
            }
        
            if let Some(tx) = transaction {
                self.add_transaction(tx);
                info!("Pending Transactions size: {}", self.pending_transactions.len());
            } else {
                sleep(Duration::from_secs(1)).await;
            }

            if self.pending_transactions.len() > 0 {
                let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                if current_time - time_block_loop > 20000 {
                    self.mine_pending_transactions("0x000000000000000000000".to_string()).await;
                    time_block_loop = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                }
            }
            
        }
    }

    pub fn create_genesis_block() -> Block {
        Block::new(0, String::from("0"), Vec::new())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub async fn mine_pending_transactions(&mut self, validator_address: String) {
        self.add_transaction(Transaction::new("network".to_string(), validator_address, 50, 0, 0));
        //// timestamp нужно указать
        
        let mut chain = self.chain.lock().await;

        let previous_block = chain.last().expect("Blockchain should have at least one block");
        let transactions = self.pending_transactions.clone();
        let new_block = Block::new(previous_block.index + 1, previous_block.hash.clone(), transactions);

        chain.push(new_block);
        self.pending_transactions.clear();
        info!("Block number {} created successfully.", chain.len());
    }

    pub async fn is_valid(&self) -> bool {
        let chain = self.chain.lock().await;
        
        for i in 1..chain.len() {
            let previous_block = &chain[i - 1];
            let current_block = &chain[i];
            
            if current_block.previous_hash != previous_block.hash {
                return false;
            }

            if current_block.hash != Block::calculate_hash(
                current_block.index,
                current_block.timestamp,
                &current_block.previous_hash,
                current_block.nonce,
                &current_block.transactions,
            ) {
                return false;
            }
        }
        true
    }
}

