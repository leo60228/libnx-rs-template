use hactool_sys as hac;
use std::ffi::OsStr;
use std::ffi::CStr;
use std::ffi::CString;
use std::path::Path;
use std::path::PathBuf;
use std::os::unix::ffi::OsStrExt;
use std::os::raw::c_char;
use super::*;
use libnx_rs::*;

pub use libnx_rs::LibnxError;

pub struct SdkPath(pub PathBuf);

const STORAGE_ID_NAND_USER: u32 = 4;

pub fn mount_sdk_fs(fs: &str) -> PathBuf {
    let mut bis_dev = libnx::FsFileSystem::default();
    let (bis_id, name, root) = match fs {
        "SystemContent" => (31, "System", PathBuf::from("System:/Contents/")),
        "UserContent" => (30, "User", PathBuf::from("User:/Contents/")),
        _ => panic!("FS not implemented!")
    };
    unsafe { libnx::fsOpenBisFileSystem(&mut bis_dev as *mut _, bis_id, &(0 as c_char) as *const _); }
    unsafe { libnx::fsdevMountDevice(CString::new(name).unwrap().as_ptr(), bis_dev); }
    root
}

impl SdkPath {
    pub fn mount(&self) -> PathBuf {
        let mut segments: Vec<_> = self.0.ancestors().filter_map(|e| e.file_name()).map(|e| e.to_str().unwrap()).collect();
        let mut fs = segments.pop().unwrap()[1..].to_string();
        fs.pop();
        segments.reverse();
        
        let mut libnx_path = mount_sdk_fs(&fs);
        for segment in &segments {
            libnx_path.push(segment);
        }
        
        libnx_path
    }
}

pub fn get_tid_nca(tid: TitleId) -> Result<SdkPath, LibnxError> {
    let mut fs_buf: [u8; libnx::FS_MAX_PATH as usize] = [0; libnx::FS_MAX_PATH as usize];

    match unsafe { libnx::lrInitialize() } {
        0 => {},
        e => return Err(LibnxError::from_raw(e))
    };

    let mut resolver = libnx::LrLocationResolver::default();
    match unsafe { libnx::lrOpenLocationResolver(STORAGE_ID_NAND_USER, &mut resolver as *mut _) } { //TODO: sd
        0 => {},
        e => return Err(LibnxError::from_raw(e))
    };

    match unsafe { libnx::lrLrResolveProgramPath(&mut resolver as *mut _, tid.0, &mut fs_buf[0] as *mut _) } {
        0 => {},
        e => return Err(LibnxError::from_raw(e))
    };

    Ok(SdkPath(OsStr::from_bytes(CStr::from_bytes_with_nul(&fs_buf).unwrap().to_bytes()).to_os_string().into()))
}

pub fn get_hactool_settings() -> hac::hactool_settings_t {
    let prod = Path::new("sdmc:/switch/prod.keys");
    let title = Path::new("sdmc:/switch/title.keys");
    let c_prod_path = CString::new(prod.as_os_str().as_bytes()).unwrap();
    let c_prod_file = unsafe { libc::fopen(c_prod_path.as_c_str().as_ptr(), CStr::from_bytes_with_nul(b"r\0").unwrap().as_ptr()) };
    let c_title_path = CString::new(title.as_os_str().as_bytes()).unwrap();
    let c_title_file = unsafe { libc::fopen(c_title_path.as_c_str().as_ptr(), CStr::from_bytes_with_nul(b"r\0").unwrap().as_ptr()) };
    if c_prod_file.is_null() || c_title_file.is_null() {
        panic!("Automatic key derivation isn't currently implemented!");
    }
    let mut settings = hac::hactool_settings_t::default();
    unsafe { hac::extkeys_initialize_settings(&mut settings as *mut _, c_prod_file); }
    unsafe { hac::extkeys_parse_titlekeys(&mut settings as *mut _, c_title_file); }
    settings
}
