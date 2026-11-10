<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests\Support;

/**
 * Builds envelope JSON byte for byte, so tests pin the exact input shape
 * instead of relying on json_encode key order or escaping.
 */
final class EnvelopeJson
{
    private function __construct() {}

    public static function fingerprint(): string
    {
        return str_repeat('a', 64);
    }

    public static function canonical(?string $docId = null, int $version = 2): string
    {
        $docIdField = $docId === null ? '' : sprintf(',"doc_id":"%s"', $docId);

        return sprintf(
            '{"format":"rich-editor-auy","version":%d,"schema_fingerprint":"%s"%s,"doc":{"type":"doc","content":[]}}',
            $version,
            self::fingerprint(),
            $docIdField,
        );
    }

    public static function withDoc(string $doc, int $version = 2): string
    {
        return sprintf(
            '{"format":"rich-editor-auy","version":%d,"schema_fingerprint":"%s","doc":%s}',
            $version,
            self::fingerprint(),
            $doc,
        );
    }
}
