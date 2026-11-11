use std::error::Error as StdError;
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Dir,
    Symlink {
        /// raw `read_link` result. May be relative to the symlink's directory.
        target: PathBuf,
        /// `true` if the target does not resolve for any reason
        /// (missing, permission denied, symlink loop)
        broken: bool,
    },
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: PathBuf,
    pub kind: EntryKind,
    /// raw `st_size` from `lstat`. Only meaningful for regular files;
    /// for symlinks it is the length of the target path
    pub size: u64,
    /// `st_mode & 0o7777`
    pub permissions: u32,
    pub mtime: SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryOp {
    ReadEntry,
    Metadata,
    ReadLink,
}

#[derive(Debug)]
pub struct EntryError {
    pub op: EntryOp,
    /// the entry the operation was about. For `EntryOp::ReadEntry` the
    /// iterator gives no name, so this is the directory being read.
    pub path: PathBuf,
    pub source: io::Error,
}

#[derive(Debug)]
pub struct Listing {
    pub entries: Vec<Entry>,
    pub errors: Vec<EntryError>,
}

#[derive(Debug)]
pub enum Error {
    ReadDir { path: PathBuf, source: io::Error },
}

impl Entry {
    /// final path component. Falls back to the full path in the (practically
    /// impossible) case where it has none, so it never panics.
    pub fn name(&self) -> &OsStr {
        self.path.file_name().unwrap_or(self.path.as_os_str())
    }
}

/// policy: an entry that vanished between `readdir` and `lstat`/`readlink`
/// is skipped silently, not reported as an error.
fn vanished(op: EntryOp, err: &io::Error) -> bool {
    op != EntryOp::ReadEntry && err.kind() == io::ErrorKind::NotFound
}

impl fmt::Display for EntryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EntryOp::ReadEntry => "failed to read directory entry",
            EntryOp::Metadata => "failed to read metadata",
            EntryOp::ReadLink => "failed to read symlink target",
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ReadDir { path, .. } => {
                write!(f, "failed to read directory {}", path.display())
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::ReadDir { source, .. } => Some(source),
        }
    }
}

impl fmt::Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.op, self.path.display())
    }
}

impl StdError for EntryError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}
