pub fn unserialize<T>(ptr: *const u8) -> &'static T {
    let path_table_ptr: *const T = ptr as *const T;
    unsafe { &*path_table_ptr }
}
