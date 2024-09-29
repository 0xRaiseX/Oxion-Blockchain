use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};
use crate::transaction::Transaction;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u128,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        
        let nonce = 0; 
        let hash = Self::calculate_hash(index, timestamp, &previous_hash, nonce, &transactions);
        
        Block {
            index,
            timestamp,
            previous_hash,
            hash,
            nonce,
            transactions,
        }
    }

    pub fn calculate_hash(index: u64, timestamp: u128, previous_hash: &str, nonce: u64, transactions: &Vec<Transaction>) -> String {
        let transactions_json = serde_json::to_string(transactions).expect("Error serializing transactions");
        let input = format!("{}{}{}{}{}", index, timestamp, previous_hash, nonce, transactions_json);
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}
