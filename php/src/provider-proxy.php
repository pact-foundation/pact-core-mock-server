<?php

require __DIR__ . '/../vendor/autoload.php';

use \Psr\Http\Message\ServerRequestInterface as Request;
use \Psr\Http\Message\ResponseInterface as Response;
use GuzzleHttp\Client;
use Slim\Factory\AppFactory;

$client = new Client();

$app = AppFactory::create();
$app->addBodyParsingMiddleware();

$app->any('{path:.*}', function(Request $request, Response $response) use ($client) {
    if ($request->getRequestTarget() === '/' && $request->getMethod() === 'POST') {
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
    }

    if ($request->getRequestTarget() === '/change-state' && $request->getMethod() === 'POST') {
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
    }

    return $client->send(
        $request->withUri(
            $request->getUri()->withHost('localhost')->withPort(8001)
        )
    );
});

$app->run();
