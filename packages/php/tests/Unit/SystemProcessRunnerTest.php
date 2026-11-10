<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests\Unit;

use PHPUnit\Framework\Attributes\CoversClass;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;
use RichEditorAuy\Exception\TransportError;
use RichEditorAuy\Process\ProcessResult;
use RichEditorAuy\Process\SystemProcessRunner;

/**
 * One test method per case for pipe, stdin and deadline behaviour.
 *
 * @mago-expect lint:too-many-methods
 */
#[CoversClass(SystemProcessRunner::class)]
#[CoversClass(ProcessResult::class)]
final class SystemProcessRunnerTest extends TestCase
{
    public function testCapturesExitCodeAndBothStreams(): void
    {
        $result = $this->runner()->run(
            [PHP_BINARY, '-r', 'fwrite(STDOUT, "out"); fwrite(STDERR, "err"); exit(3);'],
            '',
        );

        $this->assertSame(3, $result->exitCode);
        $this->assertSame('out', $result->stdout);
        $this->assertSame('err', $result->stderr);
    }

    public function testEchoesLargeStdinWithoutDeadlock(): void
    {
        $payload = str_repeat('payload-', 131_072);

        $result = $this->runner()->run([PHP_BINARY, '-r', 'fwrite(STDOUT, stream_get_contents(STDIN));'], $payload);

        $this->assertSame(0, $result->exitCode);
        $this->assertSame(strlen($payload), strlen($result->stdout));
        $this->assertSame($payload, $result->stdout);
    }

    public function testDrainsBothStreamsConcurrently(): void
    {
        $result = $this->runner()->run(
            [PHP_BINARY, '-r', 'fwrite(STDOUT, str_repeat("o", 204_800)); fwrite(STDERR, str_repeat("e", 204_800));'],
            '',
        );

        $this->assertSame(0, $result->exitCode);
        $this->assertSame(str_repeat('o', 204_800), $result->stdout);
        $this->assertSame(str_repeat('e', 204_800), $result->stderr);
    }

    public function testKeepsReadingWhenOnePipeClosesFirst(): void
    {
        $result = $this->runner()->run(
            [PHP_BINARY, '-r', 'fclose(STDOUT); usleep(100000); fwrite(STDERR, "late"); exit(0);'],
            '',
        );

        $this->assertSame(0, $result->exitCode);
        $this->assertSame('', $result->stdout);
        $this->assertSame('late', $result->stderr);
    }

    public function testTreatsClosedStdinAsEndOfInput(): void
    {
        $result = $this->runner()->run([PHP_BINARY, '-r', 'exit(0);'], str_repeat('x', 1_048_576));

        $this->assertSame(0, $result->exitCode);
        $this->assertSame('', $result->stdout);
        $this->assertSame('', $result->stderr);
    }

    public function testTimesOutAndKillsTheChild(): void
    {
        $this->expectException(TransportError::class);
        $this->expectExceptionMessage('timed out');

        (new SystemProcessRunner(0.05))->run([PHP_BINARY, '-r', 'usleep(2000000);'], '');
    }

    public function testPreservesBinaryPayloads(): void
    {
        $payload = "\x00\xff\xfe\x01binary";

        $result = $this->runner()->run([PHP_BINARY, '-r', 'fwrite(STDOUT, stream_get_contents(STDIN));'], $payload);

        $this->assertSame($payload, $result->stdout);
    }

    public function testFailsClosedWhenTheBinaryIsMissing(): void
    {
        try {
            $result = $this->runner()->run(['/nonexistent/auy-binary', 'validate'], '');
        } catch (TransportError $error) {
            $this->assertStringContainsString('could not start validator process', $error->getMessage());

            return;
        }

        $this->assertNotSame(0, $result->exitCode);
    }

    public function testRejectsNulByteInCommand(): void
    {
        $this->expectException(TransportError::class);
        $this->expectExceptionMessage('could not start validator process');

        $this->runner()->run(["/bin/true\0", 'validate'], '');
    }

    #[DataProvider('invalidTimeoutProvider')]
    public function testRejectsInvalidTimeout(float $timeout): void
    {
        $this->expectException(\InvalidArgumentException::class);
        $this->expectExceptionMessage('process timeout must be greater than zero');

        new SystemProcessRunner($timeout);
    }

    /**
     * @return iterable<string, array{float}>
     */
    public static function invalidTimeoutProvider(): iterable
    {
        yield 'zero' => [0.0];
        yield 'negative' => [-1.0];
        yield 'infinite' => [INF];
        yield 'not a number' => [NAN];
    }

    private function runner(): SystemProcessRunner
    {
        return new SystemProcessRunner(30.0);
    }
}
