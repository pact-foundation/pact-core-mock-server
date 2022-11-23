from cffi import FFI
from register_ffi import get_ffi_lib
import json
import requests

ffi = FFI()
lib = get_ffi_lib(ffi) # loads the entire C namespace
version_encoded = lib.pactffi_version()
ffi_version = ffi.string(version_encoded).decode('utf-8')

contents ={
        "provider": {
          "name": "Alice Service"
        },
        "consumer": {
          "name": "Consumer"
        },
        "interactions": [
          {
            "description": "a retrieve Mallory request",
            "request": {
              "method": "GET",
              "path": "/mallory",
              "query": "name=ron&status=good"
            },
            "response": {
              "status": 200,
              "headers": {
                "Content-Type": "text/html"
              },
              "body": "That is some good Mallory."
            }
          }
        ],
        "metadata": {
          "pact-specification": {
            "version": "1.0.0"
          },
          "pact-python": {
            "version": "1.0.0",
            "ffi": ffi_version
          }
        }
      }

print(contents)

## Setup Loggers

lib.pactffi_logger_init()
lib.pactffi_logger_attach_sink(b'file ./logs/log-info.txt',5)
lib.pactffi_logger_attach_sink(b'file ./logs/log-error.txt',5)
# lib.pactffi_logger_attach_sink(b'stdout', 5)
# lib.pactffi_logger_attach_sink(b'stderr', 5)
lib.pactffi_logger_apply()
lib.pactffi_log_message(b'pact_python_ffi', b'INFO', b'hello from pact python ffi, using Pact FFI Version: '+ ffi.string(version_encoded))


## Load pact into Mock Server and start
mock_server_port = lib.pactffi_create_mock_server(ffi.new("char[]", json.dumps(contents).encode('ascii')) , b'127.0.0.1:4432',0)
print(f"Mock server started: {mock_server_port}")

## Make our client call

expected_response = 'That is some good Mallory.'
try:
    response = requests.get(f"http://127.0.0.1:{mock_server_port}/mallory?name=ron&status=good")
    print(f"Client response - matched: {response.text}")
    print(f"Client response - matched: {response.text == expected_response}")
    response.raise_for_status()
except requests.HTTPError as http_err:
    print(f'Client request - HTTP error occurred: {http_err}')  # Python 3.6
except Exception as err:
    print(f'Client request - Other error occurred: {err}')  # Python 3.6

result = lib.pactffi_mock_server_matched(mock_server_port)
print(f"Pact - Got matching client requests: {result}")
if result == True:
    PACT_FILE_DIR='./pacts'
    print(f"Writing pact file to {PACT_FILE_DIR}")
    res_write_pact = lib.pactffi_write_pact_file(mock_server_port, PACT_FILE_DIR.encode('ascii'), False)
    print(f"Pact file writing results: {res_write_pact}")
else:
    print('pactffi_mock_server_matched did not match')
    mismatchers = lib.pactffi_mock_server_mismatches(mock_server_port)
    result = json.loads(ffi.string(mismatchers))
    print(json.dumps(result, indent=4))
    logs = lib.pactffi_mock_server_logs(mock_server_port)
    print(logs)

## Cleanup

lib.pactffi_cleanup_mock_server(mock_server_port)

