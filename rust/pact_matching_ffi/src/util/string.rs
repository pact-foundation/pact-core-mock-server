use libc::c_char;
use std::ffi::CString;

/// Converts the string into a C-compatible null terminated string,
/// then forgets the container while returning a pointer to the
/// underlying buffer.
///
/// The returned pointer must be passed to CString::from_raw to
/// prevent leaking memory.
pub fn into_leaked_cstring(
    string: String,
) -> Result<*const c_char, anyhow::Error> {
    let copy = CString::new(string)?;
    let ptr = copy.as_ptr();

    // Intentionally leak this memory so that it stays
    // valid while C is using it.
    std::mem::forget(copy);

    Ok(ptr)
}
