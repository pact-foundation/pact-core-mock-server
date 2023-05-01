import sys
from cffi import FFI
from register_ffi import get_ffi_lib
import json
import os
from pathlib import Path
# sys.path.insert(0, './examples/area_calculator')
# from area_calculator_client import get_rectangle_area

ffi = FFI()
lib = get_ffi_lib(ffi) # loads the entire C namespace
version_encoded = lib.pactffi_version()
ffi_version = ffi.string(version_encoded).decode('utf-8')

contents =  {
       "pact:proto": os.path.abspath('../proto/area_calculator.proto'),
       "pact:proto-service": 'Calculator/calculateOne',
        "pact:content-type": 'application/protobuf',
        "request": {
          "rectangle": {
            "length": 'matching(number, 3)',
            "width": 'matching(number, 4)'
          }
        },
        "response": {
          "value": ['matching(number, 12)']
        }
      }

## Setup Loggers

lib.pactffi_logger_init()
lib.pactffi_logger_attach_sink(b'file ./logs/log-info.txt',5)
lib.pactffi_logger_attach_sink(b'file ./logs/log-error.txt',5)
lib.pactffi_logger_attach_sink(b'stdout', 3)
# lib.pactffi_logger_attach_sink(b'stderr', 5)
lib.pactffi_logger_apply()
lib.pactffi_log_message(b'pact_python_ffi', b'INFO', b'hello from pact python ffi, using Pact FFI Version: '+ ffi.string(version_encoded))


## Setup pact for testing
pact = lib.pactffi_new_pact(b'grpc-consumer-python', b'area-calculator-provider')
lib.pactffi_with_pact_metadata(pact, b'pact-python', b'ffi', ffi.string(version_encoded))
message_pact = lib.pactffi_new_sync_message_interaction(pact, b'A gRPC calculateMulti request')
lib.pactffi_with_specification(pact, 5)



# Start mock server
lib.pactffi_using_plugin(pact, b'protobuf', b'0.1.17')
lib.pactffi_interaction_contents(message_pact, 0, b'application/grpc', ffi.new("char[]", json.dumps(contents).encode('ascii')))
mock_server_port = lib.pactffi_create_mock_server_for_transport(pact , b'0.0.0.0',0,b'grpc',ffi.cast("void *", 0))
print(f"Mock server started: {mock_server_port}")

# ## Make our client call
# expected_response = 12.0
# response = get_rectangle_area(f"localhost:{mock_server_port}")
# print(f"Client response: {response}")
# print(f"Client response - matched expected: {response == expected_response}")

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
    if mismatchers:
        result = json.loads(ffi.string(mismatchers))
        print(json.dumps(result, indent=4))

# Cleanup
lib.pactffi_cleanup_mock_server(mock_server_port)
lib.pactffi_cleanup_plugins(pact)
