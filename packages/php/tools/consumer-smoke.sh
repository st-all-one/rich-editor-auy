#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cat > "$tmp/composer.json" <<JSON
{
    "repositories": [
        {
            "type": "path",
            "url": "${root}",
            "options": {
                "symlink": true,
                "versions": {
                    "rich-editor-auy/php": "0.1.0"
                }
            }
        }
    ],
    "require": {
        "rich-editor-auy/php": "^0.1"
    },
    "minimum-stability": "dev",
    "prefer-stable": true
}
JSON

cat > "$tmp/smoke.php" <<'PHP'
<?php

declare(strict_types=1);

use RichEditorAuy\Document;
use RichEditorAuy\Process\ProcessResult;
use RichEditorAuy\Process\ProcessRunner;
use RichEditorAuy\Validator;

require __DIR__ . '/vendor/autoload.php';

$bytes = '{"format":"rich-editor-auy","version":2,"schema_fingerprint":"'
    . str_repeat('a', 64)
    . '","doc":{"type":"doc","content":[]}}';

$document = Document::fromBytes($bytes);

$runner = new class implements ProcessRunner {
    public function run(array $command, string $stdin): ProcessResult
    {
        return new ProcessResult(0, $stdin === '' ? '' : "ok\n", '');
    }
};

$report = (new Validator('/usr/local/bin/auy', $runner))->validate($bytes);

$checks = [
    'bytes preserved' => $document->bytes() === $bytes,
    'version read' => $document->envelope()->version === 2,
    'fingerprint read' => $document->envelope()->schemaFingerprint === str_repeat('a', 64),
    'doc read' => is_array($document->json()['doc'] ?? null),
    'report ok' => $report->ok(),
    'report stdout' => $report->stdout === "ok\n",
    'report empty issues' => $report->issues === [],
];

foreach ($checks as $label => $ok) {
    if (!$ok) {
        fwrite(STDERR, "consumer-smoke: {$label} failed\n");
        exit(1);
    }
}

echo "consumer-smoke: OK\n";
PHP

cd "$tmp"
composer install --no-interaction --no-progress --no-dev --no-scripts >/dev/null
php smoke.php
