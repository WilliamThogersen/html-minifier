<?php

class MinificationStats
{
    public function __construct(
        public readonly int $originalSize,
        public readonly int $minifiedSize,
        public readonly float $executionTimeMs
    ) {}

    public function getSpaceSaved(): int
    {
        return $this->originalSize - $this->minifiedSize;
    }

    public function getCompressionRatio(): float
    {
        if ($this->originalSize === 0) {
            return 0;
        }
        return (($this->originalSize - $this->minifiedSize) / $this->originalSize) * 100;
    }

    public function display(string $title = "Minification Results"): void
    {
        $width = 52;

        echo "\n┌" . str_repeat("─", $width - 2) . "┐\n";
        echo "│ " . str_pad($title, $width - 4) . " │\n";
        echo "├" . str_repeat("─", $width - 2) . "┤\n";

        $this->displayRow("Original size", $this->formatBytes($this->originalSize), $width);
        $this->displayRow("Minified size", $this->formatBytes($this->minifiedSize), $width);
        $this->displayRow("Space saved", $this->formatBytes($this->getSpaceSaved()), $width);
        $this->displayRow("Compression", number_format($this->getCompressionRatio(), 1) . "%", $width);
        $this->displayRow("Time taken", number_format($this->executionTimeMs, 2) . "ms", $width);

        echo "└" . str_repeat("─", $width - 2) . "┘\n\n";
    }

    private function displayRow(string $label, string $value, int $width): void
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
}