use crate::error::DispatchError;
use crate::event::OrderEvent;
use crate::handler::Handler;
use crate::message::DispatchMessage;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchStats {
    pub total_events: u64,
    pub handler_errors: u64,
}

pub struct Dispatcher {
    rx: mpsc::Receiver<DispatchMessage>,
    handlers: Vec<Box<dyn Handler>>,
}

impl Dispatcher {
    pub fn new(rx: mpsc::Receiver<DispatchMessage>, handlers: Vec<Box<dyn Handler>>) -> Self {
        Self { rx, handlers }
    }

    pub fn from_event_rx(rx: mpsc::Receiver<OrderEvent>) -> EventRxAdapter {
        EventRxAdapter { rx }
    }

    pub async fn run(mut self) -> DispatchStats {
        let mut stats = DispatchStats {
            total_events: 0,
            handler_errors: 0,
        };

        // 如果 channel 中有缓存消息，先消费
        // 如果所有 sender 都drop 了并且 buffer 也空了
        // rx.recv().await 返回 None
        // 不是 sender 被 drop 就立刻丢弃消息，否则会丢消息
        while let Some(msg) = self.rx.recv().await {
            stats.total_events += 1;

            match msg {
                DispatchMessage::FireAndForget(event) => {
                    self.dispatch_fire_and_forget(&event, &mut stats);
                }

                DispatchMessage::RequestReply { event, reply } => {
                    let result = self.dispatch_request_reply(&event, &mut stats);
                    let _ = reply.send(result);
                }
            }
        }

        stats
    }

    fn dispatch_fire_and_forget(&self, event: &OrderEvent, stats: &mut DispatchStats) {
        for handler in &self.handlers {
            if let Err(e) = handler.handle(event) {
                tracing::error!(
                    order_id = event.order_id(),
                    error = %e,
                    "handler error - continuing",
                );

                stats.handler_errors += 1;
            }
        }
    }

    fn dispatch_request_reply(
        &self,
        event: &OrderEvent,
        stats: &mut DispatchStats,
    ) -> Result<(), DispatchError> {
        let mut first_error: Option<DispatchError> = None;

        for handler in &self.handlers {
            if let Err(e) = handler.handle(event) {
                tracing::error!(
                    order_id = event.order_id(),
                    error = %e,
                    "handler error (request - reply)",
                );

                stats.handler_errors += 1;
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }

        match first_error {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }
}

pub struct EventRxAdapter {
    rx: mpsc::Receiver<OrderEvent>,
}

impl EventRxAdapter {
    pub fn with_handlers(self, handlers: Vec<Box<dyn Handler>>) -> WrappedDispatcher {
        WrappedDispatcher {
            rx: self.rx,
            handlers,
        }
    }
}

pub struct WrappedDispatcher {
    rx: mpsc::Receiver<OrderEvent>,
    handlers: Vec<Box<dyn Handler>>,
}

impl WrappedDispatcher {
    pub async fn run(mut self) -> DispatchStats {
        let mut stats = DispatchStats {
            total_events: 0,
            handler_errors: 0,
        };

        while let Some(event) = self.rx.recv().await {
            stats.total_events += 1;

            for handler in &self.handlers {
                if let Err(e) = handler.handle(&event) {
                    tracing::error!(
                        order_id = event.order_id(),
                        error = %e,
                        "handler error - continuing",
                    );
                    stats.handler_errors += 1;
                }
            }
        }

        tracing::info!(
            total = stats.total_events,
            handlers = stats.handler_errors,
            "dispatcher finished - channel closed",
        );

        stats
    }
}
