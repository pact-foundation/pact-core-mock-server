from cffi import FFI
from register_ffi import get_ffi_lib
import json
import requests

ffi = FFI()
lib = get_ffi_lib(ffi) # loads the entire C namespace
version_encoded = lib.pactffi_version()
ffi_version = ffi.string(version_encoded).decode('utf-8')

request_interaction_body = {
        "isbn": {
            "pact:matcher:type": "type",
            "value": "0099740915"
        },
        "title": {
            "pact:matcher:type": "type",
            "value": "The Handmaid\'s Tale"
        },
        "description": {
            "pact:matcher:type": "type",
            "value": "Brilliantly conceived and executed, this powerful evocation of twenty-first century America gives full rein to Margaret Atwood\'s devastating irony, wit and astute perception."
        },
        "author": {
            "pact:matcher:type": "type",
            "value": "Margaret Atwood"
        },
        "publicationDate": {
            "pact:matcher:type": "regex",
            "regex": "^\\d{4}-[01]\\d-[0-3]\\dT[0-2]\\d:[0-5]\\d:[0-5]\\d([+-][0-2]\\d:[0-5]\\d|Z)$",
            "value": "1985-07-31T00:00:00+00:00"
        }
      }

# print(request_interaction_body)

response_interaction_body = {
        "@context": "/api/contexts/Book",
        "@id": {
            "pact:matcher:type": "regex",
            "regex": "^\\/api\\/books\\/[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$",
            "value": "/api/books/0114b2a8-3347-49d8-ad99-0e792c5a30e6"
        },
        "@type": "Book",
        "title": {
            "pact:matcher:type": "type",
            "value": "Voluptas et tempora repellat corporis excepturi."
        },
        "description": {
            "pact:matcher:type": "type",
            "value": "Quaerat odit quia nisi accusantium natus voluptatem. Explicabo corporis eligendi ut ut sapiente ut qui quidem. Optio amet velit aut delectus. Sed alias asperiores perspiciatis deserunt omnis. Mollitia unde id in."
        },
        "author": {
            "pact:matcher:type": "type",
            "value": "Melisa Kassulke"
        },
        "publicationDate": {
            "pact:matcher:type": "regex",
            "regex": "^\\d{4}-[01]\\d-[0-3]\\dT[0-2]\\d:[0-5]\\d:[0-5]\\d([+-][0-2]\\d:[0-5]\\d|Z)$",
            "value": "1999-02-13T00:00:00+07:00"
        },
        "reviews": [
        ]
      }

print(response_interaction_body)

## Setup Loggers

lib.pactffi_logger_init()
lib.pactffi_logger_attach_sink(b'file ./logs/log-info.txt',5)
lib.pactffi_logger_attach_sink(b'file ./logs/log-error.txt',5)
# lib.pactffi_logger_attach_sink(b'stdout', 5)
# lib.pactffi_logger_attach_sink(b'stderr', 5)
lib.pactffi_logger_apply()
lib.pactffi_log_message(b'pact_python_ffi', b'INFO', b'hello from pact python ffi, using Pact FFI Version: '+ ffi.string(version_encoded))


## Setup pact for testing
pact = lib.pactffi_new_pact(b'http-consumer-1', b'http-provider')
lib.pactffi_with_pact_metadata(pact, b'pact-python', b'ffi', ffi.string(version_encoded))
interaction = lib.pactffi_new_interaction(pact, b'A POST request to create book')
# setup interaction request
lib.pactffi_upon_receiving(interaction, b'A POST request to create book')
lib.pactffi_given(interaction, b'No book fixtures required')
lib.pactffi_with_request(interaction, b'POST', b'/api/books')
lib.pactffi_with_header_v2(interaction, 0,b'Content-Type', 0, b'application/json')
lib.pactffi_with_body(interaction, 0,b'application/json', ffi.new("char[]", json.dumps(request_interaction_body).encode('ascii')))
# setup interaction response
lib.pactffi_response_status(interaction, 200)
lib.pactffi_with_header_v2(interaction, 1,b'Content-Type', 0, b'application/ld+json; charset=utf-8')
lib.pactffi_with_body(interaction, 1,b'application/ld+json; charset=utf-8', ffi.new("char[]", json.dumps(response_interaction_body).encode('ascii')))

# Start mock server
mock_server_port = lib.pactffi_create_mock_server_for_pact(pact , b'0.0.0.0:0',0)
print(f"Mock server started: {mock_server_port}")

## Make our client call
body =  {
    "isbn": '0099740915',
    "title": "The Handmaid's Tale",
    "description": 'Brilliantly conceived and executed, this powerful evocation of twenty-first century America gives full rein to Margaret Atwood\'s devastating irony, wit and astute perception.',
    "author": 'Margaret Atwood',
    "publicationDate": '1985-07-31T00:00:00+00:00'
  }
expected_response = '{"@context":"/api/contexts/Book","@id":"/api/books/0114b2a8-3347-49d8-ad99-0e792c5a30e6","@type":"Book","author":"Melisa Kassulke","description":"Quaerat odit quia nisi accusantium natus voluptatem. Explicabo corporis eligendi ut ut sapiente ut qui quidem. Optio amet velit aut delectus. Sed alias asperiores perspiciatis deserunt omnis. Mollitia unde id in.","publicationDate":"1999-02-13T00:00:00+07:00","reviews":[],"title":"Voluptas et tempora repellat corporis excepturi."}'
try:
    response = requests.post(f"http://127.0.0.1:{mock_server_port}/api/books", data=json.dumps(body),
    headers={'Content-Type': 'application/json'})
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

## Cleanup
lib.pactffi_cleanup_mock_server(mock_server_port)
