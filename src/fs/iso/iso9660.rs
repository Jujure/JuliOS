const ISO_BLOCK_SIZE: usize = 2048;

// Twin values structs

#[repr(C, packed)]
pub struct MultiEndian32 {
    le: u32, // Little endian value
    be: u32, // Big endian value
}

#[repr(C, packed)]
pub struct MultiEndian16 {
    le: u16, // Little endian value
    be: u16, // Big endian value
}


// Path table structure

#[repr(C, packed)]
struct IsoPathTable {
    idf_len: u8, // Identifier name length
    ext_size: u8, // Extended attribute record length
    data_blk: u8, // File data block index
    parent_dir: u16, // Number of the parent dir
    idf: [u8; 0] // Directory name, of size Self::idf_len
}

impl IsoPathTable {
    #[allow(unaligned_references)]
    pub fn get_idf(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.idf.as_ptr(), self.idf_len as usize)
        }
    }
}


// Directory structure

const ISO_DATE_LEN: usize = 7;

#[repr(u8)]
enum IsoFileType {
    HIDDEN = 0x1, // Hidden file
    ISDIR = 0x2, // Directory
    ASSOCIAT = 0x4, // Associated
    USEEXT = 0x8, //
    USEPERM = 0x10, //
    MULTIDIR = 0x80 // 
}

#[repr(C, packed)]
pub struct IsoDir {
    dir_size: u8, // Length of directory record
    ext_size: u8, // Length of extended attribute record
    data_blk: MultiEndian32, // File data block index
    file_size: MultiEndian32, // File size
    date: [u8; ISO_DATE_LEN],
    file_type: IsoFileType,

    unit_size: u8,
    gap_size: u8,

    vol_seq: MultiEndian16,

    idf_len: u8, // File name length
    idf: [u8; 0], // File name
}

impl IsoDir {
    #[allow(unaligned_references)]
    pub fn get_idf(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self.idf.as_ptr(), self.idf_len as usize)
        }
    }
}


// Primary volume descriptor structure

const ISO_PRIM_VOLDESC_BLOCK: usize = 16;

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
pub struct IsoPrimVolDesc {
    vol_desc_type: u8, // Volume descripto type (1)
    std_identifier: [u8; 5], // standard identifier ("CD001")
    vol_desc_version: u8, // Volume descriptor version (1)

    _unused1: u8,

    sys_idf: [u8; ISO_SYSIDF_LEN], // System identifier
    vol_idf: [u8; ISO_VOLIDF_LEN], // Volume identifier

    _unused2: [u8; 8],

    vol_blk_count: MultiEndian32, // Number of logical blocks in the volume

    _unused3: [u8; 32],

    vol_set_size: MultiEndian16, // The Volume Set size of the volume
    vol_seq_num: MultiEndian16, // The number of the volume in the set
    vol_blk_size: MultiEndian16, // The size in bytes of a logical block

    path_table_size: MultiEndian32, // Length in bytes of the path table
    le_path_table_blk: u32, // Path table block index little endian
    le_opt_path_table_blk: u32, // Optionnal path table block index little endian
    be_path_table_blk: u32,
    be_opt_path_table_blk: u32,

    root_dir: IsoDir, // Root directory entry

    _unused4: [u8; 34 - core::mem::size_of::<IsoDir>()], // Padding

    volset_idf: [u8; ISO_VOLSET_LEN], // name of the multiple volume set
    pub_idf: [u8; ISO_PUBIDF_LEN], // Publisher name
    dprep_idf: [u8; ISO_DPREP_LEN], // Data preparer name
    app_idf: [u8; ISO_APP_LEN], // Application name

    copyright_file: [u8; ISO_CPRFIL_LEN], // Copyright file name in root dir
    abstract_file: [u8; ISO_ABSFIL_LEN], // Abstract file name in root dir
    bibli_file: [u8; ISO_BIBFIL_LEN], // Bibliograpgic file name in root dir
    date_creat: [u8; ISO_LDATE_LEN], // Creation date
    date_modif: [u8; ISO_LDATE_LEN], // Modification date
    date_expir: [u8; ISO_LDATE_LEN], // Expiration date
    date_effect: [u8; ISO_LDATE_LEN], // Effective date

    file_struct_version: u8, // File structure version (1)
}