build_ffi:
	cd ../rust/pact_ffi && \
	cargo build && \
	rustup run nightly cbindgen  \
		--crate pact_ffi \
		--output include/pact.h

deps:
	pip install -r requires.txt

run_hello_ffi:
	python hello_ffi.py

run_pact_http:
	python pact_http_create_mock_server.py

run_pact_http_create_mock_server_for_pact:
	python pact_http_create_mock_server_for_pact.py

run_pact_message_v3:
	python pact_message_v3.py

run_pact_plugin_grpc_v4:
	python pact_plugin_grpc_v4.py

test: deps run_pact_http run_pact_http_create_mock_server_for_pact run_pact_message_v3 run_pact_plugin_grpc_v4

pacts_show:
	cat "pacts/Consumer-Alice Service.json" | jq .
	cat "pacts/http-consumer-1-http-provider.json" | jq .
	cat "pacts/http-consumer-2-http-provider.json" | jq .
	cat "pacts/message-consumer-2-message-provider.json" | jq .
	cat "pacts/grpc-consumer-python-area-calculator-provider.json" | jq .