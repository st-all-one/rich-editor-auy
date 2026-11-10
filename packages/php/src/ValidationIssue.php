<?php

declare(strict_types=1);

namespace RichEditorAuy;

/**
 * One structured finding reported by the CLI.
 *
 * `path` is empty when the CLI could not attach a JSON path to the finding,
 * which is the case for parse errors.
 */
final class ValidationIssue
{
    public function __construct(
        public readonly string $code,
        public readonly int $offset,
        public readonly string $path,
    ) {}
}
