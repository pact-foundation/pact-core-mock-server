from cffi import FFI
from register_ffi import get_ffi_lib
import json
import requests

ffi = FFI()
lib = get_ffi_lib(ffi) # loads the entire C namespace
version_encoded = lib.pactffi_version()
ffi_version = ffi.string(version_encoded).decode('utf-8')

contents = {
        "uuid": {
          "pact:matcher:type": 'regex',
          "regex": '^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$',
          "value": 'fb5a885f-f7e8-4a50-950f-c1a64a94d500'
        }
      }
## Setup Loggers

lib.pactffi_logger_init()
lib.pactffi_logger_attach_sink(b'file ./logs/log-info.txt',5)
lib.pactffi_logger_attach_sink(b'file ./logs/log-error.txt',5)
# lib.pactffi_logger_attach_sink(b'stdout', 5)
# lib.pactffi_logger_attach_sink(b'stderr', 5)
lib.pactffi_logger_apply()
lib.pactffi_log_message(b'pact_python_ffi', b'INFO', b'hello from pact python ffi, using Pact FFI Version: '+ ffi.string(version_encoded))


## Setup pact for testing
pact = lib.pactffi_new_pact(b'http-consumer-2', b'http-provider')
lib.pactffi_with_pact_metadata(pact, b'pact-python', b'ffi', ffi.string(version_encoded))
interaction = lib.pactffi_new_interaction(pact, b'A PUT request to generate book cover')
message_pact = lib.pactffi_new_pact(b'message-consumer-2', b'message-provider')
message = lib.pactffi_new_message(message_pact, b'Book (id fb5a885f-f7e8-4a50-950f-c1a64a94d500) created message')

# setup interaction request
lib.pactffi_upon_receiving(interaction, b'A PUT request to generate book cover')
lib.pactffi_given(interaction, b'A book with id fb5a885f-f7e8-4a50-950f-c1a64a94d500 is required')
lib.pactffi_with_request(interaction, b'PUT', b'/api/books/fb5a885f-f7e8-4a50-950f-c1a64a94d500/generate-cover')
lib.pactffi_with_header_v2(interaction, 0,b'Content-Type', 0, b'application/json')
lib.pactffi_with_body(interaction, 0,b'application/json', b'[]')
# setup interaction response
lib.pactffi_response_status(interaction, 204)
length = len(json.dumps(contents))
size = length + 1
# memBuf = FFI::MemoryPointer.new(:uint, length)
# memBuf.put_bytes(0, json.dump(contents))
lib.pactffi_message_expects_to_receive(message,b'Book (id fb5a885f-f7e8-4a50-950f-c1a64a94d500) created message')
lib.pactffi_message_given(message, b'A book with id fb5a885f-f7e8-4a50-950f-c1a64a94d500 is required')
lib.pactffi_message_with_contents(message, b'application/json', ffi.new("char[]", json.dumps(contents).encode('ascii')), size)
# Start mock server
mock_server_port = lib.pactffi_create_mock_server_for_pact(pact , b'0.0.0.0:0',0)
print(f"Mock server started: {mock_server_port}")
reified = lib.pactffi_message_reify(message)
uuid = json.loads(ffi.string(reified).decode('utf-8'))['contents']['uuid']
## Make our client call
body =  []
try:
    response = requests.put(f"http://127.0.0.1:{mock_server_port}/api/books/{uuid}/generate-cover", data=json.dumps(body),
    headers={'Content-Type': 'application/json'})
    print(f"Client response - matched: {response.text}")
    print(f"Client response - matched: {response.status_code}")
    print(f"Client response - matched: {response.status_code == '204'}")
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
    res_write_message_pact = lib.pactffi_write_message_pact_file(message_pact, PACT_FILE_DIR.encode('ascii'), False)
    print(f"Pact file writing results: {res_write_pact}")
else:
    print('pactffi_mock_server_matched did not match')
    mismatchers = lib.pactffi_mock_server_mismatches(mock_server_port)
    result = json.loads(ffi.string(mismatchers))
    print(json.dumps(result, indent=4))

## Cleanup
lib.pactffi_cleanup_mock_server(mock_server_port)
