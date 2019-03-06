use hactool_sys as hac;
use std::ffi::CString;
use std::path::PathBuf;
use std::os::unix::ffi::OsStrExt;
use super::*;

#[derive(Debug, Copy, Clone)]
pub struct LibnxError;
pub struct SdkPath(pub PathBuf);

pub fn mount_sdk_fs(_fs: &str) -> PathBuf {
    PathBuf::new()
}

impl SdkPath {
    pub fn mount(&self) -> PathBuf {
        self.0.clone()
    }
}

pub fn get_tid_nca(_tid: TitleId) -> Result<SdkPath, LibnxError> {
    Ok(SdkPath("/home/leo60228/fuji/tests/hbmenu.nca".into()))
}

pub fn get_hactool_settings() -> hac::hactool_settings_t {
    let mut path = dirs::home_dir().unwrap();
    path.push(".switch/prod.keys");
    let c_path = CString::new(path.as_os_str().as_bytes()).unwrap();
    let c_file = unsafe { libc::fopen(c_path.as_c_str().as_ptr(), CStr::from_bytes_with_nul(b"r\0").unwrap().as_ptr()) };
    if c_file.is_null() {
        panic!("Keys must exist when running on PC!");
    }
    let mut settings = hac::hactool_settings_t::default();
    unsafe { hac::extkeys_initialize_settings(&mut settings as *mut _, c_file); }
    settings
}

#[cfg(test)]
mod test {
    use crate::util::*;

    #[test]
    fn tid_nca_exists() {
        let path = get_tid_nca(TitleId(0x1234)).unwrap().0;
        assert_eq!(path.exists(), true);
    }
}
