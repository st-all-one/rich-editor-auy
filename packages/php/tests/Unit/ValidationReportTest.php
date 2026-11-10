<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests\Unit;

use PHPUnit\Framework\Attributes\CoversClass;
use PHPUnit\Framework\TestCase;
use RichEditorAuy\ValidationIssue;
use RichEditorAuy\ValidationReport;

#[CoversClass(ValidationReport::class)]
#[CoversClass(ValidationIssue::class)]
final class ValidationReportTest extends TestCase
{
    public function testAcceptsSuccessfulValidation(): void
    {
        $report = ValidationReport::fromProcess(0, "ok\n", '');

        $this->assertTrue($report->ok());
        $this->assertSame(0, $report->exitCode);
        $this->assertSame("ok\n", $report->stdout);
        $this->assertSame('', $report->stderr);
        $this->assertSame([], $report->issues);
        $this->assertSame([], $report->diagnostics);
        $this->assertSame([], $report->unrecognized);
    }

    public function testParsesStructuredIssues(): void
    {
        $report = ValidationReport::fromProcess(1, '', "I1 at 0 ()\nI23 at 42 (3/0)\nA003 at 7 (2/1)\n");

        $this->assertFalse($report->ok());
        $this->assertCount(3, $report->issues);

        $this->assertSame('I1', $report->issues[0]->code);
        $this->assertSame(0, $report->issues[0]->offset);
        $this->assertSame('', $report->issues[0]->path);

        $this->assertSame('I23', $report->issues[1]->code);
        $this->assertSame(42, $report->issues[1]->offset);
        $this->assertSame('3/0', $report->issues[1]->path);

        $this->assertSame('A003', $report->issues[2]->code);
        $this->assertSame(7, $report->issues[2]->offset);
        $this->assertSame('2/1', $report->issues[2]->path);

        $this->assertSame([], $report->unrecognized);
    }

    public function testParsesCliErrorWithOffsetAsIssue(): void
    {
        $report = ValidationReport::fromProcess(1, '', "error: missing_format at offset 0\n");

        $this->assertFalse($report->ok());
        $this->assertCount(1, $report->issues);
        $this->assertSame('missing_format', $report->issues[0]->code);
        $this->assertSame(0, $report->issues[0]->offset);
        $this->assertSame('', $report->issues[0]->path);
        $this->assertSame([], $report->diagnostics);
    }

    public function testKeepsFreeFormCliDiagnostics(): void
    {
        $stderr = "error: content mismatch at schema_fingerprint\nerror: could not read stdin\nusage: auy validate\n";
        $report = ValidationReport::fromProcess(2, '', $stderr);

        $this->assertFalse($report->ok());
        $this->assertSame([], $report->issues);
        $this->assertSame(
            [
                'error: content mismatch at schema_fingerprint',
                'error: could not read stdin',
                'usage: auy validate',
            ],
            $report->diagnostics,
        );
        $this->assertSame([], $report->unrecognized);
        $this->assertSame($stderr, $report->stderr);
    }

    public function testKeepsUnknownLinesAsUnrecognized(): void
    {
        $report = ValidationReport::fromProcess(1, '', "I1 at 3 (doc)\nsomething unexpected\n");

        $this->assertCount(1, $report->issues);
        $this->assertSame([], $report->diagnostics);
        $this->assertSame(['something unexpected'], $report->unrecognized);
    }

    public function testNormalizesCrlfAndIgnoresEmptyLines(): void
    {
        $stderr = "I1 at 3 (doc)\r\n\r\nA001 at 9 (doc.blocks)\r\n";
        $report = ValidationReport::fromProcess(1, '', $stderr);

        $this->assertCount(2, $report->issues);
        $this->assertSame('A001', $report->issues[1]->code);
        $this->assertSame(9, $report->issues[1]->offset);
        $this->assertSame($stderr, $report->stderr);
    }

    public function testNonZeroExitWithoutStderrIsNotOk(): void
    {
        $report = ValidationReport::fromProcess(1, '', '');

        $this->assertFalse($report->ok());
        $this->assertSame([], $report->issues);
    }
}
