import requests
import xml.etree.ElementTree as ET
from cffi import FFI
from register_ffi import get_ffi_lib
import json
import requests
ffi = FFI()

lib = get_ffi_lib(ffi) # loads the entire C namespace
lib.pactffi_logger_init()
lib.pactffi_logger_attach_sink(b'stdout', 5)
lib.pactffi_logger_apply()
version_encoded = lib.pactffi_version()
lib.pactffi_log_message(b'pact_python_ffi', b'INFO', b'hello from pact python ffi, using Pact FFI Version: '+ ffi.string(version_encoded))

expected_response_body = '''<?xml version="1.0" encoding="UTF-8"?>
    <projects>
    <item>
    <id>1</id>
    <tasks>
        <item>
            <id>1</id>
            <name>Do the laundry</name>
            <done>true</done>
        </item>
        <item>
            <id>2</id>
            <name>Do the dishes</name>
            <done>false</done>
        </item>
        <item>
            <id>3</id>
            <name>Do the backyard</name>
            <done>false</done>
        </item>
        <item>
            <id>4</id>
            <name>Do nothing</name>
            <done>false</done>
        </item>
    </tasks>
    </item>
    </projects>'''
format = 'xml'
content_type =  'application/' + format
pact_handle = lib.pactffi_new_pact(b'consumer',b'provider')
lib.pactffi_with_pact_metadata(pact_handle, b'pact-python', b'version', b'1.0.0')
interaction_handle = lib.pactffi_new_interaction(pact_handle, b'description')
lib.pactffi_given(interaction_handle, b'i have a list of projects')
lib.pactffi_upon_receiving(interaction_handle, b'a request for projects in XML')
lib.pactffi_with_request(interaction_handle, b'GET', b'/projects')
lib.pactffi_with_header_v2(interaction_handle, 0, b'Accept', 0, content_type.encode('ascii'))

# lib.pactffi_with_header_v2(interaction_handle, 1, b'Content-Type', 0, content_type.encode('ascii'))
# lib.pactffi_with_header_v2(interaction_handle, 1, b'content-type', 1, content_type.encode('ascii'))
lib.pactffi_with_body(interaction_handle, 1, content_type.encode('ascii'), expected_response_body.encode('ascii'))

mock_server_port = lib.pactffi_create_mock_server_for_transport(pact_handle, b'127.0.0.1', 0, b'http', b'{}')
print(f"Mock server started: {mock_server_port}")
try:
    uri = f"http://127.0.0.1:{mock_server_port}/projects"
    response = requests.get(uri,
                            headers={'Accept': content_type})
    response.raise_for_status()
except requests.HTTPError as http_err:
    print(f'Client request - HTTP error occurred: {http_err}')  # Python 3.6
except Exception as err:
    print(f'Client request - Other error occurred: {err}')  # Python 3.6

# Check the client made the right request

result = lib.pactffi_mock_server_matched(mock_server_port)
print(f"Pact - Got matching client requests: {result}")
if result == True:
    PACT_FILE_DIR='./pacts'
    print(f"Writing pact file to {PACT_FILE_DIR}")
    res_write_pact = lib.pactffi_write_pact_file(mock_server_port, PACT_FILE_DIR.encode('ascii'), False)
    print(f"Pact file writing results: {res_write_pact}")
else:
    print('pactffi_mock_server_matched did not match')
    mismatches = lib.pactffi_mock_server_mismatches(mock_server_port)
    result = json.loads(ffi.string(mismatches))
    print(json.dumps(result, indent=4))
    native_logs = lib.pactffi_mock_server_logs(mock_server_port)
    logs = ffi.string(native_logs).decode("utf-8").rstrip().split("\n")
    print(logs)

## Cleanup

lib.pactffi_cleanup_mock_server(mock_server_port)
assert result == True
print(f"Client request - matched: {response.text}")
# Check our response came back from the provider ok

assert response.text != '' # This should always have a response
projects = ET.fromstring(response.text)
assert len(projects) == 1
assert projects[0][0].text == '1'
tasks = projects[0].findall('tasks')[0]
assert len(tasks) == 4
assert tasks[0][0].text == '1'
# assert tasks[0][1].text == 'Do the laundry'
print(f"Client response - matched: {response.text}")
print(f"Client response - matched: {response.text == expected_response_body}")
