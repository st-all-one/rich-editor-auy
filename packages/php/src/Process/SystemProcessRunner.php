<?php

declare(strict_types=1);

namespace RichEditorAuy\Process;

use RichEditorAuy\Exception\TransportError;

/**
 * Runs the CLI with proc_open, a monotonic deadline and non-blocking pipes.
 *
 * This is the only class in the package that touches the operating system: no
 * shell, no temporary files and no sys_get_temp_dir(). Both output pipes are
 * drained concurrently, so a chatty child cannot deadlock the caller.
 *
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 */
final class SystemProcessRunner implements ProcessRunner
{
    private const CHUNK = 65_536;

    private readonly float $timeoutSeconds;

    /**
     * @throws \InvalidArgumentException when the timeout is not a positive finite number
     */
    public function __construct(float $timeoutSeconds = 30.0)
    {
        if (!is_finite($timeoutSeconds) || $timeoutSeconds <= 0.0) {
            throw new \InvalidArgumentException('process timeout must be greater than zero');
        }

        $this->timeoutSeconds = $timeoutSeconds;
    }

    public function run(array $command, string $stdin): ProcessResult
    {
        /** @var array<int, resource> $pipes */
        $pipes = [];
        $process = $this->start($command, $pipes);

        try {
            [$stdout, $stderr] = $this->pump($pipes, $stdin);
        } catch (TransportError $error) {
            // A child that ignores SIGTERM must not keep the caller waiting.
            proc_terminate($process, 9);

            throw $error;
        } finally {
            $this->closePipes($pipes);
            $exitCode = proc_close($process);
        }

        return new ProcessResult($exitCode, $stdout, $stderr);
    }

    /**
     * @param non-empty-list<string> $command
     * @param array<int, resource> $pipes
     *
     * @return resource
     *
     * @param-out array{0: resource, 1: resource, 2: resource} $pipes
     *
     * @throws TransportError when the process cannot be started
     */
    private function start(array $command, array &$pipes)
    {
        // @codeCoverageIgnoreStart
        // proc_open is absent only on hosts that disable it; not reachable portably.
        if (!function_exists('proc_open')) {
            throw new TransportError('proc_open is unavailable; install or enable it, or inject a ProcessRunner');
        }
        // @codeCoverageIgnoreEnd

        set_error_handler(static function (int $severity, string $message): bool {
            // @codeCoverageIgnoreStart
            // Environment failures (exec limits, missing loader) cannot be forced deterministically.
            throw new TransportError('could not start validator process: ' . $message);

            // @codeCoverageIgnoreEnd
        });

        $descriptors = [0 => ['pipe', 'r'], 1 => ['pipe', 'w'], 2 => ['pipe', 'w']];
        $opened = [];

        try {
            $process = proc_open($command, $descriptors, $opened);
        } catch (\ValueError $error) {
            throw new TransportError('could not start validator process: ' . $error->getMessage(), 0, $error);
        } finally {
            restore_error_handler();
        }

        $stdinPipe = $opened[0] ?? null;
        $stdoutPipe = $opened[1] ?? null;
        $stderrPipe = $opened[2] ?? null;

        // @codeCoverageIgnoreStart
        // proc_open returns a resource only after filling exactly the requested pipes.
        if (!is_resource($stdinPipe) || !is_resource($stdoutPipe) || !is_resource($stderrPipe)) {
            throw new TransportError('could not start validator process');
        }
        // @codeCoverageIgnoreEnd

        $pipes = [$stdinPipe, $stdoutPipe, $stderrPipe];

        // @codeCoverageIgnoreStart
        // A failed proc_open reports a non-resource instead of throwing.
        if (!is_resource($process)) {
            throw new TransportError('could not start validator process');
        }
        // @codeCoverageIgnoreEnd

        return $process;
    }

    /**
     * Drains stdout and stderr while feeding stdin, until both close or the deadline expires.
     *
     * @param array{0: resource, 1: resource, 2: resource} $pipes
     *
     * @return array{0: string, 1: string}
     *
     * @throws TransportError when the deadline expires or the pipes cannot be waited on
     *
     * @mago-expect lint:halstead
     */
    private function pump(array $pipes, string $stdin): array
    {
        stream_set_blocking($pipes[0], false);
        stream_set_blocking($pipes[1], false);
        stream_set_blocking($pipes[2], false);

        /** @var array<int, resource> $open */
        $open = [1 => $pipes[1], 2 => $pipes[2]];
        /** @var array<int, list<string>> $buffers */
        $buffers = [1 => [], 2 => []];
        /** @var array<int, int> $owners */
        $owners = [get_resource_id($pipes[1]) => 1, get_resource_id($pipes[2]) => 2];

        $stdinOpen = $stdin !== '';
        $offset = 0;
        if (!$stdinOpen) {
            fclose($pipes[0]);
        }

        $deadline = (int) hrtime(true) + (int) ($this->timeoutSeconds * 1_000_000_000.0);

        while ($open !== []) {
            $remaining = $deadline - (int) hrtime(true);
            if ($remaining <= 0) {
                throw new TransportError(sprintf('validator timed out after %.3F seconds', $this->timeoutSeconds));
            }

            $readable = array_values($open);
            $writable = $stdinOpen ? [$pipes[0]] : [];
            $except = null;
            $ready = stream_select(
                $readable,
                $writable,
                $except,
                intdiv($remaining, 1_000_000_000),
                intdiv($remaining % 1_000_000_000, 1000),
            );

            // @codeCoverageIgnoreStart
            // Only interrupted or invalid descriptors make stream_select fail.
            if ($ready === false) {
                throw new TransportError('could not wait for validator process');
            }
            // @codeCoverageIgnoreEnd

            if ($stdinOpen && $writable !== []) {
                $stdinOpen = $this->feed($pipes[0], $stdin, $offset);
            }

            foreach ($readable as $stream) {
                $index = $owners[get_resource_id($stream)] ?? null;
                // @codeCoverageIgnoreStart
                // stream_select only returns descriptors taken from the open pipes.
                if ($index === null) {
                    continue;
                }
                // @codeCoverageIgnoreEnd

                $chunk = fread($stream, self::CHUNK);
                if ($chunk !== false && $chunk !== '') {
                    $buffers[$index][] = $chunk;

                    continue;
                }

                if (feof($stream)) {
                    fclose($stream);
                    unset($open[$index]);
                }
            }
        }

        return [implode('', $buffers[1]), implode('', $buffers[2])];
    }

    /**
     * Writes the next stdin chunk.
     *
     * @param resource $stdin
     *
     * @return bool whether stdin stays open
     */
    private function feed($stdin, string $payload, int &$offset): bool
    {
        // @codeCoverageIgnoreStart
        // EPIPE is a normal end of input; the write result already reports it.
        set_error_handler(static fn(): bool => true);
        // @codeCoverageIgnoreEnd

        try {
            $written = fwrite($stdin, substr($payload, $offset, self::CHUNK));
        } finally {
            restore_error_handler();
        }

        if ($written === false || $written === 0) {
            fclose($stdin);

            return false;
        }

        $offset += $written;
        if ($offset >= strlen($payload)) {
            fclose($stdin);

            return false;
        }

        return true;
    }

    /**
     * @param array<int, resource> $pipes
     */
    private function closePipes(array $pipes): void
    {
        foreach ($pipes as $pipe) {
            if (!is_resource($pipe)) {
                continue;
            }

            fclose($pipe);
        }
    }
}
