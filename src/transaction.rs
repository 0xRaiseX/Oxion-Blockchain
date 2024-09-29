// src/transaction.rs
use serde::{Serialize, Deserialize};
use std::collections::BinaryHeap;
use std::sync::Arc;
use log::info;
use crossbeam::channel::Receiver;
use std::collections::HashSet;
use sha2::Sha256;
use sha2::Digest;
use std::time::UNIX_EPOCH;
use std::time::SystemTime;
use tokio::sync::Mutex;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Transaction {
    pub addr: String,
    pub to: String,
    pub amount: u128,
    pub timestamp: u128,
    pub fee: u64,
    pub hash: String,
}

impl Transaction {
    pub fn new(addr: String, to: String, amount: u128, fee: u64, timestamp: u128) -> Transaction {
        let hash = Self::calculate_hash(&addr, &to, amount, timestamp, fee);
        
        Transaction {
            addr, 
            to, 
            amount,
            timestamp, 
            fee,
            hash,
        }
    }

    pub fn calculate_hash(addr: &str, to: &str, amount: u128, timestamp: u128, fee: u64) -> String {
        let input = format!("{}{}{}{}{}", addr, to, amount, timestamp, fee);
        let mut hasher = Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

// impl Ord for Transaction {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         // Reverse позволяет нам сортировать в порядке убывания
//         self.fee.cmp(&other.fee).then_with(|| self.timestamp.cmp(&other.timestamp))
//     }
// }

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        // Сравниваем по fee (по возрастанию)
        let fee_order = self.fee.cmp(&other.fee);
        
        // Если fee равны, сортируем по timestamp (по убыванию)
        if fee_order == Ordering::Equal {
            return other.timestamp.cmp(&self.timestamp); // Меняем порядок сравнения
        }

        fee_order
    }
}
impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Mempool {
    transactions: BinaryHeap<Transaction>,
    tx_hashes: HashSet<String>,
}

impl Mempool {
    pub fn new() -> Self {
        Mempool {
            transactions: BinaryHeap::new(),
            tx_hashes: HashSet::new(),
        }
    }

    // pub async fn run(&mut self) {
    //     let receiver = self.receiver.clone();

    //     loop {
    //         let _tx_hashes = self.tx_hashes.lock().await;
    //         for _transaction in &receiver {
    //             // if tx_hashes.contains(&transaction.hash) {
    //             //     println!("Транзакция уже содержится");
    //             //     continue;
    //             // }
    //             // self.add_transaction(transaction).await;
    //             info!("Транзакция получена.");
    //         }
    //     }
    // }

    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        match self.tx_hashes.insert(tx.hash.clone()) {
            true => {
                self.transactions.push(tx);
                info!("Tx in mempool: {}", self.transactions.len());
                println!("{}", self.tx_hashes.len());
                return true;
            }
            false => {
                return false;
            }
        }
    }

    async fn remove_transaction(&mut self, tx_id: &str) -> Option<Transaction> {
        let mut temp_heap = BinaryHeap::new();
        let mut removed_transaction = None;

        while let Some(tx) = self.transactions.pop() {
            if tx.hash == tx_id {
                removed_transaction = Some(tx);
                break;
            } else {
                temp_heap.push(tx);
            }
        }

        self.transactions = temp_heap;
        removed_transaction
    }

    pub async fn get_all_transactions(&self) -> Vec<Transaction> {
        self.transactions.iter().cloned().collect()
    }

    pub fn get_highest_fee_transaction(&mut self) -> Option<Transaction> {
        let transaction = self.transactions.pop();
        if let Some(ref tx) = transaction {
            self.tx_hashes.remove(&tx.hash);
        }

        transaction
    }
}