<?php

declare(strict_types=1);

namespace RichEditorAuy;

final class ValidationReport
{
    /**
     * @param list<ValidationIssue> $issues
     */
    public function __construct(
        public readonly int $exitCode,
        public readonly string $stdout,
        public readonly string $stderr,
        public readonly array $issues,
    ) {
    }

    public static function fromProcess(int $exitCode, string $stdout, string $stderr): self
    {
        $issues = [];
        $lines = preg_split('/\R/', trim($stderr));
        foreach ($lines === false ? [] : $lines as $line) {
            if ($line === '') {
                continue;
            }
            $issues[] = self::parseLine($line);
        }

        return new self($exitCode, $stdout, $stderr, $issues);
    }

    public function ok(): bool
    {
        return $this->exitCode === 0;
    }

    private static function parseLine(string $line): ValidationIssue
    {
        if (preg_match('/^(?<code>\S+) at (?<offset>\d+) \((?<path>.*)\)$/', $line, $matches) === 1) {
            return new ValidationIssue(
                $matches['code'],
                (int) $matches['offset'],
                $matches['path'],
            );
        }

        return new ValidationIssue('validator_error', 0, $line);
    }
}
