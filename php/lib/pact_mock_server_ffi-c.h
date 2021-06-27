#ifndef PACT_MOCK_SERVER_FFI_H
#define PACT_MOCK_SERVER_FFI_H

/* Generated with cbindgen:0.14.3 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Request or Response enum
 */
typedef enum {
  /**
   * Request part
   */
  Request,
  /**
   * Response part
   */
  Response,
} InteractionPart;

/**
 * Wraps a Pact model struct
 */
typedef struct {
  /**
   * Pact reference
   */
  uintptr_t pact;
} PactHandle;

/**
 * Result of wrapping a string value
 */
typedef enum {
  /**
   * Was generated OK
   */
  Ok,
  /**
   * There was an error generating the string
   */
  Failed,
} StringResult_Tag;

/**
 * Enum defining the pact specification versions supported by the library
 */
typedef enum PactSpecification {
  /**
   * Unknown or unsupported specification version
   */
  PactSpecification_Unknown,
  /**
   * First version of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-1>)
   */
  PactSpecification_V1,
  /**
   * Second version of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-1.1>)
   */
  PactSpecification_V1_1,
  /**
   * Version two of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-2>)
   */
  PactSpecification_V2,
  /**
   * Version three of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-3>)
   */
  PactSpecification_V3,
  /**
   * Version four of the pact specification (<https://github.com/pact-foundation/pact-specification/tree/version-4>)
   */
  PactSpecification_V4,
} PactSpecification;

typedef struct {
  char *_0;
} Ok_Body;

typedef struct {
  char *_0;
} Failed_Body;

typedef struct {
  StringResult_Tag tag;
  union {
    Ok_Body ok;
    Failed_Body failed;
  };
} StringResult;

/**
 * Wraps a Pact model struct
 */
typedef struct {
  /**
   * Pact reference
   */
  uintptr_t pact;
  /**
   * Interaction reference
   */
  uintptr_t interaction;
} InteractionHandle;

/**
 * Wraps a Pact model struct
 */
typedef struct {
  /**
   * Message Pact reference
   */
  uintptr_t pact;
  /**
   * Interaction reference
   */
  uintptr_t message;
} MessageHandle;

/**
 * Wraps a Pact model struct
 */
typedef struct {
  /**
   * Pact reference
   */
  uintptr_t pact;
} MessagePactHandle;

/**
 * Checks that the example string matches the given regex
 *
 * # Safety
 *
 * Exported functions are inherently unsafe.
 */
bool check_regex(const char *regex, const char *example);

/**
 * External interface to cleanup a mock server. This function will try terminate the mock server
 * with the given port number and cleanup any memory allocated for it. Returns true, unless a
 * mock server with the given port number does not exist, or the function panics.
 */
bool cleanup_mock_server(int32_t mock_server_port);

/**
 * External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
 * as well as the port for the mock server to run on. A value of 0 for the port will result in a
 * port being allocated by the operating system. The port of the mock server is returned.
 *
 * * `pact_str` - Pact JSON
 * * `addr_str` - Address to bind to in the form name:port (i.e. 127.0.0.1:0)
 * * `tls` - boolean flag to indicate of the mock server should use TLS (using a self-signed certificate)
 *
 * # Errors
 *
 * Errors are returned as negative values.
 *
 * | Error | Description |
 * |-------|-------------|
 * | -1 | A null pointer was received |
 * | -2 | The pact JSON could not be parsed |
 * | -3 | The mock server could not be started |
 * | -4 | The method panicked |
 * | -5 | The address is not valid |
 * | -6 | Could not create the TLS configuration with the self-signed certificate |
 *
 */
int32_t create_mock_server(const char *pact_str,
                           const char *addr_str,
                           bool tls);

