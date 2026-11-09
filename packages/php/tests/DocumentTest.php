<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests;

use PHPUnit\Framework\TestCase;
use RichEditorAuy\Document;

final class DocumentTest extends TestCase
{
    public function testPreservesBytes(): void
    {
        $bytes = '{"format":"rich-editor-auy","version":1,"schema_fingerprint":"' . str_repeat('a', 64) . '","doc":{"type":"doc","content":[]}}';
        $document = Document::fromBytes($bytes);
        self::assertSame($bytes, $document->bytes());
    }

    public function testRejectsIncompatibleEnvelope(): void
    {
        $bytes = '{"format":"other","version":99,"schema_fingerprint":"bad","doc":{}}';
        $this->expectException(\InvalidArgumentException::class);
        Document::fromBytes($bytes);
    }

    public function testRejectsMissingDoc(): void
    {
        $bytes = '{"format":"rich-editor-auy","version":1,"schema_fingerprint":"' . str_repeat('a', 64) . '"}';
        $this->expectException(\InvalidArgumentException::class);
        Document::fromBytes($bytes);
    }

    public function testAcceptsLegacyVersionOneEnvelope(): void
    {
        $bytes = '{"format":"rich-editor-auy","version":1,"schema_fingerprint":"' . str_repeat('a', 64) . '","doc":{"type":"doc","content":[]}}';
        $document = Document::fromBytes($bytes);
        self::assertSame($bytes, $document->bytes());
    }

    public function testRejectsFingerprintMismatch(): void
    {
        $envelope = new \RichEditorAuy\Envelope(
            'rich-editor-auy',
            2,
            str_repeat('a', 64),
            [],
        );
        $this->expectException(\InvalidArgumentException::class);
        $envelope->requireCompatible(str_repeat('b', 64));
    }
}
