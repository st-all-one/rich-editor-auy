<?php

declare(strict_types=1);

namespace RichEditorAuy;

use RichEditorAuy\Exception\IncompatibleEnvelope;
use RichEditorAuy\Exception\InvalidDocument;

/**
 * Canonical document bytes plus their validated envelope.
 *
 * The package never rewrites the document: {@see Document::bytes()} returns
 * exactly the string that was parsed. {@see Document::json()} exists for
 * inspection only and must not be re-serialized for storage.
 */
final class Document
{
    /**
     * @param array<array-key, mixed> $json
     */
    private function __construct(
        private readonly string $bytes,
        private readonly array $json,
        private readonly Envelope $envelope,
    ) {}

    /**
     * Parses and validates the bytes of a canonical envelope.
     *
     * Associative decoding cannot distinguish `{}` from `[]`; both are rejected
     * where an object is required.
     *
     * @throws InvalidDocument when the bytes are not a JSON object with a non-empty doc
     * @throws IncompatibleEnvelope when the envelope metadata is not readable
     */
    public static function fromBytes(string $bytes): self
    {
        try {
            $decoded = json_decode($bytes, true, 512, JSON_THROW_ON_ERROR);
        } catch (\JsonException $exception) {
            throw new InvalidDocument('document must be valid JSON: ' . $exception->getMessage(), 0, $exception);
        }

        if (!is_array($decoded) || array_is_list($decoded)) {
            throw new InvalidDocument('document must be a JSON object');
        }

        $envelope = Envelope::fromArray($decoded);

        $doc = $decoded['doc'] ?? null;
        if (!is_array($doc) || $doc === [] || array_is_list($doc)) {
            throw new InvalidDocument('envelope must contain a non-empty doc object');
        }

        return new self($bytes, $decoded, $envelope);
    }

    /**
     * Returns the parsed bytes unchanged, including whitespace and escapes.
     */
    public function bytes(): string
    {
        return $this->bytes;
    }

    /**
     * Returns the decoded envelope for inspection only.
     *
     * @return array<array-key, mixed>
     */
    public function json(): array
    {
        return $this->json;
    }

    /**
     * Returns the validated envelope metadata.
     */
    public function envelope(): Envelope
    {
        return $this->envelope;
    }
}
