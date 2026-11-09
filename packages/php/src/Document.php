<?php

declare(strict_types=1);

namespace RichEditorAuy;

final class Document
{
    /**
     * @param array<string, mixed> $json Decoded envelope object.
     */
    private function __construct(
        private readonly string $bytes,
        private readonly array $json,
        private readonly Envelope $envelope,
    ) {
    }

    public static function fromBytes(string $bytes): self
    {
        $decoded = json_decode($bytes, true, 512, JSON_THROW_ON_ERROR);
        if (!is_array($decoded) || array_is_list($decoded)) {
            throw new \InvalidArgumentException('document must be a JSON object');
        }
        /** @var array<string, mixed> $json */
        $json = $decoded;
        $doc = $json['doc'] ?? null;
        if (!is_array($doc)) {
            throw new \InvalidArgumentException('envelope must contain a doc object');
        }
        $envelope = new Envelope(
            is_string($json['format'] ?? null) ? $json['format'] : '',
            is_int($json['version'] ?? null) ? $json['version'] : 0,
            is_string($json['schema_fingerprint'] ?? null) ? $json['schema_fingerprint'] : '',
            $doc,
        );
        $envelope->validate();
        return new self($bytes, $json, $envelope);
    }

    public function bytes(): string
    {
        return $this->bytes;
    }

    /**
     * @return array<string, mixed> Decoded envelope object.
     */
    public function json(): array
    {
        return $this->json;
    }

    public function envelope(): Envelope
    {
        return $this->envelope;
    }
}
