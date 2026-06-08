use crate::OrderEvent;
use tokio::sync::mpsc::Receiver;

pub struct OrderEventConsumer {
    receiver: Receiver<OrderEvent>,
}

impl OrderEventConsumer {
    pub fn init_consumer(receiver: Receiver<OrderEvent>) {
        OrderEventConsumer { receiver };
    }

    pub async fn consume(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            println!("event={:?}", event);
        }
    }
}
