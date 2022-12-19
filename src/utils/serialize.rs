pub fn unserialize<T>(mapping: &u8) -> &T {
    let ptr: *const u8 = mapping;
    let path_table_ptr: *const T = ptr as *const T;
    unsafe {
        &*path_table_ptr
    }
}