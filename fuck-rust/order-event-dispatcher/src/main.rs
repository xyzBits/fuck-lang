use order_event_dispatcher::dispatcher::Dispatcher;
use order_event_dispatcher::error::DispatchError;
use order_event_dispatcher::event::OrderEvent;
use order_event_dispatcher::handler::{Handler, LoggingHandler, MetricsHandler};
use order_event_dispatcher::message::DispatchMessage;
use order_event_dispatcher::producer;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    // 最多缓存32条没有被消费的消息，不是sender 数量，也不是task数量，只是缓冲区大小
    // 如果 producer 发送太快，而dispatch 消费太慢，缓冲区满了以后，tx.semd(msg).await 会等待，这就是 backpressure
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let metrics = MetricsHandler::new();
    // metrics 既要传给 dispatch ，又要在main中使用，因此用 Arc 包装一下，
    let arc_metrics = std::sync::Arc::new(metrics);

    let handlers: Vec<Box<dyn Handler>> = vec![
        Box::new(LoggingHandler),
        Box::new(ArcHandler(arc_metrics.clone())),// 如果直接用 Box::new(metrics) 会拿走所有权
    ];

    // clone 是复制出一个 sender，指向同一个 channel
    let producer_tx = tx.clone();
    let producer_handle = tokio::spawn(async move {
        producer::produce(producer_tx).await;
    });

    let oneshot_tx = tx.clone();
    let oneshot_handle = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let event = OrderEvent::New {
            id: 999,
            price: 42_000,
            qty: 1,
        };

        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        let msg = DispatchMessage::RequestReply {
            event,
            reply: reply_tx,
        };

        oneshot_tx.send(msg).await.expect("channel closed");

        match reply_rx.await {
            Ok(Ok(())) => println!("oneshot reply: all handlers succeed for order 999"),
            Ok(Err(e)) => println!("oneshot reply error: {:?}", e),
            Err(_) => println!("oneshot reply error"),
        }
    });

    // main中的sender 自己不再发送消息了，只保留 producer 和 oneshot 中的sender，这样dispatcher 以后能正常感知所有发送方结束 发送
    // 如果不 drop ，即使 producer oneshot中的都结束了，dispatcher 也还会以为有sender 继续发消息，recv.await会一直等等，程序可能卡住
    // producer_tx oneshot_tx 都还活着，因为他们已经move 进各自的 task 了
    // drop 了原始的 tx，不影响clone出来的
    drop(tx);

    let dispatcher = Dispatcher::new(rx, handlers);
    
    // await 等待消费完成
    let stats = dispatcher.run().await;

    // 保证 main 不会在 task 还没结束 时就退出，
    producer_handle.await.expect("producer task panicked");
    oneshot_handle.await.expect("oneshot task panicked");

    let snap = arc_metrics.snapshot();
    println!("===dispatch summary=====");
    println!("total events: {}", stats.total_events);
    println!("handler errors: {}", stats.handler_errors);

    println!(
        "metrics: new={}, cancel={}, trade={}",
        snap.new_count, snap.cancel_count, snap.trade_count
    );
}

// 同一个 MetricsHandler 既可以交给 dispatcher 处理事件，又能在 main 里最后读取统计结果
struct ArcHandler(std::sync::Arc<MetricsHandler>);

// Metrics 实现了 Handler ，不代表 ArcHandler 也实现了，所以要给包装层也实现这个 trait
// Arc<MetricsHandler> 解析共享所有权
// impl Handler 解决它可以被当成 dyn Handler 使用
impl Handler for ArcHandler {
    fn handle(&self, event: &OrderEvent) -> Result<(), DispatchError> {
        self.0.handle(event)
    }
}
