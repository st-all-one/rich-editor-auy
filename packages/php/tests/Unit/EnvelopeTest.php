<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests\Unit;

use PHPUnit\Framework\Attributes\CoversClass;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;
use RichEditorAuy\Envelope;
use RichEditorAuy\Exception\IncompatibleEnvelope;
use RichEditorAuy\Tests\Support\EnvelopeJson;

/**
 * One test method per case for envelope field contracts.
 *
 * @mago-expect lint:too-many-methods
 */
#[CoversClass(Envelope::class)]
final class EnvelopeTest extends TestCase
{
    public function testReadsCanonicalFields(): void
    {
        $envelope = Envelope::fromArray([
            'format' => Envelope::FORMAT,
            'version' => Envelope::VERSION,
            'schema_fingerprint' => EnvelopeJson::fingerprint(),
            'doc_id' => 'post-1',
            'doc' => ['type' => 'doc'],
            'producer' => 'unknown-field',
        ]);

        $this->assertSame(Envelope::FORMAT, $envelope->format);
        $this->assertSame(Envelope::VERSION, $envelope->version);
        $this->assertSame(EnvelopeJson::fingerprint(), $envelope->schemaFingerprint);
        $this->assertSame('post-1', $envelope->docId);
    }

    public function testAcceptsMissingDocId(): void
    {
        $envelope = Envelope::fromArray([
            'format' => Envelope::FORMAT,
            'version' => Envelope::VERSION,
            'schema_fingerprint' => EnvelopeJson::fingerprint(),
        ]);

        $this->assertNull($envelope->docId);
    }

    public function testRejectsMissingFormat(): void
    {
        $this->expectException(IncompatibleEnvelope::class);
        $this->expectExceptionMessage('envelope format must be "rich-editor-auy"');

        Envelope::fromArray(['version' => 2, 'schema_fingerprint' => EnvelopeJson::fingerprint()]);
    }

    #[DataProvider('invalidFormatProvider')]
    public function testRejectsInvalidFormat(mixed $format): void
    {
        $this->expectException(IncompatibleEnvelope::class);
        $this->expectExceptionMessage('envelope format must be "rich-editor-auy"');

        Envelope::fromArray([
            'format' => $format,
            'version' => 2,
            'schema_fingerprint' => EnvelopeJson::fingerprint(),
        ]);
    }

    /**
     * @return iterable<string, array{mixed}>
     */
    public static function invalidFormatProvider(): iterable
    {
        yield 'other string' => ['rich-editor'];
        yield 'uppercase' => ['RICH-EDITOR-AUY'];
        yield 'integer' => [7];
        yield 'null' => [null];
    }

    #[DataProvider('invalidVersionProvider')]
    public function testRejectsInvalidVersion(mixed $version): void
    {
        $this->expectException(IncompatibleEnvelope::class);
        $this->expectExceptionMessage('envelope version must be 1 or 2');

        Envelope::fromArray([
            'format' => Envelope::FORMAT,
            'version' => $version,
            'schema_fingerprint' => EnvelopeJson::fingerprint(),
        ]);
    }

    /**
     * @return iterable<string, array{mixed}>
     */
    public static function invalidVersionProvider(): iterable
    {
        yield 'zero' => [0];
        yield 'three' => [3];
        yield 'string' => ['2'];
        yield 'float' => [2.0];
        yield 'null' => [null];
        yield 'list' => [[2]];
    }

    public function testAcceptsBothVersions(): void
    {
        foreach ([Envelope::LEGACY_VERSION, Envelope::VERSION] as $version) {
            $envelope = Envelope::fromArray([
                'format' => Envelope::FORMAT,
                'version' => $version,
                'schema_fingerprint' => EnvelopeJson::fingerprint(),
            ]);

            $this->assertSame($version, $envelope->version);
        }
    }

    public function testRejectsMissingFingerprint(): void
    {
        $this->expectException(IncompatibleEnvelope::class);
        $this->expectExceptionMessage('schema_fingerprint must be 64 lowercase hex characters');

        Envelope::fromArray(['format' => Envelope::FORMAT, 'version' => Envelope::VERSION]);
    }

    #[DataProvider('invalidFingerprintProvider')]
    public function testRejectsInvalidFingerprint(mixed $fingerprint): void
    {
        $this->expectException(IncompatibleEnvelope::class);
        $this->expectExceptionMessage('schema_fingerprint must be 64 lowercase hex characters');

        Envelope::fromArray([
            'format' => Envelope::FORMAT,
            'version' => Envelope::VERSION,
            'schema_fingerprint' => $fingerprint,
        ]);
    }

    /**
     * @return iterable<string, array{mixed}>
     */
    public static function invalidFingerprintProvider(): iterable
    {
        yield 'missing enough characters' => [str_repeat('a', 63)];
        yield 'too long' => [str_repeat('a', 65)];
        yield 'uppercase' => [str_repeat('A', 64)];
        yield 'non-hex' => [str_repeat('z', 64)];
        yield 'trailing newline' => [str_repeat('a', 64) . "\n"];
        yield 'non-string' => [64];
    }

    public function testRequireCompatibleAcceptsMatchingFingerprint(): void
    {
        $envelope = Envelope::fromArray([
            'format' => Envelope::FORMAT,
            'version' => Envelope::VERSION,
            'schema_fingerprint' => EnvelopeJson::fingerprint(),
        ]);

        $envelope->requireCompatible(EnvelopeJson::fingerprint());

        $this->assertSame(Envelope::VERSION, $envelope->version);
    }

    public function testRequireCompatibleRejectsDifferentFingerprint(): void
    {
        $envelope = Envelope::fromArray([
            'format' => Envelope::FORMAT,
            'version' => Envelope::VERSION,
            'schema_fingerprint' => EnvelopeJson::fingerprint(),
        ]);

        $this->expectException(IncompatibleEnvelope::class);
        $this->expectExceptionMessage('schema fingerprint mismatch');

        $envelope->requireCompatible(str_repeat('b', 64));
    }

    public function testRequireCompatibleIgnoresLegacyVersion(): void
    {
        $envelope = Envelope::fromArray([
            'format' => Envelope::FORMAT,
            'version' => Envelope::LEGACY_VERSION,
            'schema_fingerprint' => EnvelopeJson::fingerprint(),
        ]);

        $envelope->requireCompatible(str_repeat('b', 64));

        $this->assertSame(Envelope::LEGACY_VERSION, $envelope->version);
    }
}
