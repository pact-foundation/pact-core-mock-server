<?php

require __DIR__ . '/../vendor/autoload.php';

use Symfony\Component\Process\Process;

$app = new Process(['php', '-S', 'localhost:8001', __DIR__ . '/provider-app.php']);
$app->start();
$app->waitUntil(function ($type, $output) {
    return false !== strpos($output, 'Development Server (http://localhost:8001) started');
});

$proxy = new Process(['php', '-S', 'localhost:8000', __DIR__ . '/provider-proxy.php']);
$proxy->start();
$proxy->waitUntil(function ($type, $output) {
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
consumer-1
--filter-consumer
http-consumer-2
--filter-consumer
message-consumer-2", __DIR__ . '/../pact');
$code = file_get_contents(__DIR__ . '/../lib/pact_verifier_ffi.h');
$ffi = FFI::cdef($code, __DIR__ . '/../../rust/target/debug/libpact_verifier_ffi.so');

$ffi->init('LOG_LEVEL');

$result = $ffi->verify($args);

if (!$result) {
    echo "Verifier verified all contracts, Yay!\n";
} else {
    echo "We got some problems, Boo!\n";
}
