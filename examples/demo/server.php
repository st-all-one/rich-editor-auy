<?php

declare(strict_types=1);

require_once __DIR__ . '/../../packages/php/src/ValidationIssue.php';
require_once __DIR__ . '/../../packages/php/src/ValidationReport.php';
require_once __DIR__ . '/../../packages/php/src/Validator.php';

use RichEditorAuy\ValidationReport;
use RichEditorAuy\Validator;

const DEMO_SEED = __DIR__ . '/seed.json';
const DEMO_DATA = __DIR__ . '/data/document.json';

$path = parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH);
$path = is_string($path) ? $path : '/';

if (str_starts_with($path, '/api/')) {
    handle_api($path);
    exit;
}

if ($path === '/' || $path === '/index.html') {
    header('Content-Type: text/html; charset=utf-8');
    readfile(__DIR__ . '/index.html');
    exit;
}

if (preg_match('#^/(?:main\.js|rich_editor_auy_wasm_bg\.wasm|styles\.css|seed\.json|assets/[A-Za-z0-9._-]+)$#', $path) === 1) {
    $file = __DIR__ . $path;
    if (is_file($file)) {
        return false;
    }
}

http_response_code(404);
header('Content-Type: text/plain; charset=utf-8');
echo "not found\n";

function handle_api(string $path): void
{
    try {
        $method = $_SERVER['REQUEST_METHOD'] ?? 'GET';
        switch ($method . ' ' . $path) {
            case 'GET /api/document':
                read_document();
                return;
            case 'POST /api/document':
                save_document();
                return;
            case 'POST /api/validate':
                validate_document();
                return;
            case 'POST /api/canonical':
                canonicalize_document();
                return;
            case 'POST /api/reset':
                reset_document();
                return;
            case 'GET /api/health':
                json_response(200, [
                    'cli' => cli_path(),
                    'executable' => is_executable(cli_path()),
                ]);
                return;
            default:
                json_response(404, ['error' => 'unknown API route']);
        }
    } catch (Throwable $error) {
        json_response(500, ['error' => $error->getMessage()]);
    }
}

function read_document(): void
{
    $bytes = is_file(DEMO_DATA) ? file_get_contents(DEMO_DATA) : file_get_contents(DEMO_SEED);
    if ($bytes === false) {
        throw new RuntimeException('could not read document');
    }
    header('Content-Type: application/json; charset=utf-8');
    header('Cache-Control: no-store');
    echo $bytes;
}

function save_document(): void
{
    $bytes = request_body();
    $report = new Validator(cli_path())->validate($bytes);
    if (!$report->ok()) {
        json_response(422, [
            'accepted' => false,
            'report' => report_json($report),
        ]);
        return;
    }

    $directory = dirname(DEMO_DATA);
    if (!is_dir($directory) && !mkdir($directory, 0o775, true) && !is_dir($directory)) {
        throw new RuntimeException('could not create data directory');
    }
    if (file_put_contents(DEMO_DATA, $bytes) === false) {
        throw new RuntimeException('could not save document');
    }

    json_response(200, [
        'accepted' => true,
        'bytes' => strlen($bytes),
        'report' => report_json($report),
    ]);
}

function validate_document(): void
{
    $report = new Validator(cli_path())->validate(request_body());
    json_response(200, report_json($report));
}

function canonicalize_document(): void
{
    $bytes = request_body();
    [$exit, $stdout, $stderr] = run_cli(['format'], $bytes);
    json_response(200, [
        'ok' => $exit === 0,
        'exit' => $exit,
        'bytes' => trim($stdout),
        'stderr' => trim($stderr),
    ]);
}

function reset_document(): void
{
    if (is_file(DEMO_DATA)) {
        unlink(DEMO_DATA);
    }
    read_document();
}

function request_body(): string
{
    $bytes = file_get_contents('php://input');
    if ($bytes === false) {
        throw new RuntimeException('could not read request body');
    }
    if (strlen($bytes) > 8 * 1024 * 1024) {
        throw new RuntimeException('request body too large');
    }
    return $bytes;
}

function cli_path(): string
{
    $configured = getenv('AUY_CLI');
    if (is_string($configured) && $configured !== '') {
        return $configured;
    }
    return dirname(__DIR__, 2) . '/target/debug/rich-editor-auy-cli';
}

/**
 * @param list<string> $arguments
 * @return array{0:int,1:string,2:string}
 */
function run_cli(array $arguments, string $input): array
{
    $stdoutPath = tempnam(sys_get_temp_dir(), 'auy-demo-out-');
    $stderrPath = tempnam(sys_get_temp_dir(), 'auy-demo-err-');
    if ($stdoutPath === false || $stderrPath === false) {
        throw new RuntimeException('could not create CLI temporary files');
    }

    try {
        $process = proc_open(
            [cli_path(), ...$arguments],
            [
                0 => ['pipe', 'r'],
                1 => ['file', $stdoutPath, 'w'],
                2 => ['file', $stderrPath, 'w'],
            ],
            $pipes,
        );
        if (!is_resource($process)) {
            throw new RuntimeException('could not start CLI');
        }

        $offset = 0;
        $length = strlen($input);
        while ($offset < $length) {
            $written = fwrite($pipes[0], substr($input, $offset));
            if ($written === false || $written === 0) {
                fclose($pipes[0]);
                proc_close($process);
                throw new RuntimeException('could not write CLI input');
            }
            $offset += $written;
        }
        fclose($pipes[0]);

        $exit = proc_close($process);
        $stdout = file_get_contents($stdoutPath);
        $stderr = file_get_contents($stderrPath);
        if ($stdout === false || $stderr === false) {
            throw new RuntimeException('could not read CLI output');
        }
        return [$exit, $stdout, $stderr];
    } finally {
        @unlink($stdoutPath);
        @unlink($stderrPath);
    }
}

/**
 * @return array<string, mixed>
 */
function report_json(ValidationReport $report): array
{
    return [
        'ok' => $report->ok(),
        'exit' => $report->exitCode,
        'stdout' => trim($report->stdout),
        'stderr' => trim($report->stderr),
        'issues' => array_map(
            static fn ($issue): array => [
                'code' => $issue->code,
                'offset' => $issue->offset,
                'path' => $issue->path,
            ],
            $report->issues,
        ),
    ];
}

/**
 * @param array<string, mixed> $payload
 */
function json_response(int $status, array $payload): void
{
    http_response_code($status);
    header('Content-Type: application/json; charset=utf-8');
    header('Cache-Control: no-store');
    echo json_encode($payload, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE | JSON_THROW_ON_ERROR);
}
