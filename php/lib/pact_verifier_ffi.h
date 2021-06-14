/// Initialise the mock server library, can provide an environment variable name to use to
/// set the log levels.
///
/// # Safety
///
/// Exported functions are inherently unsafe.
void init(const char *log_env_var);

/// External interface to verifier a provider
///
/// * `args` - the same as the CLI interface, except newline delimited
///
/// # Errors
///
/// Errors are returned as non-zero numeric values.
///
/// | Error | Description |
/// |-------|-------------|
/// | 1 | The verification process failed, see output for errors |
/// | 2 | A null pointer was received |
/// | 3 | The method panicked |
/// | 4 | Invalid arguments were provided to the verification process |
///
/// # Safety
///
/// Exported functions are inherently unsafe. Deal.
int32_t verify(const char *args);
