use crate::OrderEvent;

pub struct OrderEventProducer {
    sender: tokio::sync::mpsc::Sender<OrderEvent>,
}

impl OrderEventProducer {
    pub fn init_producer(sender: tokio::sync::mpsc::Sender<OrderEvent>) {
        OrderEventProducer { sender };
    }

    pub async fn send_order_event(&self, order_event: OrderEvent) {
        self.sender.send(order_event).await.unwrap();
    }
}
