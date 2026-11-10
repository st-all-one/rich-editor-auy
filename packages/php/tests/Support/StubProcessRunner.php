<?php

declare(strict_types=1);

namespace RichEditorAuy\Tests\Support;

use RichEditorAuy\Process\ProcessResult;
use RichEditorAuy\Process\ProcessRunner;

/**
 * Records every call and replays one canned process result.
 */
final class StubProcessRunner implements ProcessRunner
{
    /**
     * @var list<array{command: non-empty-list<string>, stdin: string}>
     */
    public array $calls = [];

    public function __construct(
        private readonly ProcessResult $result,
    ) {}

    public function run(array $command, string $stdin): ProcessResult
    {
        $this->calls[] = ['command' => $command, 'stdin' => $stdin];

        return $this->result;
    }
}
