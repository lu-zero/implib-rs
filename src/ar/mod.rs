//! A library for encoding/decoding Unix archive files.
//!
//! This library provides utilities necessary to manage [Unix archive
//! files](https://en.wikipedia.org/wiki/Ar_(Unix)) (as generated by the
//! standard `ar` command line utility) abstracted over a reader or writer.
//! This library provides a streaming interface that avoids having to ever load
//! a full archive entry into memory.
//!
//! The API of this crate is meant to be similar to that of the
//! [`tar`](https://crates.io/crates/tar) crate.
//!

mod error;
mod write;

use crate::{bail, ensure, err};
pub use write::GnuBuilder;

// ========================================================================= //

const GLOBAL_HEADER_LEN: usize = 8;
const GLOBAL_HEADER: &[u8; GLOBAL_HEADER_LEN] = b"!<arch>\n";

const GNU_NAME_TABLE_ID: &str = "//";
const GNU_SYMBOL_LOOKUP_TABLE_ID: &str = "/";

// ========================================================================= //

/// Representation of an archive entry header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
    identifier: Vec<u8>,
    mtime: u64,
    uid: u32,
    gid: u32,
    mode: u32,
    size: u64,
}

impl Header {
    /// Creates a header with the given file identifier and size, and all
    /// other fields set to zero.
    pub fn new(identifier: Vec<u8>, size: u64) -> Header {
        Header {
            identifier,
            mtime: 0,
            uid: 0,
            gid: 0,
            mode: 0o644,
            size,
        }
    }

    /// Returns the file identifier.
    pub fn identifier(&self) -> &[u8] {
        &self.identifier
    }

    /// Sets the mode bits for this file.
    pub fn set_mode(&mut self, mode: u32) {
        self.mode = mode;
    }

    /// Returns the length of the file, in bytes.
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Validates the header is somewhat sane against the specification.
    pub fn validate(&self) -> std::io::Result<()> {
        ensure!(
            num_digits(self.mtime, 10) <= 12,
            "MTime `{}` > 12 digits",
            self.mtime
        );
        ensure!(
            num_digits(self.uid, 10) <= 6,
            "UID `{}` > 6 digits",
            self.uid
        );
        ensure!(
            num_digits(self.gid, 10) <= 6,
            "GID `{}` > 6 digits",
            self.gid
        );
        ensure!(
            num_digits(self.mode, 8) <= 8,
            "Mode `{:o}` > 8 octal digits",
            self.mode
        );
        Ok(())
    }
}

#[inline]
fn num_digits<N: Into<u64>>(val: N, radix: u64) -> u64 {
    let mut val = val.into();
    if val == 0 {
        return 1;
    }

    let mut digits = 0;
    while val != 0 {
        val /= radix;
        digits += 1;
    }
    digits
}
