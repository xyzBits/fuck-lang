pub mod handler;
pub mod order_event_consumer;
pub mod order_event_producer;

#[derive(Debug)]
pub enum OrderEvent {
    New { id: u64, price: u64, qty: u64 },
    Cancel { id: u64 },
    Trade { id: u64, qty: u64 },
}

enum DispatchError {
    NotFound,
}

trait Handler {
    type Output;

    fn handle(&self, event: &OrderEvent) -> Result<(), DispatchError> {
        Ok(())
    }
}
