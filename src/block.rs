use chrono::Utc;
use log::{info, trace};
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum Transaction {
    MinerReward(String, u64),
    Text(String),
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub proof_of_work: u64,
    pub previous_hash: String,
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

impl Block {
    pub fn calculate_hash(&self) -> String {
        let mut block_data = self.clone();
        block_data.hash = String::default();
        let serialized_block_data = serde_json::to_string(&block_data).unwrap();
        // Calculate and return SHA-256 hash value.
        let mut hasher = Sha256::new();
        hasher.update(serialized_block_data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    // Create a new block. The hash will be calculated and set automatically.
    pub fn new(index: u64, previous_hash: String) -> Self {
        // Current block to be created.
        let block = Block {
            index,
            // timestamp: Utc::now().timestamp_millis() as u64,
            proof_of_work: u64::default(),
            previous_hash,
            hash: String::default(),
            transactions: vec![],
        };

        block
    }
    // Mine block hash.
    pub fn mine(&mut self, difficulty: usize) {
        loop {
            if !self.hash.starts_with(&"0".repeat(difficulty)) {
                self.proof_of_work += 1;
                self.hash = self.calculate_hash();
            } else {
                break;
            }
        }
    }

    pub fn mine_once(&mut self) {
        self.proof_of_work += 1;
        self.hash = self.calculate_hash();
        // info!("Tried to mine index {} for PoW {}", self.index, self.proof_of_work);
    }

    pub fn is_mined(&self, difficulty: usize) -> bool {
        self.hash.starts_with(&"0".repeat(difficulty))
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
        self.transactions.sort();
        self.transactions.dedup();
    }
}
