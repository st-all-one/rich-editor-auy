<?php

declare(strict_types=1);

namespace RichEditorAuy;

use RichEditorAuy\Exception\TransportError;
use RichEditorAuy\Process\ProcessRunner;
use RichEditorAuy\Process\SystemProcessRunner;

/**
 * Validates canonical bytes with the `rich-editor-auy` CLI.
 *
 * The binary is always injected; the runner defaults to the system
 * implementation but can be replaced by any {@see ProcessRunner}.
 */
final class Validator
{
    private readonly string $binary;

    private readonly ProcessRunner $runner;

    /**
     * @throws \InvalidArgumentException when the binary path is empty or holds a NUL byte
     */
    public function __construct(string $binary, ?ProcessRunner $runner = null)
    {
        if ($binary === '') {
            throw new \InvalidArgumentException('validator binary must not be empty');
        }

        if (str_contains($binary, "\0")) {
            throw new \InvalidArgumentException('validator binary must not contain NUL bytes');
        }

        $this->binary = $binary;
        $this->runner = $runner ?? new SystemProcessRunner();
    }

    /**
     * Runs `validate` with the given bytes on stdin and parses the report.
     *
     * @throws TransportError when the process cannot be executed or read
     */
    public function validate(string $bytes): ValidationReport
    {
        $result = $this->runner->run([$this->binary, 'validate'], $bytes);

        return ValidationReport::fromProcess($result->exitCode, $result->stdout, $result->stderr);
    }
}
