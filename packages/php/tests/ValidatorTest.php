<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests;

use PHPUnit\Framework\TestCase;
use RichEditorAuy\Validator;

final class ValidatorTest extends TestCase
{
    public function testRunsAnExecutableAndParsesItsReport(): void
    {
        $binary = $this->script("<?php echo 'ok';");
        try {
            $report = (new Validator($binary))->validate('{"type":"doc","content":[]}');
            self::assertTrue($report->ok());
            self::assertSame('ok', $report->stdout);
            self::assertSame([], $report->issues);
        } finally {
            @unlink($binary);
        }
    }

    public function testReportsAStructuredFailure(): void
    {
        $binary = $this->script("<?php fwrite(STDERR, 'I1 at 3 (doc.blocks[0])'); exit(1);");
        try {
            $report = (new Validator($binary))->validate('{}');
            self::assertFalse($report->ok());
            self::assertCount(1, $report->issues);
            self::assertSame('I1', $report->issues[0]->code);
            self::assertSame(3, $report->issues[0]->offset);
            self::assertSame('doc.blocks[0]', $report->issues[0]->path);
        } finally {
            @unlink($binary);
        }
    }

    private function script(string $body): string
    {
        $path = tempnam(sys_get_temp_dir(), 'auy-validator-');
        if ($path === false) {
            throw new \RuntimeException('could not create validator fixture');
        }
        if (file_put_contents($path, "#!/usr/bin/env php\n{$body}\n") === false) {
            @unlink($path);
            throw new \RuntimeException('could not write validator fixture');
        }
        chmod($path, 0o755);

        return $path;
    }
}
