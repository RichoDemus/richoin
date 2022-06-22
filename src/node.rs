use async_std::task::yield_now;
use log::info;

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::message_queue::QueueClient;
use crate::{Message, Transaction};

pub enum NodeType {
    Passive,
    Miner,
    BigRewardMiner,
}

pub struct Node {
    message_client: QueueClient,
    blockchain: Blockchain,
    node_type: NodeType,
    name: String,
    pending_transactions: Vec<Transaction>,
    next_block: Option<Block>,
}

impl Node {
    pub fn new(
        message_client: QueueClient,
        difficulty: usize,
        node_type: NodeType,
        name: String,
    ) -> Self {
        Self {
            message_client,
            blockchain: Blockchain::new(difficulty),
            node_type,
            name,
            pending_transactions: vec![],
            next_block: None,
        }
    }

    pub async fn start(&mut self) {
        loop {
            if let Some(msg) = self.message_client.receive() {
                match msg {
                    Message::BlockMined(block) => {
                        info!("Block mined: {:?}", block);
                        match self.node_type {
                            NodeType::Passive | NodeType::Miner => {
                                if self.blockchain.validate_new_block(&block) {
                                    // this is an ok block
                                    match self.blockchain.add_block(block.clone()) {
                                        Ok(_) => {
                                            info!("Chain updated");
                                        }
                                        Err(e) => info!("Chain update failed: {:?}", e),
                                    }
                                }
                            }
                            NodeType::BigRewardMiner => {
                                match self.blockchain.add_block(block.clone()) {
                                    Ok(_) => {
                                        info!("Chain updated");
                                    }
                                    Err(e) => info!("Chain update failed: {:?}", e),
                                }
                            }
                        }
                    }
                    Message::Test(_) => todo!("remove"),
                    Message::AddTransaction(transaction) => match self.node_type {
                        NodeType::Miner | NodeType::BigRewardMiner => {
                            self.pending_transactions.push(transaction);
                        }
                        _ => (),
                    },
                    Message::Shutdown => break,
                }
            }
            match self.node_type {
                NodeType::Miner | NodeType::BigRewardMiner => {
                    if let Some(block) = &self.next_block {
                        if block.previous_hash != self.blockchain.last_block().calculate_hash() {
                            info!(
                                "Wrong hash, block discarded expected: {} was {}",
                                self.blockchain.last_block().calculate_hash(),
                                block.previous_hash
                            );
                            let _ = self.next_block.take();
                        }
                    }
                    if self.next_block.is_none() {
                        let last_block = self.blockchain.last_block();
                        let mut block =
                            Block::new(last_block.index + 1, last_block.calculate_hash());
                        let reward = match self.node_type {
                            NodeType::Passive => panic!("Can't happen"),
                            NodeType::Miner => 1,
                            NodeType::BigRewardMiner => 10,
                        };
                        block.add_transaction(Transaction::MinerReward(self.name.clone(), reward));
                        self.next_block.replace(block);
                    }
                    if let Some(ref mut block) = self.next_block {
                        for transaction in &self.pending_transactions {
                            if !block.transactions.contains(transaction) {
                                info!("Added transaction {:?}", transaction);
                                block.add_transaction(transaction.clone());
                                block.proof_of_work = 0;
                            }
                        }
                        block.mine_once();
                        if block.is_mined(self.blockchain.difficulty) {
                            info!("Block successfully mined: {:?}", block);
                            let block = self.next_block.take().expect("Should be a block here");
                            let _ = self.blockchain.add_block(block.clone());
                            self.message_client.send(Message::BlockMined(block));
                        }
                    }
                }
                _ => (),
            }
            yield_now().await;
        }
        info!("{}: Blocks: {}", self.name, self.blockchain.chain.len());
        info!("{}: Wallets: {:?}", self.name, self.blockchain.wallets());
        info!("{} shutting down", self.name);
    }
}
