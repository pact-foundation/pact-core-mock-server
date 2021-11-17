//! V4 Synchronous request/response messages

use anyhow::{anyhow, Context};
use libc::{c_char, c_int, c_uchar, c_uint, EXIT_FAILURE, EXIT_SUCCESS, size_t};

use pact_models::bodies::OptionalBody;
use pact_models::provider_states::ProviderState;
use pact_models::v4::sync_message::SynchronousMessage;

use crate::{as_mut, as_ref, ffi_fn, safe_str};
use crate::models::message::ProviderStateIterator;
use crate::ptr;
use crate::util::*;

ffi_fn! {
    /// Destroy the `Message` being pointed to.
    fn pactffi_sync_message_delete(message: *mut SynchronousMessage) {
        ptr::drop_raw(message);
    }
}

ffi_fn! {
    /// Get the request contents of a `SynchronousMessage` in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// The returned string can outlive the message.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL. If the body of the request message
    /// is missing, then this function also returns NULL. This means there's
    /// no mechanism to differentiate with this function call alone between
    /// a NULL message and a missing message body.
    fn pactffi_sync_message_get_request_contents(message: *const SynchronousMessage) -> *const c_char {
        let message = as_ref!(message);

        match message.request.contents {
            // If it's missing, return a null pointer.
            OptionalBody::Missing => ptr::null_to::<c_char>(),
            // If empty or null, return an empty string on the heap.
            OptionalBody::Empty | OptionalBody::Null => {
                let content = string::to_c("")?;
                content as *const c_char
            }
            // Otherwise, get the contents, possibly still empty.
            _ => {
                let content = string::to_c(message.request.contents.str_value())?;
                content as *const c_char
            }
        }
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Get the length of the request contents of a `SynchronousMessage`.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns 0. If the body of the request
    /// is missing, then this function also returns 0.
    fn pactffi_sync_message_get_request_contents_length(message: *const SynchronousMessage) -> size_t {
        let message = as_ref!(message);

        match &message.request.contents {
            OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
            OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the request contents of a `SynchronousMessage` as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_sync_message_get_request_contents_length`.
    /// It is safe to use the pointer while the message is not deleted or changed. Using the pointer
    /// after the message is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL.
    fn pactffi_sync_message_get_request_contents_bin(message: *const SynchronousMessage) -> *const c_uchar {
        let message = as_ref!(message);

        match &message.request.contents {
            OptionalBody::Empty | OptionalBody::Null | OptionalBody::Missing => ptr::null_to::<c_uchar>(),
            OptionalBody::Present(bytes, _, _) => bytes.as_ptr()
        }
    } {
        ptr::null_to::<c_uchar>()
    }
}

ffi_fn! {
    /// Get the number of response messages in the `SynchronousMessage`.
    ///
    /// # Safety
    ///
    /// The message pointer must point to a valid SynchronousMessage.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns 0.
    fn pactffi_sync_message_get_number_responses(message: *const SynchronousMessage) -> size_t {
        let message = as_ref!(message);
        message.response.len() as size_t
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the response contents of a `SynchronousMessage` in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// The returned string can outlive the message.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL or the index is not valid, returns NULL.
    ///
    /// If the body of the response message is missing, then this function also returns NULL.
    /// This means there's no mechanism to differentiate with this function call alone between
    /// a NULL message and a missing message body.
    fn pactffi_sync_message_get_response_contents(message: *const SynchronousMessage, index: size_t) -> *const c_char {
        let message = as_ref!(message);

        match message.response.get(index) {
            Some(response) => match response.contents {
                // If it's missing, return a null pointer.
                OptionalBody::Missing => ptr::null_to::<c_char>(),
                // If empty or null, return an empty string on the heap.
                OptionalBody::Empty | OptionalBody::Null => {
                    let content = string::to_c("")?;
                    content as *const c_char
                }
                // Otherwise, get the contents, possibly still empty.
                _ => {
                    let content = string::to_c(response.contents.str_value())?;
                    content as *const c_char
                }
            }
            None => ptr::null_to::<c_char>()
        }
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Get the length of the response contents of a `SynchronousMessage`.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL or the index is not valid, returns 0. If the body of the request
    /// is missing, then this function also returns 0.
    fn pactffi_sync_message_get_response_contents_length(message: *const SynchronousMessage, index: size_t) -> size_t {
        let message = as_ref!(message);

        match message.response.get(index) {
            Some(response) => match &response.contents {
                OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
                OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
            }
            None => 0 as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the response contents of a `SynchronousMessage` as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_sync_message_get_response_contents_length`.
    /// It is safe to use the pointer while the message is not deleted or changed. Using the pointer
    /// after the message is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL or the index is not valid, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL.
    fn pactffi_sync_message_get_response_contents_bin(message: *const SynchronousMessage, index: size_t) -> *const c_uchar {
        let message = as_ref!(message);

        match message.response.get(index) {
            Some(response) => match &response.contents {
                OptionalBody::Empty | OptionalBody::Null | OptionalBody::Missing => ptr::null_to::<c_uchar>(),
                OptionalBody::Present(bytes, _, _) => bytes.as_ptr()
            }
            None => ptr::null_to::<c_uchar>()
        }
    } {
        ptr::null_to::<c_uchar>()
    }
}

ffi_fn! {
    /// Get a copy of the description.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// Since it is a copy, the returned string may safely outlive
    /// the `SynchronousMessage`.
    ///
    /// # Errors
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if the Rust string contains embedded
    /// null ('\0') bytes.
    fn pactffi_sync_message_get_description(message: *const SynchronousMessage) -> *const c_char {
        let message = as_ref!(message);
        let description = string::to_c(&message.description)?;
        description as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Write the `description` field on the `SynchronousMessage`.
    ///
    /// # Safety
    ///
    /// `description` must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    /// This function will only reallocate if the new string
    /// does not fit in the existing buffer.
    ///
    /// # Error Handling
    ///
    /// Errors will be reported with a non-zero return value.
    fn pactffi_sync_message_set_description(message: *mut SynchronousMessage, description: *const c_char) -> c_int {
        let message = as_mut!(message);
        let description = safe_str!(description);

        // Wipe out the previous contents of the string, without
        // deallocating, and set the new description.
        message.description.clear();
        message.description.push_str(description);

        EXIT_SUCCESS
    } {
        EXIT_FAILURE
    }
}


ffi_fn! {
    /// Get a copy of the provider state at the given index from this message.
    ///
    /// # Safety
    ///
    /// The returned structure must be deleted with `provider_state_delete`.
    ///
    /// Since it is a copy, the returned structure may safely outlive
    /// the `SynchronousMessage`.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a variant other than Success.
    ///
    /// This function may fail if the index requested is out of bounds,
    /// or if any of the Rust strings contain embedded null ('\0') bytes.
    fn pactffi_sync_message_get_provider_state(message: *const SynchronousMessage, index: c_uint) -> *const ProviderState {
        let message = as_ref!(message);
        let index = index as usize;

        // Get a raw pointer directly, rather than boxing it, as its owned by the `SynchronousMessage`
        // and will be cleaned up when the `SynchronousMessage` is cleaned up.
        let provider_state = message
            .provider_states
            .get(index)
            .ok_or(anyhow!("index is out of bounds"))?;

        provider_state as *const ProviderState
    } {
        ptr::null_to::<ProviderState>()
    }
}

ffi_fn! {
    /// Get an iterator over provider states.
    ///
    /// # Safety
    ///
    /// The underlying data must not change during iteration.
    ///
    /// # Error Handling
    ///
    /// Returns NULL if an error occurs.
    fn pactffi_sync_message_get_provider_state_iter(message: *mut SynchronousMessage) -> *mut ProviderStateIterator {
        let message = as_mut!(message);
        let iter = ProviderStateIterator::new(message);
        ptr::raw_to(iter)
    } {
        ptr::null_mut_to::<ProviderStateIterator>()
    }
}
