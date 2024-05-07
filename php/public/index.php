<?php

require __DIR__ . '/../vendor/autoload.php';

use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Slim\Factory\AppFactory;

$app = AppFactory::create();

$app->post('/api/books', function (Request $request, Response $response, $args) {
    $response->getBody()->write(json_encode([
        '@context' => '/api/contexts/Book',
        '@id' => '/api/books/bb50b187-ff02-422c-886f-b58dc4e0adca',
        '@type' => 'Book',
        'title' => 'Lorem ipsum dolor sit amet.',
        'description' => 'Lorem ipsum dolor sit amet, consectetur adipiscing elit. Mauris a neque erat. Donec laoreet justo.',
        'author' => 'Mrs. Samanta Gerhold',
        'publicationDate' => '2002-05-26T11:41:12+07:00',
        'reviews' => [],
    ]));

    return $response
        ->withHeader('Content-Type', 'application/ld+json; charset=utf-8')
        ->withStatus(201);
});

$app->put('/api/books/{id}/generate-cover', function (Request $request, Response $response, $args) {
    $response->getBody()->write('[]');

    return $response->withStatus(204);
});

$app->run();
