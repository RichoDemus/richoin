use std::time::Duration;

use async_executor::LocalExecutor;
use async_std::task::{sleep, yield_now};
use chrono::Utc;
use futures::future::join_all;
use futures::join;
use futures_lite::future;
use log::{info, LevelFilter};
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::block::Transaction;
use crate::message_queue::{Message, Queue};
use crate::node::{Node, NodeType};
use crate::Message::Shutdown;

mod block;
mod blockchain;
mod message_queue;
mod node;

fn main() {
    let _ = env_logger::builder()
        .filter_module("richoin", LevelFilter::Info)
        .try_init();
    let local_ex = LocalExecutor::new();

    future::block_on(local_ex.run(async {
        let difficulty = 2;

        let mut queue = Queue::new();

        let mut tasks = vec![];
        for (name, node_type) in vec![
            ("Miner 1", NodeType::Miner),
            ("Miner 2", NodeType::Miner),
            ("Node 1", NodeType::Passive),
            ("Node 2", NodeType::Passive),
            ("Evil Miner", NodeType::BigRewardMiner),
        ] {
            let client = queue.create_client();
            tasks.push(local_ex.spawn(async move {
                let mut node = Node::new(client, difficulty, node_type, name.to_string());
                node.start().await;
            }));
        }

        let mut client = queue.create_client();
        let spammer = local_ex.spawn(async move {
            loop {
                if let Some(Message::Shutdown) = client.receive() {
                    break;
                }
                client.send(Message::AddTransaction(Transaction::Text(
                    "Hello".to_string(),
                )));
                sleep(Duration::from_millis(1000)).await;
            }
            info!("spammer task finished");
        });
        tasks.push(spammer);

        let mut client = queue.create_client();
        let shutdown = local_ex.spawn(async move {
            loop {
                if let Some(Message::BlockMined(block)) = client.receive() {
                    if block.index > 19 {
                        client.send(Message::AddTransaction(Transaction::Shutdown));
                    }
                    if block.transactions.contains(&Transaction::Shutdown) {
                        client.send(Shutdown);
                        break;
                    }
                }
                yield_now().await;
            }
            info!("shutdown task finished");
        });
        tasks.push(shutdown);

        join_all(tasks).await;
    }));
}
