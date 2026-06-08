#[derive(Debug)]
struct OrderEvent {
    order_id: u64,
    data: String,
}

#[test]
fn test() {
    let ring_buffer_size = 1024;
    let (sender, receiver) = crossbeam_channel::bounded::<OrderEvent>(ring_buffer_size);

    let consumer_thread = std::thread::spawn(move || {
        for event in receiver.iter() {
            println!(
                "received event: id={} data={:?}",
                event.order_id, event.data
            );
        }

        println!("consumer terminated");
    });

    for i in 0..100 {
        let event = OrderEvent {
            order_id: i,
            data: format!("Trade-{}", i),
        };

        sender.send(event).expect("couldn't send event");
    }

    drop(sender);

    consumer_thread.join().expect("couldn't join consumer");
}

use core::range::Range;

#[derive(Debug, Copy, Clone)]
struct SliceWindow {
    x_bounds: Range<usize>,
    y_bounds: Range<usize>,
}

#[test]
fn test_1() {
    let window = SliceWindow {
        x_bounds: Range::from(0..10),
        y_bounds: Range::from(5..15),
    };

    let window_copy1 = window;
    let window_copy2 = window;

    println!("{:?}", window_copy1);
    println!("{:?}", window_copy2);

    println!("{}", window.x_bounds.end);
    println!("{:?}", window.y_bounds.start);
}

fn fatal_error(msg: &str) -> ! {
    panic!("fatal error:{}", msg);
}

#[test]
#[ignore]
fn test_never() {
    let server_response: Result<(i32, String), &str> = Err("database connection timeout");

    let (status_code, response_body) = match server_response {
        Ok(data) => data,
        Err(e) => fatal_error(e),
    };

    println!("status code: {}, response: {}", status_code, response_body);
}
