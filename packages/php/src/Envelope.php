<?php

declare(strict_types=1);

namespace RichEditorAuy;

final class Envelope
{
    /**
     * @param array<array-key, mixed> $doc Canonical document tree.
     */
    public function __construct(
        public readonly string $format,
        public readonly int $version,
        public readonly string $schemaFingerprint,
        public readonly array $doc,
    ) {
    }

    public function validate(): void
    {
        if ($this->format !== 'rich-editor-auy' || ($this->version !== 1 && $this->version !== 2)) {
            throw new \InvalidArgumentException('incompatible envelope format');
        }
        if (!preg_match('/^[0-9a-f]{64}$/', $this->schemaFingerprint)) {
            throw new \InvalidArgumentException('schema_fingerprint must be 64 lowercase hex characters');
        }
    }

    public function requireCompatible(string $schemaFingerprint): void
    {
        $this->validate();
        if ($this->version === 1) {
            return;
        }
        if ($this->schemaFingerprint !== $schemaFingerprint) {
            throw new \InvalidArgumentException('schema fingerprint mismatch');
        }
    }
}
