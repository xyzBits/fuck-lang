#![allow(clippy::all)]

use crossbeam::channel;
use futures::task;
use futures::task::ArcWake;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Notify;

async fn my_async_fn() {
    println!("hello from async");
    // let _socket = TcpStream::connect("127.0.0.1:8080").await.unwrap();

    // 连接百度或腾讯的 HTTP 端口
    let _socket = TcpStream::connect("baidu.com:80").await.unwrap();
    println!("async TCP operation complete");
}

#[tokio::test]
async fn test_my_async_fn() {
    let what_is_this = my_async_fn();

    what_is_this.await;
}

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            let waker = cx.waker().clone();
            let when = self.when;

            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                waker.wake();
            });

            Poll::Pending
        }
    }
}

#[tokio::test]
async fn test_my_future() {
    let when = Instant::now() + Duration::from_millis(100);
    let future = Delay { when };

    let out = future.await;
    assert_eq!(out, "done");
}

enum MainFuture {
    State0,
    State1(Delay),
    Terminated,
}

impl Future for MainFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        use MainFuture::*;

        loop {
            match *self {
                State0 => {
                    let when = Instant::now() + Duration::from_millis(10);
                    let future = Delay { when };
                    *self = State1(future);
                }
                State1(ref mut my_future) => match Pin::new(my_future).poll(cx) {
                    Poll::Ready(out) => {
                        assert_eq!(out, "done");
                        *self = Terminated;
                        return Poll::Ready(());
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                },
                Terminated => {
                    panic!("future polled after completion")
                }
            }
        }
    }
}

struct MiniTokio {
    // tasks: VecDeque<Task>,
    scheduled: channel::Receiver<Arc<Task>>,
    sender: channel::Sender<Arc<Task>>,
}

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: channel::Sender<Arc<Task>>,
}

// type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, scheduled) = channel::unbounded();
        MiniTokio { scheduled, sender }
    }

    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: self.sender.clone(),
        });

        self.sender.send(task).unwrap();
    }

    fn run(&mut self) {
        while let Ok(task) = self.scheduled.recv() {
            let waker = task::waker_ref(&task);
            let mut cx = Context::from_waker(&waker);

            let mut future = task.future.lock().unwrap();
            if future.as_mut().poll(&mut cx).is_pending() {}
        }
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.executor.send(arc_self.clone()).unwrap();
    }
}

#[test]
#[ignore]
fn test_mini_tokio() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async move {
        let when = Instant::now() + Duration::from_millis(100);
        let future = Delay { when };

        let out = future.await;
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}

#[tokio::test]
async fn test_tokio_notify() {
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();

    tokio::spawn(async move {
        println!("wait notify....");
        notify2.notified().await;
        println!("received notify");
    });

    println!("prepare send notify");
    notify.notify_one();
}

mod future_demo {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    // 定义一个完事后什么也不返回，返回 void 的 future
    struct VoidFuture;

    impl Future for VoidFuture {
        type Output = (); // 对应 java 的 void

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            // 物理上返回 Poll::Ready(())，里面的 () 就是单元类型的一个实例
            Poll::Ready(())
        }
    }

    #[test]
    fn test_void_future_output() {
        let mut future = VoidFuture;
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        // 验证 poll 出来的结果确实是 Ready(())
        let res = Pin::new(&mut future).poll(&mut cx);
        assert!(matches!(res, Poll::Ready(())));

        // 物理验证，() 在大小是 0 字节
        assert_eq!(std::mem::size_of::<()>(), 0);
    }

    struct NoSelfRefFuture {
        count: i32,
    }

    impl Future for NoSelfRefFuture {
        type Output = i32;

        // 虽然合并硬性要求 Pin<&mut Self> 但是因为我没有自引用，我可以用 Pin::as_mut().get_mut() 把它打回原形
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let raw_self = self.as_mut().get_mut();

            raw_self.count += 1;

            Poll::Ready(raw_self.count)
        }
    }

    #[test]
    fn test_no_self_ref_pin_bypass() {
        let mut my_fut = NoSelfRefFuture { count: 99 };

        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        let res = Pin::new(&mut my_fut).poll(&mut cx);
        assert!(matches!(res, Poll::Ready(100)));
    }
}

mod learn_pin {

    #[derive(Debug)]
    struct Player {
        id: i32,
    }

    #[test]
    fn test_step1_move_address() {
        let p1 = Player { id: 99 };
        // 打印 p1 在栈上物理内存地址
        let p1_addr = &p1 as *const Player;
        println!("p1 的物理内存地址: {:p}", p1_addr);

        // 发生 move player 结构体的4个字节都被复制到了新地址 p2
        let p2 = p1;

        let p2_addr = &p2 as *const Player;
        println!("p2 的物理内存地址: {:p}", p2_addr);

        // p1 p2 的物理地址百分百不一样了，数据搬家了
        assert_ne!(format!("{:p}", p1_addr), format!("{:p}", p2_addr));
    }

    struct SelfRef {
        data: [u8; 5],
        pointer: *const u8, // 这是一个裸指针，存的是一个绝对的内存物理地址
    }

    #[inline(never)]
    fn clean_the_room() {
        let _garbage = [0xAAu8; 2048000];
    }

