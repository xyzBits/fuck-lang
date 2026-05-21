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
