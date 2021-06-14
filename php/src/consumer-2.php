<?php

require __DIR__ . '/../vendor/autoload.php';

use Symfony\Component\HttpClient\HttpClient;

$code = file_get_contents(__DIR__ . '/../lib/pact_mock_server_ffi.h');
$ffi = FFI::cdef($code, __DIR__ . '/../../rust/target/debug/libpact_mock_server_ffi.so');

$ffi->init('LOG_LEVEL');

$pact = $ffi->new_pact('http-consumer-2', 'provider');
$interaction = $ffi->new_interaction($pact, 'A PUT request to generate book cover');
$ffi->upon_receiving($interaction, 'A PUT request to generate book cover');
$ffi->given($interaction, 'Book Fixtures Loaded');
$ffi->with_request($interaction, 'PUT', '/api/books/fb5a885f-f7e8-4a50-950f-c1a64a94d500/generate-cover');
$ffi->with_header($interaction, $ffi->Request, 'Content-Type', 0, 'application/json');
$ffi->with_body($interaction, $ffi->Request, 'application/json', '[]');
$ffi->response_status($interaction, 204);

$messagePact = $ffi->new_message_pact('message-consumer-2', 'provider');
$message = $ffi->new_message($messagePact, 'Book Created');
$ffi->message_expects_to_receive($message, 'Book Created');
$ffi->message_given($message, 'Provider has book');
$ffi->message_with_contents($message, 'application/json', '{
    "uuid": {
        "pact:matcher:type": "regex",
        "regex": "^[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$",
        "value": "fb5a885f-f7e8-4a50-950f-c1a64a94d500"
    }
}', 36); // size of uuid

$port = $ffi->create_mock_server_for_pact($pact, '127.0.0.1:0', false);
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

$reified = $ffi->message_reify($message);
$raw = json_decode($reified, false);
$messageHandler($raw->contents);

if ($ffi->mock_server_matched($port)) {
    echo getenv('MATCHING') ? "Mock server matched all requests, Yay!" : "Mock server matched all requests, That Is Not Good (tm)";
    echo "\n";

    $ffi->write_pact_file($port, __DIR__ . '/../pact', true);
    $ffi->write_message_pact_file($messagePact, __DIR__ . '/../pact', true);
} else {
    echo getenv('MATCHING') ? "We got some mismatches, Boo!" : "We got some mismatches, as expected.";
    echo "\n";
    echo sprintf("Mismatches: %s\n", print_r(json_decode(FFI::string($ffi->mock_server_mismatches($port)), true), true));
}

$ffi->cleanup_mock_server($port);
