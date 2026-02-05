<?php

declare(strict_types=1);

/*
|--------------------------------------------------------------------------
| Test Case
|--------------------------------------------------------------------------
|
| The closure you provide to your test functions is always bound to a specific PHPUnit test
| case class. By default, that class is "PHPUnit\Framework\TestCase". Of course, you may
| need to change it using the "uses()" function to bind a different classes or traits.
|
*/

// uses(Tests\TestCase::class)->in('Feature');

/*
|--------------------------------------------------------------------------
| Expectations
|--------------------------------------------------------------------------
|
| When you're writing tests, you often need to check that values meet certain conditions. The
| "expect()" function gives you access to a set of "expectations" methods that you can use
| to assert different things. Of course, you may extend the Expectation API at any time.
|
*/

expect()->extend('toStartWith', function (string $needle) {
    expect(str_starts_with($this->value, $needle))->toBeTrue();

    return $this;
});

/*
|--------------------------------------------------------------------------
| Functions
|--------------------------------------------------------------------------
|
| While Pest is very powerful out-of-the-box, you may have some testing code specific to your
| project that you don't want to repeat in every file. Here you can also expose helpers as
| global functions to help you to reduce the number of lines of code in your test files.
|
*/

function fixtures_path(string $path = ''): string
{
    return __DIR__ . '/tests/fixtures/' . ltrim($path, '/');
}

function e2e_template_path(): string
{
    return dirname(__DIR__, 4) . '/e2e-tests/template/oicana-e2e-test-x.y.z.zip';
}

function assets_path(string $path = ''): string
{
    return dirname(__DIR__, 4) . '/assets/' . ltrim($path, '/');
}
