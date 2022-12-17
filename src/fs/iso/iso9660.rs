#[repr(C, packed)]
pub struct IsoPathTable {
    idf_len: u8, // Identifier name length
    ext_size: u8, // Extended attribute record length
    data_blk: u8, // File data block index
    parent_dir: u16, // Number of the parent dir
    idf: [char; 0] // Directory name, of size Self::idf_len
}

impl IsoPathTable {
    pub fn from(mapping: &u8) -> &Self {
        let ptr: *const u8 = mapping;
        let path_table_ptr: *const IsoPathTable = ptr as *const IsoPathTable;
        unsafe {
            &*path_table_ptr
        }
    }

    #[allow(unaligned_references)]
    pub fn get_idf(&self) -> &[char] {
        unsafe {
            core::slice::from_raw_parts(self.idf.as_ptr(), self.idf_len as usize)
        }
    }
}