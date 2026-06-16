use tokio::sync::mpsc;
use crate::event::OrderEvent;
use crate::message::DispatchMessage;

pub async fn produce(tx: mpsc::Sender<DispatchMessage>) {
    let events = vec![
      OrderEvent::New {
          id: 1,
          price: 50_000,
          qty: 10,
      }  ,
      OrderEvent::New {
          id: 2,
          price: 51_000,
          qty: 20,
      },
      OrderEvent::Trade {id: 1, qty: 3},
      OrderEvent::Cancel {id: 2},
      OrderEvent::Trade {id: 1, qty: 4},
      OrderEvent::New {
          id: 3,
          price: 49_500,
          qty: 50,
      },
      OrderEvent::Cancel {id: 3},
      OrderEvent::Trade {id: 1, qty: 0},
    ];

    for event in events {
        let msg = DispatchMessage::FireAndForget(event);
        
        // 如果channel 满了，再调用，会发生的是，async task 让出执行权，直到channel 有空位，
        // 不是 os 线程阻塞，也不是 tokio runtime 整体卡住
        // 而是 producer future 暂停，tokio 去调度别的 task 
        // 等 receiver 消费出一个槽位，
        // tokio 再唤醒这个 producer future 
        // send 继续完成 
        // bounded channel 的 back pressure ，不让你无限制堆内存，而是把压力传回给生产者 
        if tx.send(msg).await.is_err() {
            tracing::warn!("receiver dropped - stopping producer");
            return;
        }
    }
    
    tracing::info!("producer finished - all events sent");
}