use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use log::info;

use crate::block::{Block, Transaction};

type Blocks = Vec<Block>;

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub genesis_block: Block,
    pub chain: Blocks,
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        // First block in the chain.
        let mut genesis_block = Block {
            index: 0,
            // timestamp: 0,
            proof_of_work: u64::default(),
            previous_hash: String::default(),
            hash: String::default(),
            transactions: vec![],
        };
        genesis_block.mine(difficulty);

        // Create chain starting from the genesis chain.
        let mut chain = Vec::new();
        info!("First block added to chain -> {:?}", genesis_block);
        chain.push(genesis_block.clone());

        // Create a blockchain Instance.
        let blockchain = Blockchain {
            genesis_block,
            chain,
            difficulty,
        };

        blockchain
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let last_block = self.chain.last().context("Should be a block")?;
        if block.index != last_block.index + 1 {
            bail!(
                "New block should have index {}, had {}",
                last_block.index + 1,
                block.index
            );
        }
        if block.previous_hash != last_block.hash {
            bail!(
                "New block prev hash should be {}, had {}",
                last_block.hash,
                block.previous_hash
            );
        }

        info!("New block added to chain -> {:?}", block);
        self.chain.push(block);
        Ok(())
    }

    pub fn validate_new_block(&self, block: &Block) -> bool {
        let rewards = block
            .transactions
            .iter()
            .filter_map(|transaction| match transaction {
                Transaction::MinerReward(name, reward) => Some((name, reward)),
                Transaction::Text(_) => None,
                Transaction::Shutdown => None,
            })
            .collect::<Vec<_>>();
        if rewards.len() != 1 {
            info!("Bad block, wrong amount of rewards");
            return false;
        }

        for (_, amount) in rewards {
            if *amount != 1 {
                info!("Bad block, wrong reward");
                return false;
            }
        }

        true
    }

    pub fn wallets(&self) -> HashMap<String, u64> {
        self.chain
            .iter()
            .flat_map(|b| b.transactions.as_slice())
            .fold(HashMap::new(), |mut acc, transaction| {
                match transaction {
                    Transaction::MinerReward(id, amount) => {
                        *acc.entry(id.clone()).or_default() += amount
                    }
                    Transaction::Text(_) => {}
                    Transaction::Shutdown => {}
                }
                acc
            })
    }

    pub fn transactions(&self) -> HashMap<u64, Vec<String>> {
        let mut result = HashMap::new();
        for block in &self.chain {
            for transaction in &block.transactions {
                if let Transaction::Text(text) = transaction {
                    result
                        .entry(block.index)
                        .or_insert(vec![])
                        .push(text.clone());
                }
            }
        }
        result
    }

    pub fn last_block(&self) -> Block {
        self.chain.last().cloned().expect("Chain can't be empty")
    }
}
