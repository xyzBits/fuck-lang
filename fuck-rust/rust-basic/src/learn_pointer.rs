use std::pin::Pin;
use std::rc::Rc;
use futures::executor::block_on;
use futures::FutureExt;

#[test]
fn test_raw_pointers() {
    let mut data = 42;

    // 创建不可变裸指针和可变裸指针，这步是安全的
    let ptr_imm: *const i32 = &data;
    let ptr_mut: *mut i32 = &mut data;

    // 只要你想操作内存，必须进入 unsafe 结界
    unsafe {
        *ptr_mut = 100; // 像 c 语言一样粗暴地修改
        assert_eq!(*ptr_imm, 100);
    }

    unsafe {
        println!("{:?}", *ptr_imm);
    }
}

#[test]
fn test_reference() {
    let mut data = 42;

    let r1 = &data;
    let r2 = &data;

    assert_eq!(*r1, 42);
    assert_eq!(*r2, 42);
    // r1 r2 的作用域在这里结束

    // 可变引用必须绝对独占
    let r_mut = &mut data;
    *r_mut = 100;

    assert_eq!(data, 100);
}

#[test]
fn test_box() {
    let mut heap_data = Box::new(42);

    *heap_data = 100;

    assert_eq!(*heap_data, 100);
}



#[test]
fn test_rc_and_get_mut() {
    let mut shared_ptr = Rc::new(42);

    if let Some(mut_ref) = Rc::get_mut(&mut shared_ptr) {
        *mut_ref = 100;
    }

    assert_eq!(*shared_ptr, 100);

    let _other_shared_ptr = Rc::clone(&shared_ptr);

    let is_none = Rc::get_mut(&mut shared_ptr).is_none();

    println!("{}", is_none);
}

async fn heavy_task() -> i32 {
    let data = "机密数据".to_string();
    let _ref = &data;

    42
}

fn create_task() -> Pin<Box<dyn Future<Output = i32>>> {
    let task = heavy_task();

    Box::pin(task)
}

#[test]
fn test_pin_box() {
    let pinned_heavy_task = create_task();

    let another_key = pinned_heavy_task;
    block_on(another_key);
}


#[test]
fn test_ref_magic() {
    let my_box = Some(String::from("核心机密"));

    match my_box {
        Some(ref inner_str) => {
            println!("看一眼金条 {}", inner_str);
        }

        None => {}
    }


    match &my_box {
        Some(inner_str) => {
            println!("看一眼金条 {}", inner_str);
        }
        None => {}
    }




    let mut my_opt = Some(42);

    match &mut my_opt {
        Some(val) => {
            *val += 100;
        }
        None => {}
    }

    println!("{:?}", my_opt);
}