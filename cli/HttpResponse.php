<?php

class HttpResponse
{
    public function __construct(
        public readonly mixed $content,
        public readonly int $httpCode,
        public readonly string $contentType,
        public readonly string $error,
        public readonly float $fetchTimeMs
    ) {}

    public function isSuccess(): bool
    {
        return $this->content !== false && empty($this->error) && $this->httpCode === 200;
    }

    public function isHtml(): bool
    {
        return strpos(strtolower($this->contentType), 'text/html') !== false;
    }

    public function getContent(): string
    {
        return (string) $this->content;
    }
}