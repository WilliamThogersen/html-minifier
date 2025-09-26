<?php declare(strict_types=1);

namespace Wexowgt\Minifier;

use FFI;

class HTMLMinifier
{
    private FFI $ffi;
    private static ?HTMLMinifier $instance = null;

    private function __construct(string $libraryPath)
    {
        $this->ffi = FFI::cdef(
            "char* minify_html_string(const char* html_ptr);
             void free_string(char* ptr);",
            $libraryPath
        );
    }

    public static function getInstance(?string $libraryPath = null): self
    {
        if (self::$instance === null) {
            if ($libraryPath === null) {
                $libraryPath = self::detectLibraryPath();
            }
            self::$instance = 
            new self($libraryPath);
        }
        return self::$instance;
    }

    private static function detectLibraryPath(): string
    {
        $baseDir = __DIR__;
        $baseName = 'libhtml_minifier_ffi';

        $extensions = [];
        if (PHP_OS_FAMILY === 'Darwin') {
            $extensions = ['dylib', 'so']; // 
        } elseif (PHP_OS_FAMILY === 'Linux') {
            $extensions = ['so', 'dylib']; // 
        } else {
            $extensions = ['dll', 'so', 'dylib'];
        }

        foreach ($extensions as $ext) {
            $path = "{$baseDir}/{$baseName}.{$ext}";
            if (file_exists($path)) {
                return $path;
            }
        }

        throw new \RuntimeException(
            "Could not find HTML minifier library. Searched for: " .
            implode(', ', array_map(fn($ext) => "{$baseName}.{$ext}", $extensions))
        );
    }

    public function minify(string $html): string
    {
        $minifiedPtr = $this->ffi->minify_html_string($html);

        if ($minifiedPtr === null) {
            return $html;
        }

        $minified = FFI::string($minifiedPtr);

        $this->ffi->free_string($minifiedPtr);

        return $minified;
    }
}
