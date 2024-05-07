<?php

require __DIR__ . '/../vendor/autoload.php';

use Symfony\Component\Process\Process;

$process = new Process(['php', '-S', 'localhost:8000', '-t', __DIR__ . '/../public', __DIR__ . '/../public/proxy.php']);
$process->start();
$process->waitUntil(function ($type, $output) {
    return false !== strpos($output, 'Development Server (http://localhost:8000) started');
});

$code = file_get_contents(__DIR__ . '/../../rust/pact_ffi/include/pact.h');
$ffi = FFI::cdef($code, __DIR__ . '/../../rust/target/debug/libpact_ffi.dylib');
// Macs use dylib extension, following will assume os's downloaded in users home dir ~/.pact/ffi/arch/libpact_ffi.<dylib|so>
// $code = file_get_contents(posix_getpwnam(get_current_user())['dir'] . '/.pact/ffi/pact.h');
// $ffi = FFI::cdef($code, posix_getpwnam(get_current_user())['dir'] . '/.pact/ffi/osxaarch64/libpact_ffi.dylib');

$ffi->pactffi_init('LOG_LEVEL');

$tags = ['feature-x', 'master', 'test', 'prod'];
$consumers = ['http-consumer-1', 'http-consumer-2', 'message-consumer-2','area-calculator-consumer'];

function getCData(array $items): FFI\CData
{
    $itemsSize = count($items);
    $cDataItems  = FFI::new("char*[{$itemsSize}]");
    foreach ($items as $index => $item) {
        $length = \strlen($item);
        $itemSize   = $length + 1;
        $cDataItem  = FFI::new("char[{$itemSize}]", false);
        FFI::memcpy($cDataItem, $item, $length);
        $cDataItems[$index] = $cDataItem;
    }

    return $cDataItems;
}

$handle = $ffi->pactffi_verifier_new();

//  // gRPC ❌
//  // HTTP ✅
//  // Verification failed with an error - Failed to verify the request: gRPC error: status Unknown error, message 'transport error'
// $ffi->pactffi_verifier_set_provider_info($handle, 'http-provider', 'http', 'localhost', 8000, '/');


//  // gRPC ✅
//  // HTTP ❌
//  // Request Failed - builder error for url (tcp://localhost:37757): URL scheme is not allowed
// $ffi->pactffi_verifier_set_provider_info($handle, 'http-provider', 'tcp', 'localhost', 37757, '/');

// //  // gRPC ✅
// //  // HTTP ❌
// //  // Request Failed - builder error for url (tcp://localhost:37757): URL scheme is not allowed
// $ffi->pactffi_verifier_set_provider_info($handle, 'http-provider', 'tcp', 'localhost', 37757, '/');
// $ffi->pactffi_verifier_add_provider_transport($handle, 'http',8000,'/','http');

//  // gRPC ✅
//  // HTTP ❌
//  // Verification failed with an error - Failed to verify the request: gRPC error: status Unknown error, message 'transport error'
$ffi->pactffi_verifier_set_provider_info($handle, 'http-provider', 'http', 'localhost', 8000, '/');
$ffi->pactffi_verifier_add_provider_transport($handle, 'protobuf',37757,'/','tcp');


//  // gRPC ✅
//  // HTTP ❌
//  // Request Failed - builder error for url (tcp://localhost:37757): URL scheme is not allowed
// $ffi->pactffi_verifier_set_provider_info($handle, 'http-provider', 'tcp', 'localhost', 37757, '/');
// $ffi->pactffi_verifier_add_provider_transport($handle, 'http',8000,'/','http');


//  // 💡
//  // This would be my preferrred option
//  // Set the provider name (which should be used by anything using the verifier_handle, and filter sourced pacts that don't contain name)
//  // add multiple transports.
//  // note pactffi_verifier_set_provider_name does not exist
//  // update_provider_info might work
//  // https://github.com/pact-foundation/pact-reference/blob/cfb2c03f87b3f67464291dd936d0aac555c42c91/rust/pact_ffi/src/verifier/handle.rs#L89
//  // but is marked as deprecated.
//  //
//  // also worthy of note, if pactffi_verifier_set_provider_info didn't mix with the information used in pactffi_verifier_add_provider_transport
//  // this probably wouldn't be neccessary.
// $ffi->pactffi_verifier_set_provider_name($handle, 'http-provider'); // note this function doesn't exist (wishlist)
// $ffi->pactffi_verifier_add_provider_transport($handle, 'http',8000,'/','http');
// $ffi->pactffi_verifier_add_provider_transport($handle, 'protobuf',37757,'/','tcp');


 // gRPC ❌
 // HTTP ❌
 // You can't just pass nulls into set_provider_info as it provides default info
 // https://github.com/pact-foundation/pact-reference/blob/master/rust/pact_ffi/src/verifier/mod.rs#L143
$ffi->pactffi_verifier_set_provider_info($handle, 'http-provider',  null,  null, null, null);
$ffi->pactffi_verifier_add_provider_transport($handle, 'protobuf',37757,'/','tcp');
// $ffi->pactffi_verifier_add_provider_transport($handle, 'http',8000,'/','http'); // registering a http transport doesnt work either




// $ffi->pactffi_verifier_set_filter_info($handle, '', 'book', false);
$ffi->pactffi_verifier_set_provider_state($handle, 'http://localhost:8000/change-state', true, true);
$ffi->pactffi_verifier_set_verification_options($handle, false, 5000);
$ffi->pactffi_verifier_set_publish_options($handle, '1.0.0', null, getCData($tags), count($tags), 'some-branch');
$ffi->pactffi_verifier_set_consumer_filters($handle, getCData($consumers), count($consumers));
// $ffi->pactffi_verifier_add_provider_transport($handle, 'protobuf',37757,null,'tcp');
$ffi->pactffi_verifier_add_directory_source($handle, __DIR__ . '/../pacts');
$result = $ffi->pactffi_verifier_execute($handle);
$ffi->pactffi_verifier_shutdown($handle);

if (!$result) {
    echo "Verifier verified all contracts, Yay!\n";
} else {
    echo "We got some problems, Boo!\n";
}
