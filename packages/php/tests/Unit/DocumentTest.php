<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests\Unit;

use PHPUnit\Framework\Attributes\CoversClass;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;
use RichEditorAuy\Document;
use RichEditorAuy\Envelope;
use RichEditorAuy\Exception\IncompatibleEnvelope;
use RichEditorAuy\Exception\InvalidDocument;
use RichEditorAuy\Tests\Support\EnvelopeJson;

/**
 * One test method per case for document envelope parsing and byte preservation.
 *
 * @mago-expect lint:too-many-methods
 */
#[CoversClass(Document::class)]
#[CoversClass(Envelope::class)]
final class DocumentTest extends TestCase
{
    public function testPreservesBytesExactly(): void
    {
        $bytes =
            '{ "format" : "rich-editor-auy", "version": 2, "schema_fingerprint": "'
            . EnvelopeJson::fingerprint()
            . '", "doc": { "type": "doc", "content": [] } }';

        $document = Document::fromBytes($bytes);

        $this->assertSame($bytes, $document->bytes());
        $this->assertSame(['type' => 'doc', 'content' => []], $document->json()['doc']);
        $this->assertSame(Envelope::VERSION, $document->envelope()->version);
    }

    public function testAcceptsLegacyVersionOneEnvelope(): void
    {
        $bytes = EnvelopeJson::canonical(version: Envelope::LEGACY_VERSION);

        $document = Document::fromBytes($bytes);

        $this->assertSame(Envelope::LEGACY_VERSION, $document->envelope()->version);
        $this->assertSame($bytes, $document->bytes());
    }

    public function testAcceptsVersionOneAndTwo(): void
    {
        foreach ([Envelope::LEGACY_VERSION, Envelope::VERSION] as $version) {
            $this->assertSame(
                $version,
                Document::fromBytes(EnvelopeJson::canonical(version: $version))->envelope()->version,
            );
        }
    }

    public function testRejectsMalformedJson(): void
    {
        $this->expectException(InvalidDocument::class);
        $this->expectExceptionMessageMatches('/\Adocument must be valid JSON: /');

        Document::fromBytes('{');
    }

    public function testRejectsListTopLevel(): void
    {
        $this->expectException(InvalidDocument::class);
        $this->expectExceptionMessage('document must be a JSON object');

        Document::fromBytes('[]');
    }

    public function testRejectsScalarTopLevel(): void
    {
        $this->expectException(InvalidDocument::class);
        $this->expectExceptionMessage('document must be a JSON object');

        Document::fromBytes('1');
    }

    /**
     * @param string $doc JSON fragment used as the doc value
     */
    #[DataProvider('missingDocProvider')]
    public function testRejectsMissingDoc(string $doc): void
    {
        $this->expectException(InvalidDocument::class);
        $this->expectExceptionMessage('envelope must contain a non-empty doc object');

        Document::fromBytes($doc);
    }

    /**
     * @return iterable<string, array{string}>
     */
    public static function missingDocProvider(): iterable
    {
        yield 'absent' => [
            '{"format":"rich-editor-auy","version":2,"schema_fingerprint":"' . EnvelopeJson::fingerprint() . '"}',
        ];
        yield 'empty object' => [EnvelopeJson::withDoc('{}')];
        yield 'empty list' => [EnvelopeJson::withDoc('[]')];
        yield 'null' => [EnvelopeJson::withDoc('null')];
        yield 'string' => [EnvelopeJson::withDoc('"doc"')];
    }

    public function testAcceptsDocIdOfMaximumLength(): void
    {
        $docId = str_repeat('x', 128);

        $document = Document::fromBytes(EnvelopeJson::canonical($docId));

        $this->assertSame($docId, $document->envelope()->docId);
    }

    #[DataProvider('invalidDocIdProvider')]
    public function testRejectsInvalidDocId(string $docIdJson): void
    {
        $bytes = sprintf(
            '{"format":"rich-editor-auy","version":2,"schema_fingerprint":"%s","doc_id":%s,"doc":{"type":"doc"}}',
            EnvelopeJson::fingerprint(),
            $docIdJson,
        );

        $this->expectException(IncompatibleEnvelope::class);
        $this->expectExceptionMessage('doc_id must be 1-128 graphic ASCII characters');

        Document::fromBytes($bytes);
    }

    /**
     * @return iterable<string, array{string}>
     */
    public static function invalidDocIdProvider(): iterable
    {
        yield 'too long' => ['"' . str_repeat('x', 129) . '"'];
        yield 'space' => ['"bad id"'];
        yield 'empty' => ['""'];
        yield 'trailing newline escape' => ['"post-1\n"'];
        yield 'leading tab escape' => ['"\tpost-1"'];
        yield 'non-string' => ['7'];
    }

    public function testAcceptsMissingDocId(): void
    {
        $document = Document::fromBytes(EnvelopeJson::canonical());

        $this->assertNull($document->envelope()->docId);
    }

    public function testKeepsUnknownEnvelopeFields(): void
    {
        $bytes =
            '{"format":"rich-editor-auy","version":2,"schema_fingerprint":"'
            . EnvelopeJson::fingerprint()
            . '","producer":"auy-test","doc":{"type":"doc"}}';

        $document = Document::fromBytes($bytes);

        $this->assertSame('auy-test', $document->json()['producer']);
        $this->assertSame($bytes, $document->bytes());
    }

    public function testPropagatesEnvelopeFailureBeforeDocCheck(): void
    {
        $this->expectException(IncompatibleEnvelope::class);
        $this->expectExceptionMessage('envelope format must be "rich-editor-auy"');

        Document::fromBytes(
            '{"format":"other","version":2,"schema_fingerprint":"' . EnvelopeJson::fingerprint() . '"}',
        );
    }
}