/**
 * External interface to create a mock server. A Pact handle is passed in,
 * as well as the port for the mock server to run on. A value of 0 for the port will result in a
 * port being allocated by the operating system. The port of the mock server is returned.
 *
 * * `pact` - Handle to a Pact model
 * * `addr_str` - Address to bind to in the form name:port (i.e. 127.0.0.1:0)
 * * `tls` - boolean flag to indicate of the mock server should use TLS (using a self-signed certificate)
 *
 * # Errors
 *
 * Errors are returned as negative values.
 *
 * | Error | Description |
 * |-------|-------------|
 * | -1 | An invalid handle was received |
 * | -3 | The mock server could not be started |
 * | -4 | The method panicked |
 * | -5 | The address is not valid |
 * | -6 | Could not create the TLS configuration with the self-signed certificate |
 *
 */
int32_t create_mock_server_for_pact(PactHandle pact,
                                    const char *addr_str,
                                    bool tls);

/**
 * Frees the memory allocated to a string by another function
 *
 * # Safety
 *
 * Exported functions are inherently unsafe.
 */
void free_string(char *s);

/**
 * Generates a datetime value from the provided format string, using the current system date and time
 * NOTE: The memory for the returned string needs to be freed with the free_string function
 *
 * # Safety
 *
 * Exported functions are inherently unsafe.
 */
StringResult generate_datetime_string(const char *format);

/**
 * Generates an example string based on the provided regex.
 * NOTE: The memory for the returned string needs to be freed with the free_string function
 *
 * # Safety
 *
 * Exported functions are inherently unsafe.
 */
StringResult generate_regex_value(const char *regex);

/**
 * Fetch the CA Certificate used to generate the self-signed certificate for the TLS mock server.
 *
 * **NOTE:** The string for the result is allocated on the heap, and will have to be freed
 * by the caller using free_string
 *
 * # Errors
 *
 * An empty string indicates an error reading the pem file
 */
char *get_tls_ca_certificate(void);

/**
 * Adds a provider state to the Interaction. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `description` - The provider state description. It needs to be unique.
 */
bool given(InteractionHandle interaction, const char *description);

/**
 * Adds a provider state to the Interaction with a parameter key and value. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `description` - The provider state description. It needs to be unique.
 * * `name` - Parameter name.
 * * `value` - Parameter value.
 */
bool given_with_param(InteractionHandle interaction,
                      const char *description,
                      const char *name,
                      const char *value);

/**
 * Initialise the mock server library, can provide an environment variable name to use to
 * set the log levels.
 *
 * # Safety
 *
 * Exported functions are inherently unsafe.
 */
void init(const char *log_env_var);

/**
 * Sets the description for the Message.
 *
 * * `description` - The message description. It needs to be unique for each message.
 */
void message_expects_to_receive(MessageHandle message, const char *description);

/**
 * Adds a provider state to the Interaction.
 *
 * * `description` - The provider state description. It needs to be unique for each message
 */
void message_given(MessageHandle message, const char *description);

/**
 * Adds a provider state to the Message with a parameter key and value.
 *
 * * `description` - The provider state description. It needs to be unique.
 * * `name` - Parameter name.
 * * `value` - Parameter value.
 */
void message_given_with_param(MessageHandle message,
                              const char *description,
                              const char *name,
                              const char *value);

/**
 * Reifies the given message
 *
 * Reification is the process of stripping away any matchers, and returning the original contents.
 * NOTE: the returned string needs to be deallocated with the `free_string` function
 */
const char *message_reify(MessageHandle message);

/**
 * Adds the contents of the Message.
 *
 * Accepts JSON, binary and other payload types. Binary data will be base64 encoded when serialised.
 *
 * * `content_type` - The content type of the body. Defaults to `text/plain`, supports JSON structures with matchers and binary data.
 * * `body` - The body contents. For JSON payloads, matching rules can be embedded in the body.
 * * `content_type` - Expected content type (e.g. application/json, application/octet-stream)
 * * `size` - number of bytes in the message to read
 */
void message_with_contents(MessageHandle message,
                           const char *content_type,
                           const char *body,
                           size_t size);

/**
 * Adds expected metadata to the Message
 *
 * * `key` - metadata key
 * * `value` - metadata value.
 */
void message_with_metadata(MessageHandle message, const char *key, const char *value);

