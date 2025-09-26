<?php

require __DIR__ . '/vendor/autoload.php';
require __DIR__ . '/cli/HttpResponse.php';
require __DIR__ . '/cli/HttpClient.php';
require __DIR__ . '/cli/MinificationStats.php';
require __DIR__ . '/cli/MinifierComparison.php';
require __DIR__ . '/cli/UrlMinifier.php';

use Wexowgt\Minifier\HTMLMinifier;

function initializeMinifier(): HTMLMinifier
{
    try {
        return HTMLMinifier::getInstance();
    } catch (\RuntimeException $e) {
        echo "\n[ERROR] " . $e->getMessage() . "\n";
        echo "[INFO] Run 'bash build.sh' first to build the shared library.\n";
        exit(1);
    }
}

function handleUrlProcessing(string $url, HTMLMinifier $minifier, bool $shouldSave): void
{
    if (!filter_var($url, FILTER_VALIDATE_URL)) {
        echo "\n[ERROR] Invalid URL provided\n";
        echo "\nUsage:\n";
        echo "   php run.php <url> [--save]\n";
        echo "\nExample: php run.php https://example.com --save\n";
        exit(1);
    }

    $urlMinifier = new UrlMinifier($minifier);
    $urlMinifier->minifyUrl($url, $shouldSave);
}

function runDefaultExample(HTMLMinifier $minifier): void
{
    echo "\nHTML Minifier Demo\n";
    echo "\nAvailable commands:\n";
    echo "   • Minify URL: php run.php <url>\n";
    echo "   • Save files: add --save flag\n";
    echo "\nExample: php run.php https://example.com --save\n";

    $html = <<<HTML
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>   My Page   </title>
    <style>
        body {
            background-color: #f0f0f0;
        }
    </style>
</head>
<body>
    <h1>  Hello, world!  </h1>
    <p>
        This is a
        sample HTML
        page.
    </p>
    <script>
        function doSomething() {
            console.log("This is some JS.");
        }
    </script>
</body>
</html>
HTML;

    $startTime = microtime(true);
    $minifiedHtml = $minifier->minify($html);
    $endTime = microtime(true);

    $stats = new MinificationStats(
        strlen($html),
        strlen($minifiedHtml),
        ($endTime - $startTime) * 1000
    );

    $stats->display();
}

$minifier = initializeMinifier();
$shouldSave = in_array('--save', $argv);

if (isset($argv[1]) && !empty($argv[1]) && !str_starts_with($argv[1], '--')) {
    handleUrlProcessing($argv[1], $minifier, $shouldSave);
} else {
    runDefaultExample($minifier);
}