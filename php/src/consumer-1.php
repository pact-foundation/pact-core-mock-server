<?php

require __DIR__ . '/../vendor/autoload.php';

use Symfony\Component\HttpClient\HttpClient;

$code = file_get_contents(__DIR__ . '/../../rust/pact_ffi/include/pact.h');
$ffi = FFI::cdef($code, __DIR__ . '/../../rust/target/debug/libpact_ffi.so');
// Macs use dylib extension, following will assume os's downloaded in users home dir ~/.pact/ffi/arch/libpact_ffi.<dylib|so>
// $code = file_get_contents(posix_getpwnam(get_current_user())['dir'] . '/.pact/ffi/pact.h');
// $ffi = FFI::cdef($code, posix_getpwnam(get_current_user())['dir'] . '/.pact/ffi/osxaarch64/libpact_ffi.dylib');

$ffi->pactffi_init('LOG_LEVEL');

$pact = $ffi->pactffi_new_pact('http-consumer-1', 'http-provider');
$ffi->pactffi_with_specification($pact, $ffi->PactSpecification_V3);

$interaction = $ffi->pactffi_new_interaction($pact, 'A POST request to create book');
$ffi->pactffi_upon_receiving($interaction, 'A POST request to create book');
$ffi->pactffi_given($interaction, 'No book fixtures required');
$ffi->pactffi_with_request($interaction, 'POST', '/api/books');
$ffi->pactffi_with_header($interaction, $ffi->InteractionPart_Request, 'Content-Type', 0, 'application/json');
$ffi->pactffi_with_body($interaction, $ffi->InteractionPart_Request, 'application/json', '{
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
        "regex": "^\\\\d{4}-[01]\\\\d-[0-3]\\\\dT[0-2]\\\\d:[0-5]\\\\d:[0-5]\\\\d([+-][0-2]\\\\d:[0-5]\\\\d|Z)$",
        "value": "1985-07-31T00:00:00+00:00"
    }
  }');
$ffi->pactffi_response_status($interaction, 201);
$ffi->pactffi_with_header($interaction, $ffi->InteractionPart_Response, 'Content-Type', 0, 'application/ld+json; charset=utf-8');
$ffi->pactffi_with_body($interaction, $ffi->InteractionPart_Response, 'application/ld+json; charset=utf-8', '{
    "@context": "/api/contexts/Book",
    "@id": {
        "pact:matcher:type": "regex",
        "regex": "^\\\\/api\\\\/books\\\\/[0-9a-f]{8}(-[0-9a-f]{4}){3}-[0-9a-f]{12}$",
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
        "regex": "^\\\\d{4}-[01]\\\\d-[0-3]\\\\dT[0-2]\\\\d:[0-5]\\\\d:[0-5]\\\\d([+-][0-2]\\\\d:[0-5]\\\\d|Z)$",
        "value": "1999-02-13T00:00:00+07:00"
    },
    "reviews": [

    ]
  }');

$port = $ffi->pactffi_create_mock_server_for_pact($pact, '127.0.0.1:0', false);
echo sprintf("Mock server port=%d\n", $port);

$client = HttpClient::create();

$json = getenv('MATCHING') ? [
    'isbn' => '0099740915',
    'title' => "The Handmaid's Tale",
    'description' => 'Brilliantly conceived and executed, this powerful evocation of twenty-first century America gives full rein to Margaret Atwood\'s devastating irony, wit and astute perception.',
    'author' => 'Margaret Atwood',
    'publicationDate' => '1985-07-31T00:00:00+00:00'
] : [
    'isbn' => '0099740915',
    'title' => 123,
    'description' => 'Natus ut doloribus magni. Impedit aperiam ea similique. Sed architecto quod nulla maxime. Quibusdam inventore esse harum accusantium rerum nulla voluptatem.',
    'author' => 'Maryse Kulas',
    'publicationDate' => 'tommorow'
];

$response = $client->request(
    'POST',
    sprintf('http://localhost:%d/api/books', $port),
    [
        'json' => $json,
    ]
);

echo sprintf("STATUS: %d\n", $response->getStatusCode());
echo sprintf("HEADERS: %s\n", print_r($response->getHeaders(false), true));
echo sprintf("BODY: %s\n", print_r(json_decode($response->getContent(false), true), true));

if ($ffi->pactffi_mock_server_matched($port)) {
    echo getenv('MATCHING') ? "Mock server matched all requests, Yay!" : "Mock server matched all requests, That Is Not Good (tm)";
    echo "\n";

    $ffi->pactffi_write_pact_file($port, __DIR__ . '/../pacts', false);
} else {
    echo getenv('MATCHING') ? "We got some mismatches, Boo!" : "We got some mismatches, as expected.";
    echo "\n";
    echo sprintf("Mismatches: %s\n", print_r(json_decode(FFI::string($ffi->pactffi_mock_server_mismatches($port)), true), true));
}

$ffi->pactffi_cleanup_mock_server($port);
