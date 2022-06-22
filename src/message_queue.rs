use std::cell::UnsafeCell;
use std::future::Future;
use std::rc::Rc;
use std::task::Poll;

use futures::future::poll_fn;
use log::info;

use crate::block::Block;
use crate::Transaction;

#[derive(Debug, Clone)]
pub enum Message {
    BlockMined(Block),
    Test(String),
    AddTransaction(Transaction),
    Shutdown,
}

pub struct Queue {
    queue: Rc<UnsafeCell<Vec<Message>>>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            queue: Rc::new(UnsafeCell::new(vec![])),
        }
    }
    pub fn create_client(&self) -> QueueClient {
        QueueClient {
            queue: self.queue.clone(),
            offset: 0,
        }
    }
}

pub struct QueueClient {
    queue: Rc<UnsafeCell<Vec<Message>>>,
    offset: usize,
}

impl QueueClient {
    pub fn send(&mut self, msg: Message) {
        unsafe {
            let mut x = &mut *self.queue.get();
            x.push(msg);
        }
    }
    pub fn receive(&mut self) -> Option<Message> {
        let maybe_message = unsafe {
            let x = &*self.queue.get();
            x.get(self.offset)
        };
        match maybe_message {
            None => None,
            Some(msg) => {
                self.offset += 1;
                Some(msg.clone())
            }
        }
    }
    // pub fn receive_async(&mut self) -> impl Future<Output = Option<Message>> {
    //     let borrow = self.queue.borrow();
    //     let maybe_message = borrow.get(self.offset).cloned();
    //     if maybe_message.is_some() {
    //         self.offset += 1;
    //     }
    //     poll_fn(move |_cx| {
    //         match &maybe_message {
    //             Some(msg) => {
    //                 Poll::Ready(Some(msg.clone()))
    //             },
    //             None => Poll::Ready(None),
    //         }
    //     })
    // }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use async_executor::LocalExecutor;
    use async_std::task;
    use async_std::task::spawn;
    use futures::join;
    use futures_lite::future;
    use futures_lite::future::yield_now;
    use log::{info, LevelFilter};

    use super::*;

    #[test]
    fn test_queue() {
        let _ = env_logger::builder()
            .filter_module("richoin", LevelFilter::Info)
            .try_init();
        let local_ex = LocalExecutor::new();

        future::block_on(local_ex.run(async {
            let queue = Queue::new();

            let mut client = queue.create_client();
            let task1 = local_ex.spawn(async move {
                for c in 0..5 {
                    client.send(Message::Test(c.to_string()));
                    task::sleep(Duration::from_millis(100)).await;
                }
            });

            let mut client = queue.create_client();
            let task2 = local_ex.spawn(async move {
                for c in vec!['a', 'b', 'c', 'd'] {
                    client.send(Message::Test(c.to_string()));
                    task::sleep(Duration::from_millis(100)).await;
                }
            });

            let mut client = queue.create_client();
            let task3 = local_ex.spawn(async move {
                let mut received = vec![];
                while received.len() != 9 {
                    let msg = client.receive();
                    if msg.is_some() {
                        received.push(msg.unwrap());
                    }

                    task::sleep(Duration::from_millis(100)).await;
                }
            });

            let mut client = queue.create_client();
            let task4 = local_ex.spawn(async move {
                let mut received = vec![];
                while received.len() != 9 {
                    let msg = client.receive();
                    if msg.is_some() {
                        received.push(msg.unwrap());
                    }

                    task::sleep(Duration::from_millis(100)).await;
                }
            });

            let task5 = local_ex.spawn(async move {
                let duration = Duration::from_secs(1);
                let start = Instant::now();
                while Instant::now() - start < duration {
                    unsafe {
                        info!("Queue: {:?}", *queue.queue.get());
                    }
                    task::sleep(Duration::from_millis(100)).await;
                }
            });

            join!(task1, task2, task3, task4, task5);
        }));
    }
}
