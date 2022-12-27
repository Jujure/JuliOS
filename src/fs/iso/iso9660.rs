pub const ISO_BLOCK_SIZE: u32 = 2048;

// Twin values structs

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct MultiEndian32 {
    pub le: u32, // Little endian value
    pub be: u32, // Big endian value
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct MultiEndian16 {
    pub le: u16, // Little endian value
    pub be: u16, // Big endian value
}

// Path table structure

#[repr(C, packed)]
#[derive(Copy, Clone)]
struct IsoPathTable {
    idf_len: u8,     // Identifier name length
    ext_size: u8,    // Extended attribute record length
    data_blk: u8,    // File data block index
    parent_dir: u16, // Number of the parent dir
    idf: [u8; 0],    // Directory name, of size Self::idf_len
}

#[allow(dead_code)]
impl IsoPathTable {
    #[allow(unaligned_references)]
    pub fn get_idf(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.idf.as_ptr(), self.idf_len as usize) }
    }
}

// Directory structure

#[allow(dead_code)]
const ISO_MAX_DIR_DEPTH: usize = 8;
const ISO_DATE_LEN: usize = 7;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum IsoFileType {
    HIDDEN = 0x1,    // Hidden file
    ISDIR = 0x2,     // Directory
    ASSOCIAT = 0x4,  // Associated
    USEEXT = 0x8,    //
    USEPERM = 0x10,  //
    MULTIDIR = 0x80, //
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IsoDir {
    pub dir_size: u8,             // Length of directory record
    pub ext_size: u8,             // Length of extended attribute record
    pub data_blk: MultiEndian32,  // File data block index
    pub file_size: MultiEndian32, // File size
    pub date: [u8; ISO_DATE_LEN],
    pub file_type: IsoFileType,

    pub unit_size: u8,
    pub gap_size: u8,

    pub vol_seq: MultiEndian16,

    pub idf_len: u8,  // File name length
    pub idf: [u8; 0], // File name
}

impl IsoDir {
    #[allow(unaligned_references)]
    pub fn get_idf(&self) -> &[u8] {
        let mut len: usize = self.idf_len as usize;
        unsafe {
            let mut idf = core::slice::from_raw_parts(self.idf.as_ptr(), len as usize);
            if len > 2 && idf[len - 2] == b';' && idf[len - 1] == b'1' {
                len -= 2;
                idf = core::slice::from_raw_parts(self.idf.as_ptr(), len as usize);
            }
            idf
        }
    }

    pub fn next_entry(&self) -> &IsoDir {
        crate::utils::ref_raw_offset(self, self.dir_size as isize)
    }

    pub fn matches(&self, path: &str) -> bool {
        self.get_idf() == path.as_bytes()
    }
}

// Primary volume descriptor structure

pub const ISO_PRIM_VOLDESC_BLOCK: u32 = 16;

const ISO_SYSIDF_LEN: usize = 32;
const ISO_VOLIDF_LEN: usize = 32;
const ISO_VOLSET_LEN: usize = 128;
const ISO_PUBIDF_LEN: usize = 128;
const ISO_DPREP_LEN: usize = 128;
const ISO_APP_LEN: usize = 128;
const ISO_CPRFIL_LEN: usize = 37;
const ISO_ABSFIL_LEN: usize = 37;
const ISO_BIBFIL_LEN: usize = 37;
const ISO_LDATE_LEN: usize = 17;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IsoPrimVolDesc {
    pub vol_desc_type: u8,       // Volume descripto type (1)
    pub std_identifier: [u8; 5], // standard identifier ("CD001")
    pub vol_desc_version: u8,    // Volume descriptor version (1)

    pub _unused1: u8,

    pub sys_idf: [u8; ISO_SYSIDF_LEN], // System identifier
    pub vol_idf: [u8; ISO_VOLIDF_LEN], // Volume identifier

    pub _unused2: [u8; 8],

    pub vol_blk_count: MultiEndian32, // Number of logical blocks in the volume

    pub _unused3: [u8; 32],

    pub vol_set_size: MultiEndian16, // The Volume Set size of the volume
    pub vol_seq_num: MultiEndian16,  // The number of the volume in the set
    pub vol_blk_size: MultiEndian16, // The size in bytes of a logical block

    pub path_table_size: MultiEndian32, // Length in bytes of the path table
    pub le_path_table_blk: u32,         // Path table block index little endian
    pub le_opt_path_table_blk: u32,     // Optionnal path table block index little endian
    pub be_path_table_blk: u32,
    pub be_opt_path_table_blk: u32,

    pub root_dir: IsoDir, // Root directory entry

    pub _unused4: [u8; 34 - core::mem::size_of::<IsoDir>()], // Padding

    pub volset_idf: [u8; ISO_VOLSET_LEN], // name of the multiple volume set
    pub pub_idf: [u8; ISO_PUBIDF_LEN],    // Publisher name
    pub dprep_idf: [u8; ISO_DPREP_LEN],   // Data preparer name
    pub app_idf: [u8; ISO_APP_LEN],       // Application name

    pub copyright_file: [u8; ISO_CPRFIL_LEN], // Copyright file name in root dir
    pub abstract_file: [u8; ISO_ABSFIL_LEN],  // Abstract file name in root dir
    pub bibli_file: [u8; ISO_BIBFIL_LEN],     // Bibliograpgic file name in root dir
    pub date_creat: [u8; ISO_LDATE_LEN],      // Creation date
    pub date_modif: [u8; ISO_LDATE_LEN],      // Modification date
    pub date_expir: [u8; ISO_LDATE_LEN],      // Expiration date
    pub date_effect: [u8; ISO_LDATE_LEN],     // Effective date

    pub file_struct_version: u8, // File structure version (1)
}
