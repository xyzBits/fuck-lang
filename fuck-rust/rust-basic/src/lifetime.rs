#![allow(clippy::all)]

use std::collections::BTreeMap;

#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn test_struct_close_copy() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = p1;

    println!("{}", p1.x);
    println!("{}", p2.x);
    println!("{}", p1.y);
}

#[derive(Debug, PartialEq, Eq)]
pub enum MyError {
    ParseFailed,
    Empty,
}

pub fn parse_nonempty(s: &str) -> Result<i32, MyError> {
    if s.is_empty() {
        return Err(MyError::Empty);
    }

    let num = s.parse::<i32>().map_err(|_| MyError::ParseFailed)?;

    Ok(num)
}
// 使用 box 的两个场景
// 1. 递归类型，不用 box 算不出大小，用 box 把递归部分变成固定大小的指针
// 2. dyn trait 对象，动态的对象

fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

#[test]
fn test_largest() {
    assert_eq!(largest(&[1, 2, 3, 4]), &4);

    let mut map = BTreeMap::new();

    map.insert(1, "hello");
    map.insert(2, "world");
    map.insert(3, "rust");
    println!("{:?}", map.last_key_value().unwrap());
    let result = map.last_key_value().unwrap();
}

fn max_ref<T: PartialOrd>(list: &[T]) -> Option<&T> {
    if list.is_empty() {
        return None;
    }

    let mut max = &list[0];

    for item in list {
        if item > max {
            max = item;
        }
    }

    Some(max)
}

struct Order {
    id: u64,
    price: u64,
    qty: u64,
}

struct OrderBook {
    bids: BTreeMap<u64, Order>,
}

impl OrderBook {
    fn best_bid(&self) -> Option<&Order> {
        self.bids.last_key_value().map(|(_price, order)| order)
    }
}

async fn hello_self_ref() {
    let mut buffer = [0u8; 1024];
    let mut ptr = &buffer[0];
    do_io().await;

    println!("data = {}", *ptr);
}

async fn do_io() {}

#[test]
fn test_io() {
    hello_self_ref();
}
