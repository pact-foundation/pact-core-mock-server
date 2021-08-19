<?php

require __DIR__ . '/../vendor/autoload.php';

use Symfony\Component\Process\Process;

$process = new Process(['php', '-S', 'localhost:8000', '-t', __DIR__ . '/../public', __DIR__ . '/../public/proxy.php']);
$process->start();
$process->waitUntil(function ($type, $output) {
    return false !== strpos($output, 'Development Server (http://localhost:8000) started');
});

$args = sprintf("--dir
%s
--hostname
localhost
--port
8000
--state-change-url
http://localhost:8000/change-state
--filter-consumer
http-consumer-1
--filter-consumer
http-consumer-2
--filter-consumer
message-consumer-2", __DIR__ . '/../pact');
$code = file_get_contents(__DIR__ . '/../../rust/pact_ffi/include/pact.h');
$ffi = FFI::cdef($code, __DIR__ . '/../../rust/target/debug/libpact_ffi.so');

$ffi->pactffi_init('LOG_LEVEL');

$result = $ffi->pactffi_verify($args);

if (!$result) {
    echo "Verifier verified all contracts, Yay!\n";
} else {
    echo "We got some problems, Boo!\n";
}
