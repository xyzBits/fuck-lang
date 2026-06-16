use criterion::{Criterion, criterion_group, criterion_main};
use order_event_dispatcher::dispatcher::Dispatcher;
use order_event_dispatcher::error::DispatchError;
use order_event_dispatcher::event::OrderEvent;
use order_event_dispatcher::handler::Handler;
use order_event_dispatcher::message::DispatchMessage;

struct NoopHandler;

// 为什么不用 LoggingHandler MetricsHandler ，因为这里想测纯 dispatcher 框架的开销，如果用日志，测到的主要是日志系统的开销
// 如果用复杂的业务 handler ，测到的是业务逻辑，不是 dispatcher本身
impl Handler for NoopHandler {
    fn handle(&self, event: &OrderEvent) -> Result<(), DispatchError> {
        Ok(())
    }
}

fn bench_dispatch_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();


    let event_counts: &[u64] = &[1_000, 10_000, 100_000];

    for &n in event_counts {
        c.bench_function(&format!("dispatch_{n}_events"), |b| {
            b.to_async(&rt).iter(|| async move {
                let (tx, rx) = tokio::sync::mpsc::channel(256);

                let handler: Vec<Box<dyn Handler>> = vec![Box::new(NoopHandler)];
                let dispatcher = Dispatcher::new(rx, handler);


                let producer = tokio::spawn(async move {
                    for i in 0..n {
                        let event = match i % 3 {
                            0 => OrderEvent::New {
                                id: 1,
                                price: 100,
                                qty: 10,
                            },
                            1 => OrderEvent::Cancel {id: 1},
                            _ => OrderEvent::Trade {id: 1, qty: 5},
                        };

                        if tx.send(DispatchMessage::FireAndForget(event)).await.is_err() {
                            break;
                        }
                    }
                });

                let stats = dispatcher.run().await;
                producer.await.unwrap();

                assert_eq!(stats.total_events, n);
            });
        });
    }
}

criterion_group!(benches, bench_dispatch_throughput);
criterion_main!(benches);
