pub mod mutex;
pub mod serialize;

pub use mutex::AsyncMutex;
pub use serialize::unserialize;

pub fn ref_offset<T>(r: &T, off: isize) -> &T {
    let ref_ptr: *const T = r;
    unsafe {
        return &*ref_ptr.offset(off);
    }
}

pub fn ref_raw_offset<T>(r: &T, off: isize) -> &T {
    let ref_ptr: *const T = r;
    unsafe {
        return &*ref_ptr.cast::<u8>().offset(off).cast::<T>();
    }
}
