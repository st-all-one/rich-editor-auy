<?php

declare(strict_types=1);

namespace RichEditorAuy;

final class ValidationIssue
{
    public function __construct(
        public readonly string $code,
        public readonly int $offset,
        public readonly string $path,
    ) {
    }
}
