use cfg_if::cfg_if;
use hactool_sys as hac;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

cfg_if! {
    if #[cfg(target_os = "horizon")] {
        mod nx;
        pub use nx::*;
    } else {
        mod pc;
        pub use pc::*;
    }
}

pub struct TitleId(pub u64);

pub fn extract_nca(path: &Path, dir: &Path) -> Result<(), String> {
    let mut tool_ctx = hac::hactool_ctx_t::default();
    let mut nca_ctx = hac::nca_ctx_t::default();
    unsafe {
        hac::nca_init(&mut nca_ctx as *mut _);
    }
    nca_ctx.tool_ctx = &mut tool_ctx as *mut _;
    nca_ctx.is_cli_target = 0;
    tool_ctx.file_type = hac::hactool_file_type::FILETYPE_NCA;
    tool_ctx.action = hac::ACTION_EXTRACT;
    tool_ctx.settings = get_hactool_settings();
    tool_ctx.settings.romfs_dir_path.enabled = 1;

    let c_dir = match CString::new(dir.as_os_str().as_bytes()) {
        Ok(c_string) => c_string,
        Err(_) => return Err("Couldn't create CString for dir!".to_string()),
    };

    unsafe {
        hac::filepath_set(
            &mut tool_ctx.settings.romfs_dir_path.path as *mut _,
            c_dir.as_c_str().as_ptr(),
        );
    }

    let c_path = match CString::new(path.as_os_str().as_bytes()) {
        Ok(c_string) => c_string,
        Err(_) => return Err("Couldn't create CString for path!".to_string()),
    };

    tool_ctx.file = unsafe {
        libc::fopen(
            c_path.as_c_str().as_ptr(),
            CStr::from_bytes_with_nul(b"r\0").unwrap().as_ptr(),
        )
    };

    if tool_ctx.file.is_null() {
        return Err("Couldn't open NCA for reading!".to_string());
    }

    nca_ctx.file = tool_ctx.file;

    unsafe {
        hac::nca_process(&mut nca_ctx as *mut _);
    }

    unsafe {
        libc::fclose(tool_ctx.file);
    }

    if !nca_ctx
        .section_contexts
        .iter()
        .any(|e| e.is_present != 0 && e.type_ == hac::nca_section_type::ROMFS)
    {
        unsafe {
            hac::nca_free_section_contexts(&mut nca_ctx as *mut _);
        }
        return Err("RomFS missing from NCA!".to_string());
    }

    unsafe {
        hac::nca_free_section_contexts(&mut nca_ctx as *mut _);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use mktemp::Temp;

    #[test]
    fn test_extract_nca() {
        let nca = get_tid_nca(TitleId(0x0)).unwrap().mount();
        let out_dir = Temp::new_dir().unwrap();
        let out_path = out_dir.to_path_buf();
        extract_nca(&nca, &out_path).unwrap();
    }
}
