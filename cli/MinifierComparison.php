<?php

use hexydec\html\htmldoc;
use Wexowgt\Minifier\HTMLMinifier;

class MinifierComparison
{
    public function __construct(
        private HTMLMinifier $rustMinifier
    ) {}

    public function compare(string $html): void
    {
        $rustStats = $this->benchmarkRustMinifier($html);
        $hexydecStats = $this->benchmarkHexydecMinifier($html);

        $rustStats->display("Rust Minifier");
        $hexydecStats->display("Hexydec Minifier");

        $this->displayComparison($rustStats, $hexydecStats);
        $this->saveComparisonFiles($html);
    }

    private function benchmarkRustMinifier(string $html): MinificationStats
    {
        $startTime = microtime(true);
        $minified = $this->rustMinifier->minify($html);
        $endTime = microtime(true);

        return new MinificationStats(
            strlen($html),
            strlen($minified),
            ($endTime - $startTime) * 1000
        );
    }

    private function benchmarkHexydecMinifier(string $html): MinificationStats
    {
        $startTime = microtime(true);
        $doc = new htmldoc();
        $doc->load($html);
        $doc->minify();
        $minified = $doc->html();
        $endTime = microtime(true);

        return new MinificationStats(
            strlen($html),
            strlen($minified),
            ($endTime - $startTime) * 1000
        );
    }

    private function displayComparison(MinificationStats $rust, MinificationStats $hexydec): void
    {
        $width = 52;
        echo "┌" . str_repeat("─", $width - 2) . "┐\n";
        echo "│ " . str_pad("Performance Comparison", $width - 4) . " │\n";
        echo "├" . str_repeat("─", $width - 2) . "┤\n";

        // Size comparison
        if ($rust->minifiedSize < $hexydec->minifiedSize) {
            $sizeDiff = $hexydec->minifiedSize - $rust->minifiedSize;
            $sizeDiffPercent = (($sizeDiff) / $hexydec->minifiedSize) * 100;
            $label = "Size winner";
            $value = "Rust (" . number_format($sizeDiffPercent, 1) . "% smaller)";
            $this->displayComparisonRow($label, $value, $width);
        } elseif ($hexydec->minifiedSize < $rust->minifiedSize) {
            $sizeDiff = $rust->minifiedSize - $hexydec->minifiedSize;
            $sizeDiffPercent = (($sizeDiff) / $rust->minifiedSize) * 100;
            $label = "Size winner";
            $value = "Hexydec (" . number_format($sizeDiffPercent, 1) . "% smaller)";
            $this->displayComparisonRow($label, $value, $width);
        } else {
            $this->displayComparisonRow("Size result", "Tie (identical)", $width);
        }

        // Speed comparison
        if ($rust->executionTimeMs < $hexydec->executionTimeMs) {
            $timeDiff = $hexydec->executionTimeMs - $rust->executionTimeMs;
            $timeDiffPercent = (($timeDiff) / $hexydec->executionTimeMs) * 100;
            $label = "Speed winner";
            $value = "Rust (" . number_format($timeDiffPercent, 1) . "% faster)";
            $this->displayComparisonRow($label, $value, $width);
        } elseif ($hexydec->executionTimeMs < $rust->executionTimeMs) {
            $timeDiff = $rust->executionTimeMs - $hexydec->executionTimeMs;
            $timeDiffPercent = (($timeDiff) / $rust->executionTimeMs) * 100;
            $label = "Speed winner";
            $value = "Hexydec (" . number_format($timeDiffPercent, 1) . "% faster)";
            $this->displayComparisonRow($label, $value, $width);
        } else {
            $this->displayComparisonRow("Speed result", "Tie (identical)", $width);
        }

        echo "└" . str_repeat("─", $width - 2) . "┘\n\n";
    }

    private function displayComparisonRow(string $label, string $value, int $width): void
    {
        $innerWidth = $width - 4; // Account for "│ " and " │"
        $dotsNeeded = $innerWidth - strlen($label) - strlen($value) - 1; // -1 for space before value

        echo "│ " . $label . str_repeat(".", $dotsNeeded) . " " . $value . " │\n";
    }

    private function formatBytes(int $bytes): string
    {
        if ($bytes >= 1024 * 1024) {
            return number_format($bytes / (1024 * 1024), 1) . "MB";
        } elseif ($bytes >= 1024) {
            return number_format($bytes / 1024, 1) . "KB";
        }
        return number_format($bytes) . "B";
    }

    private function saveComparisonFiles(string $html): void
    {
        if (isset($_SERVER['argv']) && in_array('--save-comparison', $_SERVER['argv'])) {
            $rustMinified = $this->rustMinifier->minify($html);

            $hexydecDoc = new htmldoc();
            $hexydecDoc->load($html);
            $hexydecDoc->minify();
            $hexydecMinified = $hexydecDoc->html();

            file_put_contents('rust_minified.html', $rustMinified);
            file_put_contents('hexydec_minified.html', $hexydecMinified);

            echo "[SUCCESS] Comparison files saved:\n";
            echo "          rust_minified.html\n";
            echo "          hexydec_minified.html\n\n";
        }
    }
}