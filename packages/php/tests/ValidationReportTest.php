<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests;

use PHPUnit\Framework\TestCase;
use RichEditorAuy\ValidationReport;

final class ValidationReportTest extends TestCase
{
    public function testAcceptsSuccessfulValidation(): void
    {
        $report = ValidationReport::fromProcess(0, "ok\n", '');

        self::assertTrue($report->ok());
        self::assertSame("ok\n", $report->stdout);
        self::assertSame([], $report->issues);
    }

    public function testParsesStructuredIssues(): void
    {
        $report = ValidationReport::fromProcess(
            1,
            '',
            "I1 at 4 (doc.blocks[0])\nunknown_token at 0 (3)\n",
        );

        self::assertFalse($report->ok());
        self::assertCount(2, $report->issues);
        self::assertSame('I1', $report->issues[0]->code);
        self::assertSame(4, $report->issues[0]->offset);
        self::assertSame('doc.blocks[0]', $report->issues[0]->path);
        self::assertSame('unknown_token', $report->issues[1]->code);
    }

    public function testKeepsUnstructuredStderrAsValidatorError(): void
    {
        $report = ValidationReport::fromProcess(1, '', "could not start validator\n");

        self::assertCount(1, $report->issues);
        self::assertSame('validator_error', $report->issues[0]->code);
        self::assertSame('could not start validator', $report->issues[0]->path);
    }
}
