from cffi import FFI
from register_ffi import get_ffi_lib

ffi = FFI()
lib = get_ffi_lib(ffi)                    # loads the entire C namespace
result = lib.pactffi_version()
print(ffi.string(result).decode('utf-8'))