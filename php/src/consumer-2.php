<?php

require __DIR__ . '/../vendor/autoload.php';

use Symfony\Component\HttpClient\HttpClient;

$code = file_get_contents(__DIR__ . '/../../rust/pact_ffi/include/pact.h');
$ffi = FFI::cdef($code, __DIR__ . '/../../rust/target/debug/libpact_ffi.so');
// Macs use dylib extension, following will assume os's downloaded in users home dir ~/.pact/ffi/arch/libpact_ffi.<dylib|so>
// $code = file_get_contents(posix_getpwnam(get_current_user())['dir'] . '/.pact/ffi/pact.h');
// $ffi = FFI::cdef($code, posix_getpwnam(get_current_user())['dir'] . '/.pact/ffi/osxaarch64/libpact_ffi.dylib');

$ffi->pactffi_init('LOG_LEVEL');

$pact = $ffi->pactffi_new_pact('http-consumer-2', 'http-provider');
$ffi->pactffi_with_specification($pact, $ffi->PactSpecification_V3);

$interaction = $ffi->pactffi_new_interaction($pact, 'A PUT request to generate book cover');
$ffi->pactffi_upon_receiving($interaction, 'A PUT request to generate book cover');
$ffi->pactffi_given($interaction, 'A book with id fb5a885f-f7e8-4a50-950f-c1a64a94d500 is required');
$ffi->pactffi_with_request($interaction, 'PUT', '/api/books/fb5a885f-f7e8-4a50-950f-c1a64a94d500/generate-cover');
$ffi->pactffi_with_header($interaction, $ffi->InteractionPart_Request, 'Content-Type', 0, 'application/json');
$ffi->pactffi_with_body($interaction, $ffi->InteractionPart_Request, 'application/json', '[]');
$ffi->pactffi_response_status($interaction, 204);

$contents = '{
    "uuid": {
        "pact:matcher:type": "regex",
        "regex": "^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$",
        "value": "fb5a885f-f7e8-4a50-950f-c1a64a94d500"
    }
}';
$length = \strlen($contents);
$size   = $length + 1;
$cData  = $ffi->new("uint8_t[{$size}]");
FFI::memcpy($cData, $contents, $length);

$messagePact = $ffi->pactffi_new_message_pact('message-consumer-2', 'message-provider');
$message = $ffi->pactffi_new_message($messagePact, 'Book (id fb5a885f-f7e8-4a50-950f-c1a64a94d500) created message');
$ffi->pactffi_message_expects_to_receive($message, 'Book (id fb5a885f-f7e8-4a50-950f-c1a64a94d500) created message');
$ffi->pactffi_message_given($message, 'A book with id fb5a885f-f7e8-4a50-950f-c1a64a94d500 is required');
$ffi->pactffi_message_with_contents($message, 'application/json', $cData, $size);

$port = $ffi->pactffi_create_mock_server_for_pact($pact, '127.0.0.1:0', false);
echo sprintf("Mock server port=%d\n", $port);

$messageHandler = function ($message) use ($port) {
    if (!isset($message->uuid)) {
        return;
    }

    $client = HttpClient::create();

    $response = $client->request(
        'PUT',
        sprintf('http://localhost:%d/api/books/%s/generate-cover', $port, $message->uuid),
        [
            'json' => getenv('MATCHING') ? [] : [
                'width' => '720',
                'height' => '1080'
            ],
        ]
    );

    echo sprintf("STATUS: %d\n", $response->getStatusCode());
    echo sprintf("HEADERS: %s\n", print_r($response->getHeaders(false), true));
    echo sprintf("BODY: %s\n", print_r(json_decode($response->getContent(false), true), true));
};

$reified = $ffi->pactffi_message_reify($message);
$raw = json_decode($reified, false);
$messageHandler($raw->contents);

if ($ffi->pactffi_mock_server_matched($port)) {
    echo getenv('MATCHING') ? "Mock server matched all requests, Yay!" : "Mock server matched all requests, That Is Not Good (tm)";
    echo "\n";

    $ffi->pactffi_write_pact_file($port, __DIR__ . '/../pacts', false);
    $ffi->pactffi_write_message_pact_file($messagePact, __DIR__ . '/../pacts', false);
} else {
    echo getenv('MATCHING') ? "We got some mismatches, Boo!" : "We got some mismatches, as expected.";
    echo "\n";
    echo sprintf("Mismatches: %s\n", print_r(json_decode(FFI::string($ffi->pactffi_mock_server_mismatches($port)), true), true));
}

$ffi->pactffi_cleanup_mock_server($port);
