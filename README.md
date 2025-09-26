# HTML Minifier

A high-performance HTML minifier built with Rust and exposed to PHP via FFI. Provides fast, UTF-8 safe HTML minification with intelligent optimization of HTML, CSS, and JavaScript content.

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Usage](#usage)
- [Server Configuration](#server-configuration)
- [Performance](#performance)
- [Architecture](#architecture)
- [Troubleshooting](#troubleshooting)

## Features

- **High Performance**: Rust-based core achieving 50-60% size reduction
- **UTF-8 Safe**: Proper Unicode character handling
- **Cross-Platform**: Automatic detection of shared libraries (.dylib, .so, .dll)
- **Tokenized Processing**: Uses a custom tokenizer for accurate HTML parsing and minification
- **Comprehensive**: Minifies HTML, CSS, and JavaScript with intelligent context-aware processing
- **Production Ready**: Memory-safe with proper error handling

## Requirements

- **PHP 8.0+** with FFI extension enabled
- **Rust 1.70+** (for building from source)
- **Operating System**: Linux, macOS, or Windows

## Installation

### 1. Install Dependencies

#### PHP FFI Extension
```bash
# Ubuntu/Debian
sudo apt-get install php-ffi


# macOS (with Homebrew)
brew install php
# FFI is included by default in modern PHP versions
```

#### Rust (for building)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Build the Project

```bash
# Clone or extract the project
cd /path/to/minifier

# Build the Rust library
bash build.sh

# For cross-platform builds
bash cross-compile.sh
```

### 3. Composer Setup

Create or update your `composer.json`:

```json
{
    "autoload": {
        "psr-4": {
            "Wexowgt\\Minifier\\": "src/"
        }
    },
    "require": {
        "php": ">=8.0",
        "ext-ffi": "*"
    }
}
```

Install dependencies:
```bash
composer install
```

## Usage

### Basic Usage

```php
<?php
require_once 'vendor/autoload.php';

use Wexowgt\Minifier\HTMLMinifier;

// Initialize with automatic library detection
$minifier = HTMLMinifier::getInstance();

// Minify HTML content
$html = '<div class="container">  <p>Hello World!</p>  </div>';
$minified = $minifier->minify($html);

echo $minified; // <div class=container><p>Hello World!</p></div>
```

### File Processing

```php
<?php
$minifier = HTMLMinifier::getInstance();
$html = file_get_contents('input.html');
$minified = $minifier->minify($html);
file_put_contents('output.html', $minified);
```

### Output Buffering

```php
<?php
function minifyOutput($buffer) {
    static $minifier = null;
    if ($minifier === null) {
        $minifier = HTMLMinifier::getInstance();
    }
    return $minifier->minify($buffer);
}

ob_start('minifyOutput');
?>
<!DOCTYPE html>
<html>
<head><title>Auto-minified Page</title></head>
<body><h1>Welcome</h1></body>
</html>
<?php
ob_end_flush();
```

## Performance

Typical performance characteristics:
- **Speed**: 1-5ms for average web pages
- **Compression**: 50-60% size reduction
- **Memory**: Minimal overhead, scales linearly with input size

## Architecture

### Tokenized HTML Processing

This minifier uses a **tokenized approach** rather than regex-based parsing, providing more accurate and reliable HTML minification.

#### How Tokenization Works

The tokenizer breaks HTML into discrete tokens by parsing the input character by character:

```rust
enum Token<'a> {
    TextNode(&'a str),      // Plain text content
    TagOpenStart(&'a str),  // <div, <p, etc.
    Attribute(&'a str),     // class="foo", id="bar"
    TagOpenEnd,             // >
    TagSelfClose,           // />
    TagClose(&'a str),      // </div>, </p>
    Comment(&'a str),       // <!-- comment -->
    Doctype(&'a str),       // <!DOCTYPE html>
    Cdata(&'a str),         // <![CDATA[...]]>
}
```

#### Token Building Process

1. **Character-by-Character Parsing**: The tokenizer examines each byte of the input HTML
2. **Context Awareness**: Tracks whether it's inside a tag, parsing attributes, or processing text
3. **State Machine**: Uses flags like `in_tag`, `in_pre_tag`, `in_script_tag` to handle different contexts
4. **Intelligent Boundaries**: Recognizes tag boundaries, attribute separators, and content types

#### Processing Pipeline

```
HTML Input → Tokenizer → Token Stream → Context-Aware Processing → Minified Output
```

Each token type receives specialized treatment:
- **TextNode**: Whitespace collapsed (except in `<pre>`, `<code>`, `<textarea>`)
- **TagOpenStart**: Tag names converted to lowercase
- **Attribute**: Boolean attributes simplified, default values removed, quotes optimized
- **Script/Style Content**: Dedicated JavaScript and CSS minifiers applied
- **Comments**: Removed entirely (except conditional comments)

This approach ensures accurate parsing of complex HTML structures while maintaining semantic correctness during minification.

## Server Configuration

### Nginx
```nginx
# Enable FFI in PHP-FPM pool
location ~ \.php$ {
    fastcgi_pass unix:/var/run/php/php8.2-fpm.sock;
    fastcgi_param PHP_VALUE "ffi.enable=1";
    include fastcgi_params;
}
```

### Apache
```apache
# Enable FFI
<Directory /var/www/html>
    php_admin_value ffi.enable 1
    php_admin_value memory_limit 256M
</Directory>

# Protect library files
<Files "*.so *.dylib *.dll">
    Require all denied
</Files>
```

### PHP Configuration
```ini
; Enable FFI (required)
ffi.enable = 1

; Performance optimizations
opcache.enable = 1
opcache.memory_consumption = 256
```


## Troubleshooting

**FFI Not Available**: Install and enable the PHP FFI extension.

**Library Not Found**: Run `bash build.sh` to compile the Rust library.

**Permission Denied**: Check file permissions and SELinux settings.

**Memory Issues**: Increase PHP memory limit or process content in chunks.