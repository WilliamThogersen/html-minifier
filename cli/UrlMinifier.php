<?php

use Wexowgt\Minifier\HTMLMinifier;

class UrlMinifier
{
    public function __construct(
        private HTMLMinifier $minifier
    ) {}

    public function minifyUrl(string $url, bool $shouldSave = false): bool
    {
        echo "\n[INFO] Fetching URL: $url\n";

        $response = HttpClient::fetch($url);

        if (!$response->isSuccess()) {
            $this->displayError($response);
            return false;
        }

        if (!$response->isHtml()) {
            echo "[WARNING] Content-Type doesn't appear to be HTML\n";
        }

        $this->processAndDisplay($response->getContent(), $url, $shouldSave);
        return true;
    }

    private function displayError(HttpResponse $response): void
    {
        if ($response->content === false || !empty($response->error)) {
            echo "[ERROR] Failed to fetch URL\n";
            echo "         cURL Error: {$response->error}\n";
            echo "         HTTP Code: {$response->httpCode}\n\n";
        } elseif ($response->httpCode !== 200) {
            echo "[ERROR] HTTP {$response->httpCode}\n";
            echo "         The server returned an error response.\n\n";
        }
    }

    private function processAndDisplay(string $html, string $url, bool $shouldSave): void
    {
        $startTime = microtime(true);
        $minified = $this->minifier->minify($html);
        $endTime = microtime(true);

        $stats = new MinificationStats(
            strlen($html),
            strlen($minified),
            ($endTime - $startTime) * 1000
        );

        if ($shouldSave) {
            $this->saveFiles($html, $minified, $url);
        }

        $stats->display("URL Minification Results");
    }

    private function saveFiles(string $original, string $minified, string $url): void
    {
        $filename = preg_replace('/[^a-z0-9]/i', '_', parse_url($url, PHP_URL_HOST));
        if (empty($filename)) {
            $filename = 'output';
        }

        $originalFilename = "{$filename}_original.html";
        $minifiedFilename = "{$filename}_minified.html";

        file_put_contents($originalFilename, $original);
        file_put_contents($minifiedFilename, $minified);

        echo "\n[SUCCESS] Files saved:\n";
        echo "          Original: $originalFilename\n";
        echo "          Minified: $minifiedFilename\n";
    }
}