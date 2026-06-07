use std::thread;
use std::time::Duration;
use my_macros::time_it;



#[time_it]
fn process_matching_engine() -> i32 {
    println!("Hello, world starting");

    thread::sleep(Duration::from_secs(10));
    println!("Hello, world starting 2");

    return 42;
}
#[test]
fn test_time_it_macros() {
    process_matching_engine();
}