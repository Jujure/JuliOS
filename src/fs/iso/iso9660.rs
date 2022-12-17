#[repr(C, packed)]
pub struct IsoPathTable {
    idf_len: u8, // Identifier name length
    ext_size: u8, // Extended attribute record length
    data_blk: u8, // File data block index
    parent_dir: u16, // Number of the parent dir
    // idf: [char] // Directory name, of size Self::idf_len
}

impl IsoPathTable {
    pub fn from(mapping: &u8) -> &Self {
        let ptr: *const u8 = mapping;
        let path_table_ptr: *const IsoPathTable = ptr as *const IsoPathTable;
        unsafe {
            &*path_table_ptr
        }
    }

    pub fn get_idf(&self) -> *const char {
        let ptr: *const IsoPathTable = self;
        unsafe {
            ptr.offset(1) as *const char
        }
    }
}