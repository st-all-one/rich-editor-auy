<?php

declare(strict_types=1);

namespace RichEditorAuy\Process;

/**
 * Raw result of running an external process.
 */
final class ProcessResult
{
    public function __construct(
        public readonly int $exitCode,
        public readonly string $stdout,
        public readonly string $stderr,
    ) {}
}
