extern crate libc;

#[cfg(feature = "pg_rewind")]
use libc::c_char;
#[cfg(feature = "pg_rewind")]
use std::ffi::CStr;

use std::fs;
#[cfg(feature = "pg_rewind")]
use std::os::unix::io::IntoRawFd;
use std::path::Path;
use std::process;

use crate::common::archive::build_restore_command;
use crate::common::logging::pg_fatal;
use crate::include::access::xlog_internal::XLOGDIR;

pub fn restore_archived_file(
    path: &str,
    xlog_fname: &str,
    expected_size: u64,
    restore_command: &str,
) -> fs::File {
    let xlog_path = format!("{}/{}/{}", XLOGDIR, path, xlog_fname);
    let xlog_path = Path::new(&xlog_path);
    let xlog_restore_command =
        match build_restore_command(restore_command, &xlog_path, xlog_fname, "") {
            Some(xlog_restore_command) => xlog_restore_command,
            None => pg_fatal("cannot use restore_command with %r placeholder"),
        };

    // Run command
    match process::Command::new(xlog_restore_command).output() {
        Ok(output) => match output.status.code() {
            Some(0) => (),
            Some(_) => pg_fatal(&format!(
                "could not restore file \"{}\" from archive",
                xlog_path.display()
            )),
            None => pg_fatal("restore command failed: terminated by signal"),
        },
        Err(err) => pg_fatal(&format!("restore command failed: {:?}", err)),
    };

    match fs::metadata(&xlog_path) {
        Ok(metadata) => {
            if metadata.len() != expected_size {
                pg_fatal(&format!(
                    "unexpected file size for \"{}\": {} instead of {}",
                    xlog_path.display(),
                    metadata.len(),
                    expected_size
                ));
            }
        }
        Err(err) => pg_fatal(&format!(
            "could not stat file \"{}\": {:?}",
            xlog_path.display(),
            err
        )),
    };

    match fs::File::open(&xlog_path) {
        Ok(file) => file,
        Err(err) => pg_fatal(&format!(
            "could not open file \"{}\" restored from archive: {:?}",
            xlog_path.display(),
            err
        )),
    }
}

#[cfg(feature = "pg_rewind")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn RestoreArchivedFile(
    path: *const c_char,
    xlog_fname: *const c_char,
    expected_size: libc::size_t,
    restore_command: *const c_char,
) -> libc::c_int {
    let file = restore_archived_file(
        unsafe { CStr::from_ptr(path).to_str().unwrap() },
        unsafe { CStr::from_ptr(xlog_fname).to_str().unwrap() },
        expected_size.try_into().unwrap(),
        unsafe { CStr::from_ptr(restore_command).to_str().unwrap() },
    );

    file.into_raw_fd()
}
