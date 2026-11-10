<?php

declare(strict_types=1);

namespace RichEditorAuy\Process;

use RichEditorAuy\Exception\TransportError;

/**
 * Injected port that executes the validator CLI.
 *
 * The rest of the package depends only on this interface, so consumers can run
 * validation in a worker, a container or a fake without the package assuming
 * anything about the host.
 */
interface ProcessRunner
{
    /**
     * Runs a command with the given standard input.
     *
     * @param non-empty-list<string> $command argv without a shell
     *
     * @throws TransportError when the process cannot be executed or read
     */
    public function run(array $command, string $stdin): ProcessResult;
}
