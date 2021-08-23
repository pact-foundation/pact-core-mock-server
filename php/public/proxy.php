<?php

require __DIR__ . '/../vendor/autoload.php';

use Psr\Http\Message\ServerRequestInterface as Request;
use Psr\Http\Message\ResponseInterface as Response;
use Slim\Exception\HttpNotFoundException;
use Slim\Factory\AppFactory;

$app = AppFactory::create();
$app->addBodyParsingMiddleware();

$app->post('/', function(Request $request, Response $response) {
    $body = $request->getParsedBody();
    switch ($body['description']) {
        case 'Book Created':
            $response->getBody()->write(json_encode([
                'uuid' => '90d0f930-b1c6-48b6-b351-88f6c2b5aa9e',
            ]));
            return $response->withHeader('Content-Type', 'application/json');

        default:
            break;
    }
    // What to do with $body['providerStates'] ?

    return $response;
});

$app->post('/change-state', function(Request $request, Response $response) {
    $body = $request->getParsedBody();
    switch ($body['state']) {
        case 'Book Fixtures Loaded':
            if (($body['action'] ?? null) === 'teardown') {
                error_log('Removing book fixtures...');
            } else {
                error_log('Creating book fixtures...');
            }
            break;

        default:
            break;
    }

    return $response;
});

try {
    $app->run();
} catch (HttpNotFoundException $exception) {
    return false;
}
