use crate::error::DispatchError;
use crate::event::OrderEvent;
use std::sync::atomic::{AtomicU64, Ordering};

// handle 会跨 await 持有，
// send 可以跨线程可以转移，sync 可以跨线程借用
pub trait Handler: Send + Sync {
    // 只是分发，如果定义成 async 函数，就全部被包装成状态机，如果里面确实需要调用 async函数，可以使用 tokio::spawn
    // 返回值 () 产生 side effect 的函数，而不用返回数据
    fn handle(&self, event: &OrderEvent) -> Result<(), DispatchError>;
}

pub struct LoggingHandler;

impl Handler for LoggingHandler {
    fn handle(&self, event: &OrderEvent) -> Result<(), DispatchError> {
        match event {
            OrderEvent::New { id, price, qty } => {
                tracing::info!(order_id = id, price, qty, "new order");
            }
            OrderEvent::Cancel { id } => {
                tracing::info!(order_id = id, "cancel order");
            }
            OrderEvent::Trade { id, qty } => {
                tracing::info!(order_id = id, qty = qty, "trade");
            }
        }

        Ok(())
    }
}

pub struct MetricsHandler {
    new_count: AtomicU64,
    cancel_count: AtomicU64,
    trade_count: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub new_count: u64,
    pub cancel_count: u64,
    pub trade_count: u64,
}

impl MetricsHandler {
    pub fn new() -> Self {
        Self {
            new_count: AtomicU64::new(0),
            cancel_count: AtomicU64::new(0),
            trade_count: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            new_count: self.new_count.load(Ordering::Relaxed),
            cancel_count: self.cancel_count.load(Ordering::Relaxed),
            trade_count: self.trade_count.load(Ordering::Relaxed),
        }
    }
}

impl Default for MetricsHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Handler for MetricsHandler {
    fn handle(&self, event: &OrderEvent) -> Result<(), DispatchError> {
        // 入参是 &self ，但是却修改了数据，因为使用了内部可变性，因为如果只有一个线程挂有 &mut T，只能他修改，全宇宙只有你能改

        match event {
            OrderEvent::New { .. } => {
                self.new_count.fetch_add(1, Ordering::Relaxed);
            }
            OrderEvent::Cancel { .. } => {
                self.cancel_count.fetch_add(1, Ordering::Relaxed);
            }
            OrderEvent::Trade { .. } => {
                self.trade_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(())
    }
}

// 条件编译，只有测试时，才会构造失败场景
#[cfg(test)]
pub struct FailingHandler {
    pub fail_divisor: u64,
}

#[cfg(test)]
impl Handler for FailingHandler {
    fn handle(&self, event: &OrderEvent) -> Result<(), DispatchError> {
        if event.order_id() % self.fail_divisor == 0 {
            return Err(DispatchError::HandlerFailed(format!(
                "deliberate failure for order {}",
                event.order_id()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_handler_succeeds() {
        let handler = LoggingHandler;
        let event = OrderEvent::New {
            id: 1,
            price: 100,
            qty: 10,
        };

        assert!(handler.handle(&event).is_ok());
    }

    #[test]
    fn metrics_handler_counts() {
        let handler = MetricsHandler::new();
        handler
            .handle(&OrderEvent::New {
                id: 1,
                price: 100,
                qty: 10,
            })
            .unwrap();

        handler
            .handle(&OrderEvent::Trade { id: 2, qty: 10 })
            .unwrap();

        handler
            .handle(&OrderEvent::Trade { id: 3, qty: 10 })
            .unwrap();

        handler.handle(&OrderEvent::Cancel { id: 4 }).unwrap();

        let snap = handler.snapshot();

        assert_eq!(snap.new_count, 1);
        assert_eq!(snap.cancel_count, 1);
        assert_eq!(snap.trade_count, 2);
    }
}
