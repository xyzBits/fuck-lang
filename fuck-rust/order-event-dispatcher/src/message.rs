use crate::error::DispatchError;
use crate::event::OrderEvent;

/// 消息流程
/// 1 生产者
///     构造好要发送的 OrderEvent
///     创建一对临时的 oneshot 通道，let (reply_tx, reply_rx) = oneshot::channel();
///     把事件和 reply_tx 发信端 打包进 RequestReply 变体，丢进 mpsc 大通道里，
///     紧接着，生产者在一旁乖乖 await 那个 reply_rx 收信端，挂起等待
/// 2 消费者
///     从 mpsc 大道通里拿到了 RequestReply 包裹
///     拆开包裹，拿出 OrderEvent 交给一堆 Handler 去处理
///     处理完了，无论是成功 Ok(()) 还是失败 Err
///     处理器会出包裹中附带的 reply，就是那个 sender 调用 reply.send
/// 3 结果交汇
///     分发器把信一寄出，刚才在苦苦等待的生产者 reply_rx 瞬间就被唤醒了，并且拿到了具体的结果处理集
pub enum DispatchMessage {
    FireAndForget(OrderEvent),

    RequestReply {
        event: OrderEvent,
        // 处理完，让对方把结果传回来，带着事件，塞好了一个可以用来 寄信回来 的信封
        // tokio 中的返回通道，只能使用一次，发完就报废，是为了这种一次性回调的
        reply: tokio::sync::oneshot::Sender<Result<(), DispatchError>>,
    },
}

impl DispatchMessage {
    pub fn event(&self) -> &OrderEvent {
        match self {
            DispatchMessage::FireAndForget(event) | DispatchMessage::RequestReply { event, .. } => {
                event
            }
        }
    }
}
