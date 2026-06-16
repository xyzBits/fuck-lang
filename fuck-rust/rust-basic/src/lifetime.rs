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



fn first_word(s: &str) -> usize {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return i;
        }
    }

    s.len()
}

#[test]
fn test_first_word() {
    let mut s = "hello world".to_string();
    let word = first_word(&s);
    s.clear();
    println!("the first word is at index: {}", word);

    // println!("{}", &s[..word]);
}

fn first_word2(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

    &s[..]
}

#[test]
fn test_first_word2() {
    // slice 包含两个信息，指向起始位置的指针
    let mut s = "hello world".to_string();

    let word = first_word2(&s);

    // s.clear();
    println!("the first word is at index: {}", word);
    // 字符串字面量本身就是切片，指向二进制程序特定内存位置的切片，这也是字符串字面量不可变的原因
}