/**
 * Fetch the logs for the mock server. This needs the memory buffer log sink to be setup before
 * the mock server is started. Returned string will be freed with the `cleanup_mock_server`
 * function call.
 *
 * Will return a NULL pointer if the logs for the mock server can not be retrieved.
 */
const char *mock_server_logs(int32_t mock_server_port);

/**
 * External interface to check if a mock server has matched all its requests. The port number is
 * passed in, and if all requests have been matched, true is returned. False is returned if there
 * is no mock server on the given port, or if any request has not been successfully matched, or
 * the method panics.
 */
bool mock_server_matched(int32_t mock_server_port);

/**
 * External interface to get all the mismatches from a mock server. The port number of the mock
 * server is passed in, and a pointer to a C string with the mismatches in JSON format is
 * returned.
 *
 * **NOTE:** The JSON string for the result is allocated on the heap, and will have to be freed
 * once the code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is
 * provided for this purpose.
 *
 * # Errors
 *
 * If there is no mock server with the provided port number, or the function panics, a NULL
 * pointer will be returned. Don't try to dereference it, it will not end well for you.
 *
 */
char *mock_server_mismatches(int32_t mock_server_port);

/**
 * Creates a new Interaction and returns a handle to it.
 *
 * * `description` - The interaction description. It needs to be unique for each interaction.
 *
 * Returns a new `InteractionHandle`.
 */
InteractionHandle new_interaction(PactHandle pact, const char *description);

/**
 * Creates a new Message and returns a handle to it.
 *
 * * `description` - The message description. It needs to be unique for each Message.
 *
 * Returns a new `MessageHandle`.
 */
MessageHandle new_message(MessagePactHandle pact, const char *description);

/**
 * Creates a new Pact Message model and returns a handle to it.
 *
 * * `consumer_name` - The name of the consumer for the pact.
 * * `provider_name` - The name of the provider for the pact.
 *
 * Returns a new `MessagePactHandle`.
 */
MessagePactHandle new_message_pact(const char *consumer_name, const char *provider_name);

/**
 * Creates a new Pact model and returns a handle to it.
 *
 * * `consumer_name` - The name of the consumer for the pact.
 * * `provider_name` - The name of the provider for the pact.
 *
 * Returns a new `PactHandle`.
 */
PactHandle new_pact(const char *consumer_name, const char *provider_name);

/**
 * Configures the response for the Interaction. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `status` - the response status. Defaults to 200.
 */
bool response_status(InteractionHandle interaction, unsigned short status);

/**
 * Sets the description for the Interaction. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `description` - The interaction description. It needs to be unique for each interaction.
 */
bool upon_receiving(InteractionHandle interaction, const char *description);

/**
 * Get the current library version
 */
const char *version(void);

/**
 * Adds a binary file as the body with the expected content type and example contents. Will use
 * a mime type matcher to match the body. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `interaction` - Interaction handle to set the body for.
 * * `part` - Request or response part.
 * * `content_type` - Expected content type.
 * * `body` - example body contents in bytes
 */
bool with_binary_file(InteractionHandle interaction,
                      InteractionPart part,
                      const char *content_type,
                      const char *body,
                      size_t size);

/**
 * Adds the body for the interaction. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `part` - The part of the interaction to add the body to (Request or Response).
 * * `content_type` - The content type of the body. Defaults to `text/plain`. Will be ignored if a content type
 *   header is already set.
 * * `body` - The body contents. For JSON payloads, matching rules can be embedded in the body.
 */
bool with_body(InteractionHandle interaction,
               InteractionPart part,
               const char *content_type,
               const char *body);

/**
 * Configures a header for the Interaction. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `part` - The part of the interaction to add the header to (Request or Response).
 * * `name` - the header name.
 * * `value` - the header value.
 * * `index` - the index of the value (starts at 0). You can use this to create a header with multiple values
 */
bool with_header(InteractionHandle interaction,
                 InteractionPart part,
                 const char *name,
                 size_t index,
                 const char *value);

