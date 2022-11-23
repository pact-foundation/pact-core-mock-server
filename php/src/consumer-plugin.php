<?php

require __DIR__ . '/../vendor/autoload.php';

use Symfony\Component\HttpClient\HttpClient;

$code = file_get_contents(__DIR__ . '/../../rust/pact_ffi/include/pact.h');
$ffi = FFI::cdef($code, __DIR__ . '/../../rust/target/debug/libpact_ffi.so');
// Macs use dylib extension, following will assume os's downloaded in users home dir ~/.pact/ffi/arch/libpact_ffi.<dylib|so>
// $code = file_get_contents(posix_getpwnam(get_current_user())['dir'] . '/.pact/ffi/pact.h');
// $ffi = FFI::cdef($code, posix_getpwnam(get_current_user())['dir'] . '/.pact/ffi/osxaarch64/libpact_ffi.dylib');

// Setup Loggers
$ffi->pactffi_logger_init();
$ffi->pactffi_logger_attach_sink('file ./logs/log-info.txt',5);
$ffi->pactffi_logger_attach_sink('file ./logs/log-error.txt',5);
$ffi->pactffi_logger_attach_sink('stdout', 3);
$ffi->pactffi_logger_attach_sink('stderr', 2);
$ffi->pactffi_logger_apply();
$ffi->pactffi_log_message('pact-php-ffi', 'INFO', 'hello from pact php ffi, using Pact FFI Version: ' . $ffi->pactffi_version());

// Setup pact for testing
$pact = $ffi->pactffi_new_pact('grpc-consumer-php', 'area-calculator-provider');
$ffi->pactffi_with_pact_metadata($pact, 'pact-php','ffi',$ffi->pactffi_version());
$message_pact = $ffi->pactffi_new_sync_message_interaction($pact, 'A gRPC calculateOne request');
$ffi->pactffi_with_specification($pact, $ffi->PactSpecification_V4);

// Setup contents

$proto_file_path = __DIR__ . '/../../proto/area_calculator.proto';

$contents = '{
    "pact:proto":  "'. $proto_file_path . '",
    "pact:proto-service": "Calculator/calculateOne",
     "pact:content-type": "application/protobuf",
     "request": {
       "rectangle": {
         "length": "matching(number, 3)",
         "width": "matching(number, 4)"
       }
     },
     "response": {
       "value": ["matching(number, 12)"]
     }
   }';

// Start mock server

$ffi->pactffi_using_plugin($pact, 'protobuf', '0.1.17');
$ffi->pactffi_interaction_contents($message_pact, 0, 'application/grpc', $contents);
$port = $ffi->pactffi_create_mock_server_for_transport($pact , '0.0.0.0',0,'grpc', null);

echo sprintf("Mock server port=%d\n", $port);

// This is where we would call our client, gRPC in this example plugin demo
// PHP gRPC is client only, so would need to use a provider from the following
// https://github.com/pact-foundation/pact-plugins/tree/main/examples/gRPC/area_calculator
// TODO build out an area calculator PHP example
// https://grpc.io/docs/languages/php/

// Check if requests match - Note this _should_ fail, but the pactffi_mock_server_matched is returning true
// even if no requests were made.

if ($ffi->pactffi_mock_server_matched($port)) {
    echo getenv('MATCHING') ? "Mock server matched all requests, Yay!" : "Mock server matched all requests, That Is Not Good (tm)";
    echo "\n";

    $ffi->pactffi_write_pact_file($port, __DIR__ . '/../pacts', false);
    // $ffi->pactffi_write_message_pact_file($messagePact, __DIR__ . '/../pacts', false);
} else {
    echo getenv('MATCHING') ? "We got some mismatches, Boo!" : "We got some mismatches, as expected.";
    echo "\n";
    echo sprintf("Mismatches: %s\n", print_r(json_decode(FFI::string($ffi->pactffi_mock_server_mismatches($port)), true), true));
}

$ffi->pactffi_cleanup_mock_server($port);
$ffi->pactffi_cleanup_plugins($pact);
