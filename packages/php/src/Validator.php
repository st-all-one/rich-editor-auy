<?php

declare(strict_types=1);

namespace RichEditorAuy;

final class Validator
{
    public function __construct(private readonly string $binary)
    {
    }

    /** @return array{exit:int,stdout:string,stderr:string} */
    public function validate(string $bytes): array
    {
        $process = proc_open(
            [$this->binary, 'validate'],
            [0 => ['pipe', 'r'], 1 => ['pipe', 'w'], 2 => ['pipe', 'w']],
            $pipes,
        );
        if (!is_resource($process)) {
            throw new \RuntimeException('could not start validator');
        }
        fwrite($pipes[0], $bytes);
        fclose($pipes[0]);
        $stdout = stream_get_contents($pipes[1]);
        $stderr = stream_get_contents($pipes[2]);
        fclose($pipes[1]);
        fclose($pipes[2]);
        $exit = proc_close($process);
        return ['exit' => $exit, 'stdout' => $stdout === false ? '' : $stdout, 'stderr' => $stderr === false ? '' : $stderr];
    }
}
