//! Module for functions to deal with request, response and message contents

use bytes::Bytes;
use libc::{c_char, c_uchar, size_t};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, ContentTypeHint};
use pact_models::v4::message_parts::MessageContents;

use crate::{as_mut, as_ref, ffi_fn, safe_str};
use crate::models::message::MessageMetadataIterator;
use crate::string::optional_str;
use crate::util::*;

ffi_fn! {
    /// Get the message contents in string form.
    ///
    /// # Safety
    ///
    /// The returned string must be deleted with `pactffi_string_delete`.
    ///
    /// The returned string can outlive the message.
    ///
    /// # Error Handling
    ///
    /// If the message contents is NULL, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL. This means there's
    /// no mechanism to differentiate with this function call alone between
    /// a NULL message and a missing message body.
    fn pactffi_message_contents_get_contents_str(contents: *const MessageContents) -> *const c_char {
        let contents = as_ref!(contents);

        match contents.contents {
            // If it's missing, return a null pointer.
            OptionalBody::Missing => std::ptr::null(),
            // If empty or null, return an empty string on the heap.
            OptionalBody::Empty | OptionalBody::Null => {
                let content = string::to_c("")?;
                content as *const c_char
            }
            // Otherwise, get the contents, possibly still empty.
            _ => {
                let content = string::to_c(contents.contents.value_as_string().unwrap_or_default().as_str())?;
                content as *const c_char
            }
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the contents of the message as a string.
  ///
  /// * `contents` - the message contents to set the contents for
  /// * `contents_str` - pointer to contents to copy from. Must be a valid NULL-terminated UTF-8 string pointer.
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
  ///
  /// # Safety
  ///
  /// The message contents and content type must either be NULL pointers, or point to valid
  /// UTF-8 encoded NULL-terminated strings. Otherwise behaviour is undefined.
  ///
  /// # Error Handling
  ///
  /// If the contents string is a NULL pointer, it will set the message contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_message_contents_set_contents_str(contents: *mut MessageContents, contents_str: *const c_char, content_type: *const c_char) {
    let contents = as_mut!(contents);

    if contents_str.is_null() {
      contents.contents = OptionalBody::Null;
    } else {
      let contents_str = safe_str!(contents_str);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      contents.contents = OptionalBody::Present(Bytes::from(contents_str), content_type, Some(ContentTypeHint::TEXT));
    }
  }
}

ffi_fn! {
    /// Get the length of the message contents.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns 0. If the body of the message
    /// is missing, then this function also returns 0.
    fn pactffi_message_contents_get_contents_length(contents: *const MessageContents) -> size_t {
        let contents = as_ref!(contents);

        match &contents.contents {
            OptionalBody::Missing | OptionalBody::Empty | OptionalBody::Null => 0 as size_t,
            OptionalBody::Present(bytes, _, _) => bytes.len() as size_t
        }
    } {
        0 as size_t
    }
}

ffi_fn! {
    /// Get the contents of a message as a pointer to an array of bytes.
    ///
    /// # Safety
    ///
    /// The number of bytes in the buffer will be returned by `pactffi_message_contents_get_contents_length`.
    /// It is safe to use the pointer while the message is not deleted or changed. Using the pointer
    /// after the message is mutated or deleted may lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// If the message is NULL, returns NULL. If the body of the message
    /// is missing, then this function also returns NULL.
    fn pactffi_message_contents_get_contents_bin(contents: *const MessageContents) -> *const c_uchar {
        let contents = as_ref!(contents);

        match &contents.contents {
            OptionalBody::Empty | OptionalBody::Null | OptionalBody::Missing => std::ptr::null(),
            OptionalBody::Present(bytes, _, _) => bytes.as_ptr()
        }
    } {
        std::ptr::null()
    }
}

ffi_fn! {
  /// Sets the contents of the message as an array of bytes.
  ///
  /// * `message` - the message contents to set the contents for
  /// * `contents_bin` - pointer to contents to copy from
  /// * `len` - number of bytes to copy from the contents pointer
  /// * `content_type` - pointer to the NULL-terminated UTF-8 string containing the content type of the data.
  ///
  /// # Safety
  ///
  /// The contents pointer must be valid for reads of `len` bytes, and it must be properly aligned
  /// and consecutive. Otherwise behaviour is undefined.
  ///
  /// # Error Handling
  ///
  /// If the contents is a NULL pointer, it will set the message contents as null. If the content
  /// type is a null pointer, or can't be parsed, it will set the content type as unknown.
  fn pactffi_message_contents_set_contents_bin(
    contents: *mut MessageContents,
    contents_bin: *const c_uchar,
    len: size_t,
    content_type: *const c_char
  ) {
    let contents = as_mut!(contents);

    if contents_bin.is_null() {
      contents.contents = OptionalBody::Null;
    } else {
      let slice = unsafe { std::slice::from_raw_parts(contents_bin, len) };
      let contents_bytes = Bytes::from(slice);
      let content_type = optional_str(content_type).map(|ct| ContentType::parse(ct.as_str()).ok()).flatten();
      contents.contents = OptionalBody::Present(contents_bytes, content_type, Some(ContentTypeHint::BINARY));
    }
  }
}

ffi_fn! {
    /// Get an iterator over the metadata of a message.
    ///
    /// # Safety
    ///
    /// This iterator carries a pointer to the message contents, and must
    /// not outlive the message.
    ///
    /// The message metadata also must not be modified during iteration. If it is,
    /// the old iterator must be deleted and a new iterator created.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain
    /// embedded null ('\0') bytes.
    fn pactffi_message_contents_get_metadata_iter(contents: *mut MessageContents) -> *mut MessageMetadataIterator {
        let contents = as_mut!(contents);

        let iter = MessageMetadataIterator::new_from_contents(&contents);
        ptr::raw_to(iter)
    } {
        ptr::null_mut_to::<MessageMetadataIterator>()
    }
}
