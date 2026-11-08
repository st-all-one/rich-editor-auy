<?php

declare(strict_types=1);

require __DIR__ . '/../../packages/php/src/Envelope.php';
require __DIR__ . '/../../packages/php/src/Document.php';

use RichEditorAuy\Document;

$bytes = '{"format":"rich-editor-auy","version":1,"schema_fingerprint":"' . str_repeat('a', 64) . '","doc":{"type":"doc","content":[]}}';
$document = Document::fromBytes($bytes);
if ($document->bytes() !== $bytes) {
    throw new RuntimeException('round-trip failed');
}
echo "ok\n";
