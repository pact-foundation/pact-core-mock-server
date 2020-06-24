## Publishing the FFI libs to conan repo

### For lib conan package

```
 git checkout libpact_mock_server_ffi-v0.0.7
 cd conan/lib/
 conan create . pact/beta
 conan upload pact_mock_server_ffi/0.0.7@pact/beta -r=pact-foundation
```

### For DLL conan package

```
 git checkout libpact_mock_server_ffi-v0.0.7
 cd conan/dll/
 conan create . pact/beta
 conan upload pact_mock_server_ffi_dll/0.0.7@pact/beta -r=pact-foundation
```
