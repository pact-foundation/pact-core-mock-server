//! Represents the state of providers in a message.

use crate::util::*;
use crate::{as_mut, as_ref, ffi};
use anyhow::anyhow;
use libc::{c_char, c_int, EXIT_FAILURE, EXIT_SUCCESS};
use pact_matching::models::provider_states::ProviderState;
use serde_json::Value as JsonValue;

/// Get the name of the provider state as a string, which needs to be deleted with `string_delete`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn provider_state_get_name(
    provider_state: *const ProviderState,
) -> *const c_char {
    ffi! {
        name: "provider_state_get_name",
        params: [provider_state],
        op: {
            let provider_state = as_ref!(provider_state);
            Ok(string::to_c(&provider_state.name)? as *const c_char)
        },
        fail: {
            ptr::null_to::<c_char>()
        }
    }
}

/// Get an iterator over the params of a provider state.
///
/// This iterator carries a pointer to the provider state, and must
/// not outlive the provider state.
///
/// The provider state params also must not be modified during iteration. If it is,
/// the old iterator must be deleted and a new iterator created.
///
/// # Errors
///
/// On failure, this function will return a NULL pointer.
///
/// This function may fail if any of the Rust strings contain
/// embedded null ('\0') bytes.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn provider_state_get_param_iter(
    provider_state: *mut ProviderState,
) -> *mut ProviderStateParamIterator {
    ffi! {
        name: "provider_state_get_param_iter",
        params: [provider_state],
        op: {
            let provider_state = as_mut!(provider_state);

            let iter = ProviderStateParamIterator {
                keys:  provider_state.params.keys().cloned().collect(),
                current: 0,
                provider_state: provider_state as *const ProviderState,
            };

            Ok(ptr::raw_to(iter))
        },
        fail: {
            ptr::null_mut_to::<ProviderStateParamIterator>()
        }
    }
}

/// Get the next key and value out of the iterator, if possible
///
/// Returns a pointer to a heap allocated array of 2 elements, the pointer to the
/// key string on the heap, and the pointer to the value string on the heap.
///
/// The user needs to free both the contained strings and the array.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn provider_state_param_iter_next(
    iter: *mut ProviderStateParamIterator,
) -> *mut ProviderStateParamPair {
    ffi! {
        name: "provider_state_param_iter_next",
        params: [iter],
        op: {
            // Reconstitute the iterator.
            let iter = as_mut!(iter);

            // Reconstitute the message.
            let provider_state = as_ref!(iter.provider_state);

            // Get the current key from the iterator.
            let key = iter.next().ok_or(anyhow::anyhow!("iter past the end of params"))?;

            // Get the value for the current key.
            let (key, value) = provider_state.params.get_key_value(key).ok_or(anyhow::anyhow!("iter provided invalid param key"))?;

            // Package up for return.
            let pair = ProviderStateParamPair::new(key, value)?;

            // Leak the value out to the C-side.
            Ok(ptr::raw_to(pair))
        },
        fail: {
            ptr::null_mut_to::<ProviderStateParamPair>()
        }
    }
}

/// Free the provider state param iterator when you're done using it.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn provider_state_param_iter_delete(
    iter: *mut ProviderStateParamIterator,
) -> c_int {
    ffi! {
        name: "provider_state_param_iter_delete",
        params: [iter],
        op: {
            ptr::drop_raw(iter);
            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
    }
}

/// Free a pair of key and value returned from `provider_state_param_iter_next`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn provider_state_param_pair_delete(
    pair: *mut ProviderStateParamPair,
) -> c_int {
    ffi! {
        name: "provider_state_param_pair_delete",
        params: [pair],
        op: {
            ptr::drop_raw(pair);
            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
    }
}

/// An iterator that enables FFI iteration over provider state params by putting all the keys on the heap
/// and tracking which one we're currently at.
///
/// This assumes no mutation of the underlying provider state happens while the iterator is live.
#[derive(Debug)]
pub struct ProviderStateParamIterator {
    /// The provider state param keys
    keys: Vec<String>,
    /// The current key
    current: usize,
    /// Pointer to the provider state.
    provider_state: *const ProviderState,
}

impl ProviderStateParamIterator {
    fn next(&mut self) -> Option<&String> {
        let idx = self.current;
        self.current += 1;
        self.keys.get(idx)
    }
}

/// A single key-value pair exported to the C-side.
#[derive(Debug)]
#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct ProviderStateParamPair {
    key: *const c_char,
    value: *const c_char,
}

impl ProviderStateParamPair {
    fn new(
        key: &str,
        value: &JsonValue,
    ) -> anyhow::Result<ProviderStateParamPair> {
        let value = value.to_string();

        Ok(ProviderStateParamPair {
            key: string::to_c(key)? as *const c_char,
            value: string::to_c(&value)? as *const c_char,
        })
    }
}
