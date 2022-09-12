use std::path::{Path, PathBuf};

///	make_native_path - on WIN32, change / to \ in the path
///
///	This effectively undoes canonicalize_path.
///
///	This is required because WIN32 COPY is an internal CMD.EXE
///	command and doesn't process forward slashes in the same way
///	as external commands.  Quoting the first argument to COPY
///	does not convert forward to backward slashes, but COPY does
///	properly process quoted forward slashes in the second argument.
///
///	COPY works with quoted forward slashes in the first argument
///	only if the current directory is the same as the directory
///	of the first argument.
pub fn make_native_path(path: &Path) -> PathBuf {
    // UTF-8 or bust, the year is 2022.

    if cfg!(windows) {
        let path = path.to_str().unwrap_or("");
        Path::new(&path.replace("/", "\\")).to_owned()
    } else {
        path.to_owned()
    }
}
