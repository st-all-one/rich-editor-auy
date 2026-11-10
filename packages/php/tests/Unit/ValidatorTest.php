<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests\Unit;

use PHPUnit\Framework\Attributes\CoversClass;
use PHPUnit\Framework\TestCase;
use RichEditorAuy\Process\ProcessResult;
use RichEditorAuy\Tests\Support\StubProcessRunner;
use RichEditorAuy\Validator;

#[CoversClass(Validator::class)]
final class ValidatorTest extends TestCase
{
    public function testRunsValidateSubcommandWithExactStdin(): void
    {
        $runner = new StubProcessRunner(new ProcessResult(0, "ok\n", ''));
        $bytes = "{\"doc\":\"\xff\x00\"}";

        $report = (new Validator('/usr/local/bin/auy', $runner))->validate($bytes);

        $this->assertTrue($report->ok());
        $this->assertSame("ok\n", $report->stdout);
        $this->assertSame([['command' => ['/usr/local/bin/auy', 'validate'], 'stdin' => $bytes]], $runner->calls);
    }

    public function testMapsStructuredFailureFromRunner(): void
    {
        $runner = new StubProcessRunner(new ProcessResult(1, '', "I1 at 3 (doc.blocks[0])\n"));

        $report = (new Validator('/usr/local/bin/auy', $runner))->validate('{}');

        $this->assertFalse($report->ok());
        $this->assertCount(1, $report->issues);
        $this->assertSame('I1', $report->issues[0]->code);
        $this->assertSame(3, $report->issues[0]->offset);
        $this->assertSame('doc.blocks[0]', $report->issues[0]->path);
    }

    public function testDefaultsToSystemRunner(): void
    {
        // PHP_BINARY does not understand "validate", so the run must fail closed
        // instead of throwing: the default port executed a real process.
        $report = (new Validator(PHP_BINARY))->validate('{"doc":{}}');

        $this->assertFalse($report->ok());
    }

    public function testRejectsEmptyBinary(): void
    {
        $this->expectException(\InvalidArgumentException::class);
        $this->expectExceptionMessage('validator binary must not be empty');

        new Validator('');
    }

    public function testRejectsBinaryWithNulByte(): void
    {
        $this->expectException(\InvalidArgumentException::class);
        $this->expectExceptionMessage('validator binary must not contain NUL bytes');

        new Validator("/usr/local/bin/\0auy");
    }
}
