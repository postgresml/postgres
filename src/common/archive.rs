use libc::c_char;

use std::ffi::CString;
use std::path::Path;

use crate::port::path::make_native_path;
use crate::safe_cstr;

/// BuildRestoreCommand
///
/// Builds a restore command to retrieve a file from WAL archives, replacing
/// the supported aliases with values supplied by the caller as defined by
/// the GUC parameter restore_command: xlogpath for %p, xlogfname for %f and
/// lastRestartPointFname for %r.
///
/// The result is a palloc'd string for the restore command built.  The
/// caller is responsible for freeing it.  If any of the required arguments
/// is NULL and that the corresponding alias is found in the command given
/// by the caller, then NULL is returned.
#[no_mangle]
#[allow(non_snake_case)]
extern "C" fn BuildRestoreCommand(
    restore_command: *const c_char,
    xlog_path: *const c_char,
    xlog_fname: *const c_char,
    last_restart_point_fname: *const c_char,
) -> *const c_char {
    match build_restore_command(
        &safe_cstr(restore_command).unwrap_or(String::from("")),
        Path::new(&safe_cstr(xlog_path).unwrap_or(String::from(""))),
        &safe_cstr(xlog_fname).unwrap_or(String::from("")),
        &safe_cstr(last_restart_point_fname).unwrap_or(String::from("")),
    ) {
        Some(restore_command) => {
            let restore_command = CString::new(restore_command).unwrap();
            restore_command.as_ptr()
        }

        None => std::ptr::null(),
    }
}

pub fn build_restore_command(
    restore_command: &str,
    xlog_path: &Path,
    xlog_fname: &str,
    last_restart_point_fname: &str,
) -> Option<String> {
    let command = restore_command.to_string();
    let xlog_path = make_native_path(xlog_path);

    if command.contains("%p") && xlog_path.to_str().unwrap().is_empty() {
        None
    } else if command.contains("%f") && xlog_fname.is_empty() {
        None
    } else if command.contains("%r") && last_restart_point_fname.is_empty() {
        None
    } else {
        Some(
            command
                .replace("%p", xlog_path.to_str().unwrap())
                .replace("%f", xlog_fname)
                .replace("%r", last_restart_point_fname)
                .replace("%%", "%"),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_restore_command() {
        let restore_command = "pgbackrest restore --stanza main \"%p\" %f";

        let restore_command = build_restore_command(
            restore_command,
            &Path::new("/var/lib/postgresql/14/main/pg_wal"),
            "0123456",
            "",
        )
        .unwrap();

        assert_eq!(
            restore_command,
            "pgbackrest restore --stanza main \"/var/lib/postgresql/14/main/pg_wal\" 0123456"
        );
    }

    #[test]
    fn test_build_restore_command_no_xlog_path() {
        let restore_command = "pgbackrest restore --stanza main \"%p\" %f";
        let restore_command = build_restore_command(
            restore_command,
            &Path::new("/var/lib/postgresql/14/main/pg_wal"),
            "",
            "",
        );

        assert_eq!(restore_command, None);
    }
}
