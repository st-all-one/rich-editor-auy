<?php

declare(strict_types=1);

namespace RichEditorAuy;

final class Document
{
    private function __construct(
        private readonly string $bytes,
        private readonly array $json,
        private readonly Envelope $envelope,
    ) {
    }

    public static function fromBytes(string $bytes): self
    {
        $json = json_decode($bytes, true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($json)) {
            throw new \InvalidArgumentException('document must be a JSON object');
        }
        if (!is_array($json['doc'] ?? null)) {
            throw new \InvalidArgumentException('envelope must contain a doc object');
        }
        $envelope = new Envelope(
            is_string($json['format'] ?? null) ? $json['format'] : '',
            is_int($json['version'] ?? null) ? $json['version'] : 0,
            is_string($json['schema_fingerprint'] ?? null) ? $json['schema_fingerprint'] : '',
            is_array($json['doc'] ?? null) ? $json['doc'] : [],
        );
        $envelope->validate();
        return new self($bytes, $json, $envelope);
    }

    public function bytes(): string
    {
        return $this->bytes;
    }

    public function json(): array
    {
        return $this->json;
    }

    public function envelope(): Envelope
    {
        return $this->envelope;
    }
}
