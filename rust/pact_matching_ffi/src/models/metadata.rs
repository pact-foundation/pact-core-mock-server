//! The `Metadata` type and operations on it.

/*
use crate::ffi;
use crate::util::string;
use libc::c_char;
use std::collections::HashMap;
use std::ffi::CString;

/// FFI structure representing a list of (key, value) pairs.
/// It is an array with a number of elements equal to `length`.
///
/// This structure should not be mutated.
#[allow(missing_copy_implementations)]
#[repr(C)]
#[derive(Debug)]
pub struct MetadataList {
    /// pointer to array of key, value pairs
    pub list: *const MetadataKV,
    /// number of elements in `list`
    pub length: usize,
    /// private, tracks allocated capacity of the underlying Vec
    capacity: usize,
}

/// FFI structure representing a (key, value) pair.
///
/// This structure should not be mutated.
#[allow(missing_copy_implementations)]
#[repr(C)]
#[derive(Debug)]
pub struct MetadataKV {
    /// null terminated string containing the key
    pub key: *const c_char,
    /// null terminated string containing the value
    pub value: *const c_char,
}

/// Create and leak a MetadataList.  Must be passed back to
/// impl_metadata_list_delete to clean up memory.
fn into_leaked_metadata_list(
    metadata: &HashMap<String, String>,
) -> Result<*const MetadataList, anyhow::Error> {
    let mut list = Vec::with_capacity(metadata.len());

    // First check all the strings for embedded null.
    // This prevents leaking memory in the case where
    // an error occurs after some strings were intentionally
    // leaked, but before they can be passed to C.
    for (k, v) in metadata.iter() {
        if k.find(|c| c == '\0').is_some()
            || v.find(|c| c == '\0').is_some()
        {
            anyhow::bail!(
                "Found embedded null in \
                          a (key, value) pair: ('{}', '{}')",
                k,
                v
            );
        }
    }

    for (k, v) in metadata.iter() {
        // It is safe to unwrap, since the strings were already
        // checked for embedded nulls.
        let kv = MetadataKV {
            key: string::into_leaked_cstring(k.as_ref()).unwrap(),
            value: string::into_leaked_cstring(v.as_ref()).unwrap(),
        };

        list.push(kv);
    }

    let metadata_list = MetadataList {
        list: list.as_ptr(),
        length: list.len(),
        capacity: list.capacity(),
    };

    std::mem::forget(list);

    let output = Box::new(metadata_list);

    Ok(Box::into_raw(output))
}

/// Manually delete a MetadataList.
/// Returns all leaked memory into Rust structures, which will
/// be automatically cleaned up on Drop.
fn impl_metadata_list_delete(ptr: *const MetadataList) {
    let metadata_list =
        unsafe { Box::from_raw(ptr as *mut MetadataList) };

    let list = unsafe {
        Vec::from_raw_parts(
            metadata_list.list as *mut MetadataKV,
            metadata_list.length,
            metadata_list.capacity,
        )
    };

    for kv in list {
        let _k = unsafe { CString::from_raw(kv.key as *mut c_char) };
        let _v = unsafe { CString::from_raw(kv.value as *mut c_char) };
    }
}

/// Delete a MetadataList previously returned by this FFI.
///
/// It is explicitly allowed to pass a null pointer to this function;
/// in that case the function will do nothing.
#[no_mangle]
pub extern "C" fn metadata_list_delete(list: *const MetadataList) {
    ffi! {
        name: "metadata_list_delete",
        params: [list],
        op: {
            if list.is_null() {
                return Ok(());
            }

            impl_metadata_list_delete(list);
            Ok(())
        },
        fail: {
        }
    }
}
*/
