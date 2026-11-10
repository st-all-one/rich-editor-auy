<?php

declare(strict_types=1);

namespace RichEditorAuy;

use RichEditorAuy\Exception\IncompatibleEnvelope;

/**
 * Validated view of the canonical envelope fields.
 *
 * The document tree belongs to {@see Document}; this value object only carries
 * the metadata a consumer needs to reject bytes it cannot read. Instances are
 * created through {@see Envelope::fromArray()}, so no invalid envelope exists.
 */
final class Envelope
{
    public const FORMAT = 'rich-editor-auy';

    public const VERSION = 2;

    public const LEGACY_VERSION = 1;

    /**
     * @param 1|2 $version
     */
    private function __construct(
        public readonly string $format,
        public readonly int $version,
        public readonly string $schemaFingerprint,
        public readonly ?string $docId,
    ) {}

    /**
     * Reads the envelope fields from a decoded JSON object.
     *
     * Unknown fields are ignored so that a newer producer stays readable.
     *
     * @param array<array-key, mixed> $json
     *
     * @throws IncompatibleEnvelope when format, version, fingerprint or doc_id break the contract
     */
    public static function fromArray(array $json): self
    {
        $format = $json['format'] ?? null;
        if ($format !== self::FORMAT) {
            throw new IncompatibleEnvelope('envelope format must be "rich-editor-auy"');
        }

        $version = $json['version'] ?? null;
        if (!is_int($version) || $version !== self::LEGACY_VERSION && $version !== self::VERSION) {
            throw new IncompatibleEnvelope('envelope version must be 1 or 2');
        }

        $fingerprint = $json['schema_fingerprint'] ?? null;
        if (!is_string($fingerprint) || preg_match('/\A[0-9a-f]{64}\z/', $fingerprint) !== 1) {
            throw new IncompatibleEnvelope('schema_fingerprint must be 64 lowercase hex characters');
        }

        $docId = $json['doc_id'] ?? null;
        if ($docId !== null && (!is_string($docId) || preg_match('/\A[\x21-\x7e]{1,128}\z/', $docId) !== 1)) {
            throw new IncompatibleEnvelope('doc_id must be 1-128 graphic ASCII characters');
        }

        return new self(self::FORMAT, $version, $fingerprint, $docId);
    }

    /**
     * Fails when this envelope was produced by a different schema.
     *
     * Version 1 envelopes predate the fingerprint contract, so the comparison
     * is skipped for them.
     *
     * @throws IncompatibleEnvelope when the schema fingerprints differ
     */
    public function requireCompatible(string $schemaFingerprint): void
    {
        if ($this->version === self::LEGACY_VERSION) {
            return;
        }

        if ($this->schemaFingerprint !== $schemaFingerprint) {
            throw new IncompatibleEnvelope('schema fingerprint mismatch');
        }
    }
}
