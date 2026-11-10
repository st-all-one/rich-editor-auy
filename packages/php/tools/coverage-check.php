<?php

declare(strict_types=1);

/**
 * Fail when Clover statement coverage is below the requested percentage.
 *
 * Usage: php tools/coverage-check.php [build/coverage.xml] [100]
 */

$argv = $_SERVER['argv'] ?? [];
$pathArgument = is_array($argv) ? $argv[1] ?? null : null;
$minimumArgument = is_array($argv) ? $argv[2] ?? null : null;

$path = is_string($pathArgument) ? $pathArgument : 'build/coverage.xml';
$minimumRaw = is_string($minimumArgument) ? $minimumArgument : '100';

if (preg_match('/\A\d{1,3}\z/', $minimumRaw) !== 1) {
    fwrite(STDERR, "coverage-check: minimum must be an integer between 0 and 100\n");
    exit(2);
}

$minimum = (int) $minimumRaw;

if ($minimum > 100) {
    fwrite(STDERR, "coverage-check: minimum must be between 0 and 100\n");
    exit(2);
}

if (!is_file($path)) {
    fwrite(STDERR, "coverage-check: file not found: {$path}\n");
    exit(2);
}

$xml = simplexml_load_file($path);

if ($xml === false) {
    fwrite(STDERR, "coverage-check: invalid XML: {$path}\n");
    exit(2);
}

$project = $xml->project;

if (!$project instanceof \SimpleXMLElement) {
    fwrite(STDERR, "coverage-check: missing <project> in {$path}\n");
    exit(2);
}

$metrics = $project->metrics;

if (!$metrics instanceof \SimpleXMLElement) {
    fwrite(STDERR, "coverage-check: missing <metrics> in {$path}\n");
    exit(2);
}

$statementsAttribute = $metrics['statements'] ?? null;
$coveredAttribute = $metrics['coveredstatements'] ?? null;

if ($statementsAttribute === null || $coveredAttribute === null) {
    fwrite(STDERR, "coverage-check: missing statement metrics in {$path}\n");
    exit(2);
}

$statements = (int) $statementsAttribute;
$covered = (int) $coveredAttribute;

if ($statements === 0) {
    fwrite(STDERR, "coverage-check: no statements found in {$path}\n");
    exit(2);
}

$percent = ($covered / $statements) * 100;
printf("coverage: %d/%d statements (%.2f%%), required %d%%\n", $covered, $statements, $percent, $minimum);

if (($percent + 0.0001) < $minimum) {
    fwrite(STDERR, "coverage-check: FAILED\n");
    exit(1);
}

echo "coverage-check: OK\n";