    #[test]
    fn test_step2_self_ref_disaster() {
        let mut item = SelfRef {
            data: [10, 20, 30, 40, 50],
            pointer: std::ptr::null(),
        };

        item.pointer = &item.data[0] as *const u8;

        println!("item 的物理地址: {:p}", &item);
        println!("pointer 里面写着的地址: {:p}", item.pointer);

        assert_eq!(
            format!("{:p}", &item.data[0]),
            format!("{:p}", item.pointer)
        );

        clean_the_room();
        let moved_item = item;

        println!("moved_item 的物理地址: {:p}", &moved_item);
        println!(
            "moved_item.pointer 里面写着的地址: {:p}",
            moved_item.pointer
        );

        unsafe {
            println!("读取指针指向的值: {}", *moved_item.pointer);
        }
    }
}

mod learn_tokio_spawn {
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};
    use tokio::net::TcpListener;
    use tokio::task;

    #[tokio::test]
    async fn test_tokio_spawn() {
        let listener = TcpListener::bind("baidu.com:80").await.unwrap();

        tokio::spawn(async move {
            let rc = Arc::new("Hello, World!");

            tokio::fs::read_to_string("foo.txt").await.unwrap();

            println!("rc={}", rc);
        });
    }

    fn heavy_matching_logic(order_id: u64) -> String {
        let mut _burn_cpu = 0;

        for _ in 0..500_000_000 {
            _burn_cpu += 1;
        }

        format!("订单 {} 撮合成功 ", order_id)
    }

    #[tokio::test]
    async fn test_matching_engine_isolation() {
        println!("主线程异步网络正在畅快收包。。。");

        let order_id = 888888;

        let join_handle = task::spawn_blocking(move || heavy_matching_logic(order_id));

        println!("网络线程：我继续去收下一个订单包了，撮合的重活让后台哥去干了");

        let match_result = join_handle.await.unwrap();
        assert_eq!(match_result, format!("订单 {} 撮合成功 ", order_id));
    }

    #[tokio::test]
    #[ignore]
    async fn test_console() {
        console_subscriber::init();

        tokio::spawn(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    #[tokio::test]
    async fn test_send_channel() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<i32>(100);

        tokio::spawn(async move {
            for i in 0..3 {
                tx.send(i).await.unwrap();
            }
        });

        while let Some(item) = rx.recv().await {
            println!("received item: {}", item);
        }
    }

    #[test]
    fn test_std_channel_blocking_when_full() {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        tx.send("第一单").unwrap();

        let start_time = Instant::now();

        thread::spawn(move || {
           thread::sleep(Duration::from_millis(50));
            let _  = rx.recv().unwrap();// 50ms 后才收走第一单，腾出空位
        });

        println!("主线程 准备发送第二单，但此时通道是满的");

        tx.send("第二单").unwrap();

        let elapsed = start_time.elapsed();

        println!("主线程 终于发出第二单了，耗时 {:?}", elapsed);

        assert!(elapsed >= Duration::from_millis(50));
    }



    #[tokio::test(flavor = "current_thread")]
    async fn test_tokio_channel_yielding_when_full() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<_>(1);

        tx.send("第一单").await.unwrap();

        let tx_clone = tx.clone();
        tokio::task::spawn(async move {
            println!("子任务 准备发送第二单，通道是满的，我即将卡住");

            tx_clone.send("第二单").await.unwrap();

            println!("子任务 极其丝滑，我的第二单终于发出去了");
        });

        task::yield_now().await;

        println!("主任务 此时子任务已经卡住了，但我主回路所在的物理线程依然活蹦乱跳");
        println!("主任务  我现在来消费第一单，给子任务腾地方");

        let res1 = rx.recv().await.unwrap();
        assert_eq!(res1, "第一单");

        let res2 = rx.recv().await.unwrap();
        assert_eq!(res2, "第二单");



    }

    use tokio::sync::oneshot;
    use tokio::time::{sleep};

    #[tokio::test(flavor = "current_thread")]
    async fn test_oneshot_channel() {
        // 1. 创建 oneshot 通道，返回发送端 (tx) 和接收端 (rx)
        // 注意：oneshot 不需要指定容量，因为它只能装 1 个数据
        let (tx, rx) = oneshot::channel();

        // 2. 派生一个后台异步任务去干活
        tokio::spawn(async move {
            println!("[后台任务] 开始执行极其复杂的计算...");
            sleep(Duration::from_millis(500)).await; // 模拟耗时操作

            let result = "计算结果：42";
            println!("[后台任务] 计算完成，准备把结果扔回去！");

            // 3. 使用 tx.send() 发送数据
            // 注意：send 消耗了 tx 的所有权，所以 tx 只能用这一次！
            // 如果接收端 (rx) 已经被销毁了，send 会返回 Err。
            if let Err(_) = tx.send(result) {
                println!("[后台任务] 糟了，接收端跑路了，数据发不出去了！");
            }
        });

        println!("[主任务] 活儿已经交出去了，我在这里等结果...");

        // 4. 使用 rx.await 等待接收数据
        // 如果发送端 (tx) 还没发数据就被销毁了（比如后台任务 panic 了），这里会返回 Err
        match rx.await {
            Ok(data) => println!("[主任务] 成功收到快递: {}", data),
            Err(_) => println!("[主任务] 完蛋，发送端死在半路上了，我永远等不到结果了！"),
        }
    }
}

#[test]
fn test_send_success() {
    let data = "hello world".to_string();

    let handle = thread::spawn(move || {
        println!("msg={}", data);
    });

    handle.join().unwrap();

    let x = f64::NAN;

    // println!("{}", data);
}