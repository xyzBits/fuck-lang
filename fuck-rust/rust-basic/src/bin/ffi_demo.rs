use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

// ==========================================
// 1. 声明外部 C/C++ 函数的签名 (接口定义层)
// ==========================================
// 告诉 Rust：去 "user32.dll" 里找这个函数
#[link(name = "user32")]
unsafe extern "system" {
    // 对应 C 语言的签名：
    // int MessageBoxW(HWND hWnd, LPCWSTR lpText, LPCWSTR lpCaption, UINT uType);
    fn MessageBoxW(
        hWnd: *mut std::ffi::c_void, // 窗口句柄，没有就传 null
        lpText: *const u16,          // 弹窗内容的宽字符串指针 (UTF-16)
        lpCaption: *const u16,       // 弹窗标题的宽字符串指针 (UTF-16)
        uType: u32,                  // 按钮类型，0 表示 MB_OK
    ) -> i32;
}

// ==========================================
// 2. 编写数据转换辅助函数 (防腐层)
// ==========================================
// Rust 的字符串是 UTF-8，而 Windows API (带 'W' 后缀的) 需要 UTF-16 且以 0 结尾的宽字符串。
// 这个函数负责把 Rust 的 &str 翻译成 C 听得懂的格式。
fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide() // 转换成 UTF-16
        .chain(std::iter::once(0)) // 灵魂一步：在末尾强行补上 '\0' (C语言字符串的结束符)
        .collect()
}

// ==========================================
// 3. 业务调用层
// ==========================================
fn main() {
    // 准备数据
    let text = to_wstring("你好！这是一个极其干净的 Rust FFI 弹窗。");
    let caption = to_wstring("顺德老兵的 FFI 试炼");

    // FFI 调用的核心：越过安全边界
    unsafe {
        MessageBoxW(
            ptr::null_mut(),  // 相当于 C 的 NULL
            text.as_ptr(),    // 获取原始内存指针
            caption.as_ptr(), // 获取原始内存指针
            0,                // 0x00000000L 代表 MB_OK (只有一个确定按钮)
        );
    }

    println!("弹窗已关闭，程序继续向下执行...");
}
