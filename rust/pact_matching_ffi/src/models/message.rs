//! The Pact `Message` type, including associated matching rules and provider states.

use crate::ffi;
use crate::models::pact_specification::PactSpecification;
use crate::util::*;
use anyhow::Context;
use libc::{c_char, c_int, c_uint, EXIT_FAILURE, EXIT_SUCCESS};
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;

// Necessary to make 'cbindgen' generate an opaque struct on the C side.
pub use pact_matching::models::message::Message;
pub use pact_matching::models::provider_states::ProviderState as NonCProviderState;

/// Get a mutable pointer to a newly-created default message on the heap.
#[no_mangle]
pub extern "C" fn message_new() -> *mut Message {
    ffi! {
        name: "message_new",
        op: {
            Ok(ptr::raw_to(Message::default()))
        },
        fail: {
            ptr::null_mut_to::<Message>()
        }
    }
}

/// Destroy the `Message` being pointed to.
#[no_mangle]
pub extern "C" fn message_delete(message: *mut Message) -> c_int {
    ffi! {
        name: "message_delete",
        params: [message],
        op: {
            ptr::drop_raw(message);
            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
    }
}

/// Constructs a `Message` from the JSON string
#[no_mangle]
pub extern "C" fn message_from_json(
    index: c_uint,
    json_str: *const c_char,
    spec_version: PactSpecification,
) -> *mut Message {
    ffi! {
        name: "message_from_json",
        op: {
            if json_str.is_null() {
                anyhow::bail!("json_str is null");
            }

            let json_str = unsafe { CStr::from_ptr(json_str) };
            let json_str = json_str
                .to_str()
                .context("Error parsing json_str as UTF-8")?;

            let json_value: serde_json::Value =
                serde_json::from_str(json_str)
                .context("Error parsing json_str as JSON")?;

            let message = Message::from_json(
                index as usize,
                &json_value,
                &spec_version.into())
                .map_err(|e| anyhow::anyhow!("Pact error: {}", e))?;

            Ok(ptr::raw_to(message))
        },
        fail: {
            ptr::null_mut_to::<Message>()
        }
    }
}

/// Get a copy of the description.
/// The returned string must be deleted with `string_delete`.
///
/// Since it is a copy, the returned string may safely outlive
/// the `Message`.
///
/// # Errors
///
/// On failure, this function will return a NULL pointer.
///
/// This function may fail if the Rust string contains embedded
/// null ('\0') bytes.
#[no_mangle]
pub extern "C" fn message_get_description(
    message: *const Message,
) -> *const c_char {
    ffi! {
        name: "message_get_description",
        op: {
            if message.is_null() {
                anyhow::bail!("message is null");
            }

            let description = unsafe { &(*message).description };

            Ok(string::into_leaked_cstring(description.clone())?)
        },
        fail: {
            ptr::null_to::<c_char>()
        }
    }
}

/// Write the `description` field on the `Message`.
///
/// `description` must contain valid UTF-8. Invalid UTF-8
/// will be replaced with U+FFFD REPLACEMENT CHARACTER.
///
/// This function will only reallocate if the new string
/// does not fit in the existing buffer.
#[no_mangle]
pub extern "C" fn message_set_description(
    message: *mut Message,
    description: *const c_char,
) {
    ffi! {
        name: "message_set_description",
        op: {
            if message.is_null() {
                anyhow::bail!("message is null");
            }

            if description.is_null() {
                anyhow::bail!("description is null");
            }

            let message = unsafe { &mut (*message) };
            let description = unsafe { CStr::from_ptr(description) };
            let description = description.to_string_lossy();

            // Wipe out the previous contents of the string, without
            // deallocating.
            message.description.clear();

            message.description.push_str(&description);
            Ok(())
        },
        fail: {
        }
    }
}

/// FFI structure mirroring the internal Rust ProviderState struct.
/// Contains the name of this Provider State,
/// and a list of (key, value) parameters as an array of structures.
/// The number of elements is stored in 'params_length'.
///
/// This structure should not be mutated.
#[allow(missing_copy_implementations)]
#[repr(C)]
#[derive(Debug)]
pub struct ProviderState {
    /// null terminated string containing the name
    pub name: *const c_char,
    /// pointer to array of key, value pairs
    pub params_list: *const ProviderStateParamsKV,
    /// number of elements in `params_list`
    pub params_length: usize,
    /// private, tracks allocated capacity of the underlying Vec
    capacity: usize,
}

/// FFI structure representing a (key, value) pair
/// for the ProviderState parameters.
///
/// The `value` field is a JSON object, serialized to a string.
///
/// This structure should not be mutated.
#[allow(missing_copy_implementations)]
#[repr(C)]
#[derive(Debug)]
pub struct ProviderStateParamsKV {
    /// null terminated string containing the key
    pub key: *const c_char,
    /// null terminated JSON string
    pub value: *const c_char,
}

/// Create and leak a ProviderState.  Must be passed back to
/// impl_provider_state_delete to clean up memory.
fn into_leaked_provider_state(
    provider_state: &NonCProviderState,
) -> Result<*const ProviderState, anyhow::Error> {
    let name = &provider_state.name;
    let params = &provider_state.params;
    let mut list = Vec::with_capacity(params.len());

    // First check all the strings for embedded null.
    // This prevents leaking memory in the case where
    // an error occurs after some strings were intentionally
    // leaked, but before they can be passed to C.

    if name.find(|c| c == '\0').is_some() {
        anyhow::bail!(
            "Found embedded null in \
                      a provider state name: '{}'",
            name
        );
    }

    for (k, _v) in params.iter() {
        if k.find(|c| c == '\0').is_some() {
            anyhow::bail!(
                "Found embedded null in \
                          a provider state key name: '{}'",
                k
            );
        }
    }

    for (k, v) in params.iter() {
        // It is safe to unwrap, since the strings were already
        // checked for embedded nulls.
        let kv = ProviderStateParamsKV {
            key: string::into_leaked_cstring(k.clone()).unwrap(),
            value: string::into_leaked_cstring(v.to_string()).unwrap(),
        };

        list.push(kv);
    }

    let provider_state_ffi = ProviderState {
        // It is safe to unwrap, since the string was already
        // checked for embedded nulls.
        name: string::into_leaked_cstring(name.clone()).unwrap(),
        params_list: list.as_ptr(),
        params_length: list.len(),
        capacity: list.capacity(),
    };

    std::mem::forget(list);

    let output = Box::new(provider_state_ffi);

    Ok(Box::into_raw(output))
}

/// Manually delete a ProviderState.
/// Returns all leaked memory into Rust structures, which will
/// be automatically cleaned up on Drop.
fn impl_provider_state_delete(ptr: *const ProviderState) {
    let provider_state =
        unsafe { Box::from_raw(ptr as *mut ProviderState) };

    let _name =
        unsafe { CString::from_raw(provider_state.name as *mut c_char) };

    let list = unsafe {
        Vec::from_raw_parts(
            provider_state.params_list as *mut ProviderStateParamsKV,
            provider_state.params_length,
            provider_state.capacity,
        )
    };

    for kv in list {
        let _k = unsafe { CString::from_raw(kv.key as *mut c_char) };
        let _v = unsafe { CString::from_raw(kv.value as *mut c_char) };
    }
}

/// The result of calling message_get_provider_state
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GetProviderStateResult {
    /// Success
    Success,
    /// The requested index was out of bounds
    IndexOutOfBounds,
    /// Some other error occured; check error messages
    OtherError,
}

/// Get a copy of the provider state at the given index from this message.
/// A pointer to the structure will be written to `out_provider_state`,
/// only if no errors are encountered.
///
/// The returned structure must be deleted with `provider_state_delete`.
///
/// Since it is a copy, the returned structure may safely outlive
/// the `Message`.
///
/// # Errors
///
/// On failure, this function will return a variant other than Success.
///
/// This function may fail if the index requested is out of bounds,
/// or if any of the Rust strings contain embedded null ('\0') bytes.
#[no_mangle]
pub extern "C" fn message_get_provider_state(
    message: *const Message,
    index: usize,
    out_provider_state: *mut *const ProviderState,
) -> GetProviderStateResult {
    use GetProviderStateResult::*;
    ffi! {
        name: "message_get_provider_state",
        op: {
            if message.is_null() {
                anyhow::bail!("message is null");
            }

            let message = unsafe { &(*message) };

            match message.provider_states.get(index) {
                None => {
                    Ok(IndexOutOfBounds)
                }
                Some(provider_state) => {
                    unsafe { std::ptr::write(
                        out_provider_state,
                        into_leaked_provider_state(provider_state)?) };

                    Ok(Success)
                }
            }
        },
        fail: {
            OtherError
        }
    }
}

/// Delete a ProviderState previously returned by this FFI.
///
/// It is explicitly allowed to pass a null pointer to this function;
/// in that case the function will do nothing.
#[no_mangle]
pub extern "C" fn provider_state_delete(
    provider_state: *const ProviderState,
) {
    ffi! {
        name: "provider_state_delete",
        op: {
            if provider_state.is_null() {
                return Ok(());
            }

            impl_provider_state_delete(provider_state);
            Ok(())
        },
        fail: {
        }
    }
}

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
            key: string::into_leaked_cstring(k.clone()).unwrap(),
            value: string::into_leaked_cstring(v.clone()).unwrap(),
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

/// Get a copy of the metadata list from this message.
/// It is in the form of a list of (key, value) pairs,
/// in an unspecified order.
/// The returned structure must be deleted with `metadata_list_delete`.
///
/// Since it is a copy, the returned structure may safely outlive
/// the `Message`.
///
/// # Errors
///
/// On failure, this function will return a NULL pointer.
///
/// This function may fail if any of the Rust strings contain
/// embedded null ('\0') bytes.
#[no_mangle]
pub extern "C" fn message_get_metadata_list(
    message: *const Message,
) -> *const MetadataList {
    ffi! {
        name: "message_metadata_list",
        op: {
            if message.is_null() {
                anyhow::bail!("message is null");
            }

            let message = unsafe { &(*message) };
            into_leaked_metadata_list(&message.metadata)
        },
        fail: {
            ptr::null_to::<MetadataList>()
        }
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

/// Get a copy of the metadata value indexed by `key`.
/// The returned string must be deleted with `string_delete`.
///
/// Since it is a copy, the returned string may safely outlive
/// the `Message`.
///
/// The returned pointer will be NULL if the metadata does not contain
/// the given key, or if an error occurred.
///
/// # Errors
///
/// On failure, this function will return a NULL pointer.
///
/// This function may fail if the provided `key` string contains
/// invalid UTF-8, or if the Rust string contains embedded null ('\0')
/// bytes.
#[no_mangle]
pub extern "C" fn message_metadata_get(
    message: *const Message,
    key: *const c_char,
) -> *const c_char {
    ffi! {
        name: "message_metadata_get",
        op: {
            if message.is_null() {
                anyhow::bail!("message is null");
            }

            if key.is_null() {
                anyhow::bail!("key is null");
            }

            let message = unsafe { &(*message) };
            let key = unsafe { CStr::from_ptr(key) };
            let key = key
                .to_str()
                .context("Error parsing key as UTF-8")?;

            match message.metadata.get(key) {
                None => Ok(ptr::null_to::<c_char>()),
                Some(value) => {
                    Ok(string::into_leaked_cstring(value.clone())?)
                },
            }
        },
        fail: {
            ptr::null_to::<c_char>()
        }
    }
}

/// Result from an attempt to insert into a HashMap
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HashMapInsertResult {
    /// The value was inserted, and the key was unset
    SuccessNew,
    /// The value was inserted, and the key was previously set
    SuccessOverwrite,
    /// An error occured, and the value was not inserted
    Error,
}

/// Insert the (`key`, `value`) pair into this Message's
/// `metadata` HashMap.
/// This function returns an enum indicating the result;
/// see the comments on HashMapInsertResult for details.
///
/// # Errors
///
/// This function may fail if the provided `key` or `value` strings
/// contain invalid UTF-8.
#[no_mangle]
pub extern "C" fn message_metadata_insert(
    message: *mut Message,
    key: *const c_char,
    value: *const c_char,
) -> HashMapInsertResult {
    use HashMapInsertResult::*;
    ffi! {
        name: "message_metadata_insert",
        op: {
            if message.is_null() {
                anyhow::bail!("message is null");
            }

            if key.is_null() {
                anyhow::bail!("key is null");
            }

            if value.is_null() {
                anyhow::bail!("value is null");
            }

            let message = unsafe { &mut (*message) };
            let key = unsafe { CStr::from_ptr(key) };
            let key = key
                .to_str()
                .context("Error parsing key as UTF-8")?;

            let value = unsafe { CStr::from_ptr(value) };
            let value = value
                .to_str()
                .context("Error parsing value as UTF-8")?;

            match message.metadata.insert(key.to_string(), value.to_string()) {
                None => Ok(SuccessNew),
                Some(_) => Ok(SuccessOverwrite),
            }
        },
        fail: {
            Error
        }
    }
}

/// Delete a string previously returned by this FFI.
///
/// It is explicitly allowed to pass a null pointer to this function;
/// in that case the function will do nothing.
#[no_mangle]
pub extern "C" fn string_delete(string: *mut c_char) {
    ffi! {
        name: "string_delete",
        op: {
            if string.is_null() {
                return Ok(());
            }

            let string = unsafe { CString::from_raw(string) };
            std::mem::drop(string);
            Ok(())
        },
        fail: {
        }
    }
}