/**
 * Sets the additional metadata on the Pact file. Common uses are to add the client library details such as the name and version
 *
 * * `pact` - Handle to a Pact model
 * * `namespace` - the top level metadat key to set any key values on
 * * `name` - the key to set
 * * `value` - the value to set
 */
void with_message_pact_metadata(MessagePactHandle pact,
                                const char *namespace_,
                                const char *name,
                                const char *value);

/**
 * Adds a binary file as the body as a MIME multipart with the expected content type and example contents. Will use
 * a mime type matcher to match the body. Returns an error if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `interaction` - Interaction handle to set the body for.
 * * `part` - Request or response part.
 * * `content_type` - Expected content type of the file.
 * * `file` - path to the example file
 * * `part_name` - name for the mime part
 */
StringResult with_multipart_file(InteractionHandle interaction,
                                 InteractionPart part,
                                 const char *content_type,
                                 const char *file,
                                 const char *part_name);

/**
 * Sets the additional metadata on the Pact file. Common uses are to add the client library details such as the name and version
 * Returns false if the interaction or Pact can't be modified (i.e. the mock server for it has already started)
 *
 * * `pact` - Handle to a Pact model
 * * `namespace` - the top level metadat key to set any key values on
 * * `name` - the key to set
 * * `value` - the value to set
 */
bool with_pact_metadata(PactHandle pact,
                        const char *namespace_,
                        const char *name,
                        const char *value);

/**
 * Configures a query parameter for the Interaction. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `name` - the query parameter name.
 * * `value` - the query parameter value.
 * * `index` - the index of the value (starts at 0). You can use this to create a query parameter with multiple values
 */
bool with_query_parameter(InteractionHandle interaction,
                          const char *name,
                          size_t index,
                          const char *value);

/**
 * Configures the request for the Interaction. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `method` - The request method. Defaults to GET.
 * * `path` - The request path. Defaults to `/`.
 */
bool with_request(InteractionHandle interaction, const char *method, const char *path);

/**
 * Sets the specification version for a given Pact model. Returns false if the interaction or Pact can't be
 * modified (i.e. the mock server for it has already started)
 *
 * * `pact` - Handle to a Pact model
 * * `version` - the spec version to use
 */
bool with_specification(PactHandle pact,
                        PactSpecification version);

/**
 * External interface to write out the message pact file. This function should
 * be called if all the consumer tests have passed. The directory to write the file to is passed
 * as the second parameter. If a NULL pointer is passed, the current working directory is used.
 *
 * If overwrite is true, the file will be overwritten with the contents of the current pact.
 * Otherwise, it will be merged with any existing pact file.
 *
 * Returns 0 if the pact file was successfully written. Returns a positive code if the file can
 * not be written, or there is no mock server running on that port or the function panics.
 *
 * # Errors
 *
 * Errors are returned as positive values.
 *
 * | Error | Description |
 * |-------|-------------|
 * | 1 | The pact file was not able to be written |
 * | 2 | The message pact for the given handle was not found |
 */
int32_t write_message_pact_file(MessagePactHandle pact, const char *directory, bool overwrite);

/**
 * External interface to trigger a mock server to write out its pact file. This function should
 * be called if all the consumer tests have passed. The directory to write the file to is passed
 * as the second parameter. If a NULL pointer is passed, the current working directory is used.
 *
 * If overwrite is true, the file will be overwritten with the contents of the current pact.
 * Otherwise, it will be merged with any existing pact file.
 *
 * Returns 0 if the pact file was successfully written. Returns a positive code if the file can
 * not be written, or there is no mock server running on that port or the function panics.
 *
 * # Errors
 *
 * Errors are returned as positive values.
 *
 * | Error | Description |
 * |-------|-------------|
 * | 1 | A general panic was caught |
 * | 2 | The pact file was not able to be written |
 * | 3 | A mock server with the provided port was not found |
 */
int32_t write_pact_file(int32_t mock_server_port, const char *directory, bool overwrite);

#endif /* PACT_MOCK_SERVER_FFI_H */
