use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

static DATA: AtomicI32 = AtomicI32::new(0);
static READY: AtomicBool = AtomicBool::new(false);


fn producer() {
    DATA.store(42, Ordering::Relaxed);// 无 barrier 
    READY.store(true, Ordering::Release);// release 写 = store barrier 
}


fn consumer() {
    if READY.load(Ordering::Acquire) {
        
        let v = DATA.load(Ordering::Relaxed);// 
        println!("{}", v);
    }
}