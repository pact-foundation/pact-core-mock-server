<?php

require __DIR__ . '/../vendor/autoload.php';

use Symfony\Component\Process\Process;

$process = new Process(['php', '-S', 'localhost:8000', '-t', __DIR__ . '/../public', __DIR__ . '/../public/proxy.php']);
$process->start();
$process->waitUntil(function ($type, $output) {
    return false !== strpos($output, 'Development Server (http://localhost:8000) started');
});

$code = file_get_contents(__DIR__ . '/../../rust/pact_ffi/include/pact.h');
$ffi = FFI::cdef($code, __DIR__ . '/../../rust/target/debug/libpact_ffi.so');

$ffi->pactffi_init('LOG_LEVEL');

$tags = ['feature-x', 'master', 'test', 'prod'];
$consumers = ['http-consumer-1', 'http-consumer-2'];

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
$ffi->pactffi_verifier_set_provider_info($handle, 'http-provider', 'http', 'localhost', 8000, '/');
$ffi->pactffi_verifier_set_provider_state($handle, 'http://localhost:8000/change-state', true, true);
$ffi->pactffi_verifier_set_verification_options($handle, false, '1.0.0', null, false, 5000, getCData($tags), count($tags));
$ffi->pactffi_verifier_set_consumer_filters($handle, getCData($consumers), count($consumers));
$ffi->pactffi_verifier_add_directory_source($handle, __DIR__ . '/../pact');
$result = $ffi->pactffi_verifier_execute($handle);
$ffi->pactffi_verifier_shutdown($handle);

if (!$result) {
    echo "Verifier verified all contracts, Yay!\n";
} else {
    echo "We got some problems, Boo!\n";
}
