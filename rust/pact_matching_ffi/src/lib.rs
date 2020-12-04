//! A crate exposing the `pact_matching` API to other languages
//! via a C Foreign Function Interface.

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]

pub mod error;
pub mod log;
pub mod models;
pub(crate) mod util;

use crate::util::*;
use libc::c_char;
use models::message::Message;
use pact_matching as pm;

pub use pact_matching::Mismatch;

ffi_fn! {
    /// Match a pair of messages, producing a collection of mismatches,
    /// which is empty if the two messages matched.
    fn match_message(msg_1: *const Message, msg_2: *const Message) -> *const Mismatches {
        let msg_1 = as_ref!(msg_1);
        let msg_2 = as_ref!(msg_2);
        let mismatches = Mismatches(pm::match_message(msg_1, msg_2));

        ptr::raw_to(mismatches) as *const Mismatches
    } {
        ptr::null_to::<Mismatches>() as *const Mismatches
    }
}

ffi_fn! {
    /// Get an iterator over mismatches.
    fn mismatches_get_iter(mismatches: *const Mismatches) -> *mut MismatchesIterator {
        let mismatches = as_ref!(mismatches);
        let iter = MismatchesIterator { current: 0, mismatches };
        ptr::raw_to(iter)
    } {
        ptr::null_mut_to::<MismatchesIterator>()
    }
}

ffi_fn! {
    /// Delete mismatches
    fn mismatches_delete(mismatches: *const Mismatches) {
        ptr::drop_raw(mismatches as *mut Mismatches);
    }
}

ffi_fn! {
    /// Get the next mismatch from a mismatches iterator.
    ///
    /// Returns a null pointer if no mismatches remain.
    fn mismatches_iter_next(iter: *mut MismatchesIterator) -> *const Mismatch {
        let iter = as_mut!(iter);
        let mismatches = as_ref!(iter.mismatches);
        let index = iter.next();
        let mismatch = mismatches
            .0
            .get(index)
            .ok_or(anyhow::anyhow!("iter past the end of mismatches"))?;
       mismatch as *const Mismatch
    } {
        ptr::null_to::<Mismatch>()
    }
}

ffi_fn! {
    /// Delete a mismatches iterator when you're done with it.
    fn mismatches_iter_delete(iter: *mut MismatchesIterator) {
        ptr::drop_raw(iter);
    }
}

ffi_fn! {
    /// Get a JSON representation of the mismatch.
    fn mismatch_to_json(mismatch: *const Mismatch) -> *const c_char {
        let mismatch = as_ref!(mismatch);
        let json = mismatch.to_json().to_string();
        string::to_c(&json)? as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Get the type of a mismatch.
    fn mismatch_type(mismatch: *const Mismatch) -> *const c_char {
        let mismatch = as_ref!(mismatch);
        let t = mismatch.mismatch_type();
        string::to_c(&t)? as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Get a summary of a mismatch.
    fn mismatch_summary(mismatch: *const Mismatch) -> *const c_char {
        let mismatch = as_ref!(mismatch);
        let summary = mismatch.summary();
        string::to_c(&summary)? as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Get a description of a mismatch.
    fn mismatch_description(mismatch: *const Mismatch) -> *const c_char {
        let mismatch = as_ref!(mismatch);
        let description = mismatch.description();
        string::to_c(&description)? as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Get an ANSI-compatible description of a mismatch.
    fn mismatch_ansi_description(mismatch: *const Mismatch) -> *const c_char {
        let mismatch = as_ref!(mismatch);
        let ansi_description = mismatch.ansi_description();
        string::to_c(&ansi_description)? as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

/// A collection of mismatches from a matching comparison.
#[allow(missing_copy_implementations)]
#[allow(missing_debug_implementations)]
pub struct Mismatches(Vec<Mismatch>);

/// An iterator over mismatches.
#[allow(missing_copy_implementations)]
#[allow(missing_debug_implementations)]
pub struct MismatchesIterator {
    current: usize,
    mismatches: *const Mismatches,
}

impl MismatchesIterator {
    fn next(&mut self) -> usize {
        let idx = self.current;
        self.current += 1;
        idx
    }
}
