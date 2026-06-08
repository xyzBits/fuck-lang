use order_event_dispatcher::OrderEvent;
use order_event_dispatcher::order_event_consumer::OrderEventConsumer;
use order_event_dispatcher::order_event_producer::OrderEventProducer;

#[tokio::main]
async fn main() {
    println!("Hello, order-event-dispatcher is starting!");

    let (sender, receiver) = tokio::sync::mpsc::channel::<OrderEvent>(1024);

    tokio::spawn(async move {
        OrderEventProducer::init_producer(sender);
    });

    tokio::spawn(async move {
        OrderEventConsumer::init_consumer(receiver);
    });
}
