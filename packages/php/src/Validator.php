<?php

declare(strict_types=1);

namespace RichEditorAuy;

final class Validator
{
    public function __construct(private readonly string $binary)
    {
    }

    /**
     * Runs the CLI and returns its structured report.
     */
    public function validate(string $bytes): ValidationReport
    {
        $result = $this->run($bytes);

        return ValidationReport::fromProcess($result['exit'], $result['stdout'], $result['stderr']);
    }

    /**
     * Runs the CLI and returns the raw process result.
     *
     * @return array{exit:int,stdout:string,stderr:string}
     */
    public function run(string $bytes): array
    {
        $stdoutPath = tempnam(sys_get_temp_dir(), 'auy-out-');
        $stderrPath = tempnam(sys_get_temp_dir(), 'auy-err-');
        if ($stdoutPath === false || $stderrPath === false) {
            throw new \RuntimeException('could not create validator temporary files');
        }

        try {
            $process = proc_open(
                [$this->binary, 'validate'],
                [
                    0 => ['pipe', 'r'],
                    1 => ['file', $stdoutPath, 'w'],
                    2 => ['file', $stderrPath, 'w'],
                ],
                $pipes,
            );
            if (!is_resource($process)) {
                throw new \RuntimeException('could not start validator');
            }

            $offset = 0;
            $length = strlen($bytes);
            while ($offset < $length) {
                $written = fwrite($pipes[0], substr($bytes, $offset));
                if ($written === false || $written === 0) {
                    fclose($pipes[0]);
                    proc_close($process);
                    throw new \RuntimeException('could not write validator input');
                }
                $offset += $written;
            }
            fclose($pipes[0]);

            $exit = proc_close($process);
            $stdout = file_get_contents($stdoutPath);
            $stderr = file_get_contents($stderrPath);
            if ($stdout === false || $stderr === false) {
                throw new \RuntimeException('could not read validator output');
            }

            return ['exit' => $exit, 'stdout' => $stdout, 'stderr' => $stderr];
        } finally {
            @unlink($stdoutPath);
            @unlink($stderrPath);
        }
    }
}
