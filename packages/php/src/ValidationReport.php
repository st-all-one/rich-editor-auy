<?php

declare(strict_types=1);

namespace RichEditorAuy;

/**
 * Typed view of one CLI validation run.
 *
 * Every line of stderr lands in exactly one bucket: a structured issue, a raw
 * CLI diagnostic (`error: ...` / `usage: ...`) or an unrecognized line. Nothing
 * is invented and nothing is dropped, so a future CLI format is visible instead
 * of being misreported as a validation error.
 */
final class ValidationReport
{
    private const ISSUE_PATTERN = '/\A([A-Za-z][A-Za-z0-9_]*) at (\d+) \(([^()]*)\)\z/';

    private const DIAGNOSTIC_PATTERN = '/\Aerror: ([a-z][a-z0-9_]*) at offset (\d+)\z/';

    /**
     * @param list<ValidationIssue> $issues
     * @param list<string> $diagnostics
     * @param list<string> $unrecognized
     *
     * @mago-expect lint:excessive-parameter-list
     */
    private function __construct(
        public readonly int $exitCode,
        public readonly string $stdout,
        public readonly string $stderr,
        public readonly array $issues,
        public readonly array $diagnostics,
        public readonly array $unrecognized,
    ) {}

    /**
     * Builds a report from the raw process result.
     */
    public static function fromProcess(int $exitCode, string $stdout, string $stderr): self
    {
        /** @var list<ValidationIssue> $issues */
        $issues = [];
        /** @var list<string> $diagnostics */
        $diagnostics = [];
        /** @var list<string> $unrecognized */
        $unrecognized = [];

        $normalized = str_replace("\r\n", "\n", $stderr);
        foreach (explode("\n", $normalized) as $line) {
            if ($line === '') {
                continue;
            }

            if (preg_match(self::ISSUE_PATTERN, $line, $matches) === 1) {
                $issues[] = new ValidationIssue($matches[1], (int) $matches[2], $matches[3]);
                continue;
            }

            if (preg_match(self::DIAGNOSTIC_PATTERN, $line, $matches) === 1) {
                $issues[] = new ValidationIssue($matches[1], (int) $matches[2], '');
                continue;
            }

            if (str_starts_with($line, 'error: ') || str_starts_with($line, 'usage: ')) {
                $diagnostics[] = $line;
                continue;
            }

            $unrecognized[] = $line;
        }

        return new self($exitCode, $stdout, $stderr, $issues, $diagnostics, $unrecognized);
    }

    /**
     * The exit code is the authority; stdout and stderr are evidence.
     */
    public function ok(): bool
    {
        return $this->exitCode === 0;
    }
}
