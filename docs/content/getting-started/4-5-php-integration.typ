#import "/src/boxes.typ": *
#import "/src/responsive-image.typ": *
#import "/src/code.typ": *

In this chapter, you'll integrate Oicana into a PHP web service using #link("https://www.slimframework.com/")[Slim]. Slim is a lightweight PHP micro-framework well-suited for building APIs and web services. We'll create a simple web service that compiles your Oicana template to PDF and serves it via an HTTP endpoint.

\
#note[This chapter assumes that you have a working PHP 8.3+ setup with Composer. If that is not the case, please follow #link("https://www.php.net/manual/en/install.php")[the official PHP installation guide] and #link("https://getcomposer.org/doc/00-intro.md")[the Composer installation guide] to set up your environment.]

\
Let's start with a fresh Slim project. Create a new directory for your PHP project (separate from your template directory) and initialize it with Composer:

```bash
mkdir my-pdf-service
cd my-pdf-service
composer init --no-interaction
```

Install Slim with its PSR-7 implementation:

```bash
composer require slim/slim slim/psr7
```

Create an `index.php` file with the following basic Slim application:

\
#code("index.php", ```php
<?php
declare(strict_types=1);

use Psr\Http\Message\ResponseInterface as Response;
use Psr\Http\Message\ServerRequestInterface as Request;
use Slim\Factory\AppFactory;

require __DIR__ . '/vendor/autoload.php';

$app = AppFactory::create();

$app->get('/', function (Request $request, Response $response) {
    $response->getBody()->write('Hello World');
    return $response;
});

$app->run();
```)

You can test it by running ```bash php -S localhost:8000 index.php``` and navigating to #link("http://localhost:8000") in your browser. You should see "Hello World".

== New service endpoint

We will define a new endpoint to compile our Oicana template to a PDF and return the PDF file to the user.

\
1. Create a new directory in the PHP project called `templates` and copy `example-0.1.0.zip` into that directory.
2. Add the `oicana/oicana` Composer package as a dependency. The package is hosted on a custom Composer repository, so add the following to your `composer.json`:

  ```json
  {
      "repositories": [
          {
              "type": "composer",
              "url": "https://composer.oicana.com"
          }
      ],
      "require": {
          "oicana/oicana": "^0.1.0-alpha.1"
      }
  }
  ```

  Then configure the minimum stability, allow the Oicana installer plugin, and install:

  ```bash
  composer config minimum-stability alpha
  composer config allow-plugins.oicana/installer true
  composer install
  ```

  \
  #note[
    The installer downloads the native extension binary into `vendor/` and writes a PHP ini file at `vendor/oicana/installer/php/oicana.ini`. At the end of the install output you will see the command to activate the extension — it looks like this:

    ```bash
    export PHP_INI_SCAN_DIR=":/path/to/my-pdf-service/vendor/oicana/installer/php"
    ```

    Set this environment variable (with your actual project path) before starting PHP. Verify the extension is loaded with: ```bash php -m | grep oicana```

    You can re-display this command at any time with: ```bash vendor/bin/oicana-env```
  ]

3. Before proceeding, make sure your template is packaged. Navigate to your template directory and run:

  ```bash
  cd /path/to/your/template
  oicana pack
  ```

  \
  #note[The `oicana pack` command will create `example-0.1.0.zip` in the template directory. The command produces no output on success - you'll only see the .zip file appear.]

4. Update `index.php` to load the template at startup and add a compile endpoint:

  \
  #code("index.php", ```php
  <?php
  declare(strict_types=1);

  use Oicana\CompilationMode;
  use Oicana\Template;
  use Psr\Http\Message\ResponseInterface as Response;
  use Psr\Http\Message\ServerRequestInterface as Request;
  use Slim\Factory\AppFactory;

  require __DIR__ . '/vendor/autoload.php';

  // Load the template at startup
  $templateBytes = file_get_contents('templates/example-0.1.0.zip');
  if ($templateBytes === false) {
      die('Failed to load template file. Make sure templates/example-0.1.0.zip exists.');
  }
  $template = new Template($templateBytes);

  $app = AppFactory::create();

  $app->post('/compile', function (Request $request, Response $response) use ($template) {
      $pdf = $template->compile(mode: CompilationMode::Development);

      $response->getBody()->write($pdf);
      return $response
          ->withHeader('Content-Type', 'application/pdf')
          ->withHeader('Content-Disposition', 'attachment; filename="example.pdf"');
  });

  $app->run();
  ```)

  \
  This code loads the template once at application startup. The `/compile` endpoint compiles the template and returns the PDF file. We explicitly pass the compilation mode with ```php $template->compile(mode: CompilationMode::Development)``` so the template uses the development value you defined for the `info` input (```json { "name": "Chuck Norris" }```). In a follow-up step, we will set an input value instead.

Start the PHP development server with `PHP_INI_SCAN_DIR` set so the extension is loaded:

```bash
export PHP_INI_SCAN_DIR=":/path/to/my-pdf-service/vendor/oicana/installer/php"
php -S localhost:8000 index.php
```

\
#note[PHP's built-in development server (```bash php -S```) is single-threaded and handles only one request at a time. This is fine for testing, but for production deployments, use a proper web server like Nginx or Apache with PHP-FPM for concurrent request handling.]

\
Test the endpoint with curl:

```bash
curl -X POST http://localhost:8000/compile --output example.pdf
```

The generated `example.pdf` file should contain your template with the development value.

== About performance

The PDF generation should not take longer than a couple of milliseconds. You can measure the compilation time with PHP's built-in ```php microtime(true)``` before and after the `compile` call.

== Passing inputs from PHP

Our `compile` endpoint is currently calling ```php $template->compile()``` with development mode. Now we'll provide explicit input values and switch to production mode:

#code(
  "Part of index.php",
  ```php
  $app->post('/compile', function (Request $request, Response $response) use ($template) {
      $pdf = $template->compile(
          jsonInputs: ['info' => ['name' => 'Baby Yoda']]
      );

      $response->getBody()->write($pdf);
      return $response
          ->withHeader('Content-Type', 'application/pdf')
          ->withHeader('Content-Disposition', 'attachment; filename="example.pdf"');
  });
  ```,
)

\
Notice that we removed the explicit ```php mode: CompilationMode::Development``` parameter. The ```php compile()``` method defaults to ```php CompilationMode::Production``` when no mode is specified. Production mode is the recommended default for all document compilation in your application - it ensures you never accidentally generate a document with test data. In production mode, the template will never fall back to development values. If an input value is missing in production mode and the input does not have a default value, the compilation will fail unless your template handles ```typst none``` values for that input.

\
Calling the endpoint now will result in a PDF with "Baby Yoda" instead of "Chuck Norris". Building on this minimal service, you could set input values based on database entries or the request payload. Take a look at the #link("https://github.com/oicana/oicana-example-php-slim/")[open source PHP example project on GitHub] for a more complete showcase of the Oicana PHP integration, including blob inputs, error handling, and request validation.
