# PHP 8.2+: endurecer e tornar agnóstico o pacote `rich-editor-auy/php`

## Contexto

O pedido é uma atualização profunda de `packages/php`, tornando o pacote mais rigoroso e agnóstico,
inspirado no pacote `../blake3php/blake3-for-php` (PHPStan `max` com strict/deprecation/PHPUnit/dead
code, Mago, PHPUnit estrito, cobertura 100%, imagens fixadas por digest, `Makefile` único, smoke de
consumidor). Requisito adicional do usuário: o pacote deve funcionar **a partir do PHP 8.2**.

Hoje o pacote tem 5 classes, 3 arquivos de teste, PHPStan `max` sem regras extras, PHPUnit sem
flags de rigor, `Validator` com `proc_open` + arquivos temporários no próprio corpo, parsing de
`stderr` que transforma qualquer linha desconhecida em `validator_error` (inclusive erros de parse
legítimos do CLI, cujo `Display` é `error: <code> at offset <n>`), `Envelope` com construtor público
que permite estado inválido, regex de `doc_id` com `$` (aceita `\n` final), sem `composer.lock`
versionado, sem linter/formatter, sem cobertura, sem `Makefile` e sem smoke.

Estado final: transporte fino e determinístico, sem framework, sem dependência de runtime além de
`php ^8.2`, `php-64bit` e `ext-json`; a camada de valores não faz I/O; a execução do CLI é um port
injetável (`ProcessRunner`), com implementação de sistema única e testável; a gramática do `stderr`
do CLI é documentada e preservada; o gate roda em 8.2/8.3/8.4/8.5 (glibc) e 8.2 (musl), com
cobertura de linhas 100% em `src/`.

Fronteira que não muda: o PHP **não** implementa schema, normalização, steps ou fingerprint. Ele
valida apenas o contrato do envelope (formato, versão, fingerprint de 64 hex, `doc_id`) e a forma
mínima do JSON; validação semântica é do CLI (`AGENTS.md` §3 regra 14 e `plan/09` §2.1).

## Decisões fechadas

- **Agnóstico** = sem framework, sem PSR, sem dependência de runtime, sem assumir o mecanismo de
  execução. Toda a I/O de processo fica em `RichEditorAuy\Process\SystemProcessRunner`; o resto do
  pacote depende só da interface `ProcessRunner`. O binário do CLI é sempre injetado; comando é
  array (sem shell), sem arquivos temporários e sem `sys_get_temp_dir()`.
- **PHP 8.2** é o piso (`php: ^8.2`), e a matriz do gate cobre 8.2, 8.3, 8.4 e 8.5. Por isso:
  PHPUnit 11.5.57 (o 12/13 exige 8.3/8.4), `phpstan.neon` com `phpVersion: 80200`, Mago com
  `php-version = "8.2"`, e nenhuma sintaxe 8.3+ (constantes de classe tipadas, `#[\Override]`,
  `json_validate`) no código.
- **Uma única fonte de parsing do relatório**: `ValidationReport::fromProcess()` é o único
  construtor (privado). Linhas reconhecidas viram `ValidationIssue`; linhas `error: ...`/`usage: ...`
  viram `diagnostics` cruas; o resto vira `unrecognized` cru. Nada é inventado e nada é descartado.
- **Sem linha nova em `DIVERGENCES.md`**: `plan/09` passa a declarar PHP 8.2+ e a matriz
  8.2/8.3/8.4/8.5, então o plano e a implementação coincidem. Não criar divergência artificial.

## Abordagem

### 1. Metadados, licença e configuração estática do pacote

1. Substituir `packages/php/composer.json` por (JSON exato; `require-dev` fixo, sem `^`):

```json
{
    "$schema": "https://getcomposer.org/schema.json",
    "name": "rich-editor-auy/php",
    "description": "Thin, dependency-free PHP transport for canonical rich-editor-auy documents",
    "type": "library",
    "license": "MIT OR Apache-2.0",
    "keywords": ["rich-text", "editor", "json", "validation", "accessibility", "headless", "composer"],
    "homepage": "https://github.com/st-all-one/rich-editor-auy",
    "support": {
        "issues": "https://github.com/st-all-one/rich-editor-auy/issues",
        "source": "https://github.com/st-all-one/rich-editor-auy"
    },
    "authors": [
        {"name": "st-all-one"}
    ],
    "require": {
        "php": "^8.2",
        "php-64bit": "*",
        "ext-json": "*"
    },
    "require-dev": {
        "carthage-software/mago": "1.59.0",
        "phpstan/phpstan": "2.3.5",
        "phpstan/phpstan-deprecation-rules": "2.0.5",
        "phpstan/phpstan-phpunit": "2.1.1",
        "phpstan/phpstan-strict-rules": "2.1.1",
        "phpunit/phpunit": "11.5.57",
        "shipmonk/dead-code-detector": "1.4.2"
    },
    "autoload": {
        "psr-4": {"RichEditorAuy\\": "src/"}
    },
    "autoload-dev": {
        "psr-4": {"RichEditorAuy\\Tests\\": "tests/"}
    },
    "scripts": {
        "check": [
            "composer validate --strict --no-check-publish",
            "@lint",
            "@static",
            "@test"
        ],
        "coverage": [
            "mkdir -p build",
            "@php -d pcov.enabled=1 vendor/bin/phpunit --coverage-clover=build/coverage.xml",
            "@php tools/coverage-check.php build/coverage.xml 100"
        ],
        "fmt": "vendor/bin/mago format",
        "lint": [
            "vendor/bin/mago lint",
            "vendor/bin/mago format --check"
        ],
        "smoke": "bash tools/consumer-smoke.sh",
        "static": "vendor/bin/phpstan analyse --no-progress --memory-limit=1G",
        "test": "vendor/bin/phpunit"
    },
    "config": {
        "allow-plugins": false,
        "optimize-autoloader": true,
        "platform": {"php": "8.2.0"},
        "preferred-install": "dist",
        "sort-packages": true
    },
    "minimum-stability": "stable",
    "prefer-stable": true
}
```

2. Licença: sobrescrever `packages/php/LICENSE` com o Apache-2.0 do root (`LICENSE` é Apache-2.0) e
   copiar `LICENSE-MIT` do root para `packages/php/LICENSE-MIT`. O campo `license` dual passa a ter
   os dois textos no pacote. Não alterar o texto da licença.

3. Remover a linha `packages/php/composer.lock` de `.gitignore` (raiz). Manter
   `packages/php/vendor/`. O lock passa a ser versionado; `composer validate --strict` falha se
   `composer.json` e lock divergirem.

4. Criar `.editorconfig` (4 espaços; tabs no `Makefile`; LF; newline final; `charset=utf-8`),
   `.gitattributes` (`* text=auto eol=lf`, `export-ignore` para `tests/`, `docker/`, `tools/`,
   `Makefile`, `mago.toml`, `phpstan.neon`, `phpunit.xml`) e `.gitignore` local
   (`/vendor/`, `/build/`, `/.phpunit.cache/`, `/.phpunit.result.cache`). Criar `.dockerignore`
   (`vendor`, `build`, `.phpunit.cache`, `.git`). Esses arquivos são mecânicos; os valores exatos
   seguem o pacote de referência citado no Contexto.

5. Substituir `packages/php/phpstan.neon` por:

```neon
includes:
    - vendor/phpstan/phpstan-strict-rules/rules.neon
    - vendor/phpstan/phpstan-deprecation-rules/rules.neon
    - vendor/phpstan/phpstan-phpunit/extension.neon
    - vendor/phpstan/phpstan/conf/bleedingEdge.neon
    - vendor/shipmonk/dead-code-detector/rules.neon

parameters:
    level: max
    phpVersion: 80200
    paths:
        - src
        - tests
        - tools
    reportUnmatchedIgnoredErrors: true
    treatPhpDocTypesAsCertain: true
    checkUninitializedProperties: true
    checkBenevolentUnionTypes: true
    checkExplicitMixed: true
    checkImplicitMixed: true
```

Sem baseline e sem `ignoreErrors`. Qualquer achado é corrigido no código; não adicionar ignores
amplos.

6. Substituir `packages/php/phpunit.xml` por (schema do PHPUnit 11; `requireCoverageMetadata` exige
   `#[CoversClass]` em toda classe de teste):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<phpunit xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:noNamespaceSchemaLocation="vendor/phpunit/phpunit/phpunit.xsd"
         bootstrap="vendor/autoload.php"
         colors="true"
         cacheDirectory=".phpunit.cache"
         failOnWarning="true"
         failOnDeprecation="true"
         failOnNotice="true"
         failOnRisky="true"
         failOnEmptyTestSuite="true"
         beStrictAboutTestsThatDoNotTestAnything="true"
         requireCoverageMetadata="true"
         executionOrder="default"
         resolveDependencies="true">
    <testsuites>
        <testsuite name="RichEditorAuy">
            <directory>tests</directory>
        </testsuite>
    </testsuites>
    <source>
        <include>
            <directory>src</directory>
        </include>
    </source>
    <php>
        <ini name="error_reporting" value="-1"/>
        <ini name="display_errors" value="1"/>
        <ini name="zend.assertions" value="1"/>
        <ini name="memory_limit" value="512M"/>
        <env name="TZ" value="UTC"/>
    </php>
</phpunit>
```

7. Criar `packages/php/mago.toml`:

```toml
version = "1.59.0"
php-version = "8.2"

[source]
paths = ["src", "tests", "tools"]
includes = []
excludes = ["build", "vendor", ".phpunit.cache"]

[formatter]
print-width = 120
end-of-line = "lf"
single-quote = true
trailing-comma = true
use-tabs = false

[linter]
integrations = ["phpunit"]
```

Se o Mago acusar complexidade em `SystemProcessRunner::pump`, adicionar **apenas** no método
`@mago-expect lint:cyclomatic-complexity` com a razão “single non-blocking I/O pump; splitting
duplicates deadline and pipe bookkeeping”. Não adicionar ignores amplos.

### 2. Camada de valores (zero I/O, sem estado inválido)

Criar `src/Exception/` e reescrever as classes abaixo. Todos os arquivos começam com
`declare(strict_types=1);`, usam `final class` e propriedades promovidas `public readonly` quando o
valor é público.

1. `src/Exception/InvalidDocument.php`:

```php
final class InvalidDocument extends \InvalidArgumentException {}
```

2. `src/Exception/IncompatibleEnvelope.php`:

```php
final class IncompatibleEnvelope extends \InvalidArgumentException {}
```

3. `src/Exception/TransportError.php`:

```php
final class TransportError extends \RuntimeException {}
```

Todas as mensagens são em inglês, minúsculas e estáveis; nenhuma interpola `doc_id`/conteúdo do
documento.

4. Reescrever `src/Envelope.php` com contrato exato:

```php
final class Envelope
{
    public const FORMAT = 'rich-editor-auy';
    public const VERSION = 2;
    public const LEGACY_VERSION = 1;

    private function __construct(
        public readonly string $format,
        /** @var 1|2 */
        public readonly int $version,
        public readonly string $schemaFingerprint,
        public readonly ?string $docId,
    ) {}

    /** @param array<array-key, mixed> $json */
    public static function fromArray(array $json): self;

    public function requireCompatible(string $schemaFingerprint): void;
}
```

`fromArray` valida, nesta ordem, e lança `IncompatibleEnvelope`:

- `$json['format'] ?? null` deve ser exatamente `self::FORMAT`; senão
  `envelope format must be "rich-editor-auy"`;
- `$json['version'] ?? null` deve ser `1` ou `2`; senão `envelope version must be 1 or 2`;
- `$json['schema_fingerprint'] ?? null` deve ser string que case com `/\A[0-9a-f]{64}\z/`; senão
  `schema_fingerprint must be 64 lowercase hex characters`;
- `$json['doc_id'] ?? null` deve ser `null` ou string que case com `/\A[\x21-\x7e]{1,128}\z/`;
  senão `doc_id must be 1-128 graphic ASCII characters`.

Não armazenar `doc` (o `Document` é dono do JSON). Ignorar campos desconhecidos (compatibilidade
para frente). `requireCompatible($fingerprint)` retorna sem comparar quando
`$this->version === self::LEGACY_VERSION`; caso contrário, se o fingerprint diferir, lança
`IncompatibleEnvelope('schema fingerprint mismatch')`. O construtor privado garante que nenhum
`Envelope` inválido existe.

5. Reescrever `src/Document.php`:

```php
final class Document
{
    private function __construct(
        private readonly string $bytes,
        /** @var array<array-key, mixed> */
        private readonly array $json,
        private readonly Envelope $envelope,
    ) {}

    public static function fromBytes(string $bytes): self;
    public function bytes(): string;
    /** @return array<array-key, mixed> */
    public function json(): array;
    public function envelope(): Envelope;
}
```

`fromBytes` faz **um** `json_decode($bytes, true, 512, JSON_THROW_ON_ERROR)` dentro de
`try/catch (\JsonException $exception)`, relançando
`InvalidDocument('document must be valid JSON: ' . $exception->getMessage(), 0, $exception)`.
Depois:

- se `!is_array($decoded) || array_is_list($decoded)` → `InvalidDocument('document must be a JSON object')`;
- `$envelope = Envelope::fromArray($decoded)` (propaga `IncompatibleEnvelope`);
- `$doc = $decoded['doc'] ?? null`; se `!is_array($doc) || $doc === [] || array_is_list($doc)` →
  `InvalidDocument('envelope must contain a non-empty doc object')`;
- retorna `new self($bytes, $decoded, $envelope)`.

O `json_decode` associativo não distingue `{}` de `[]`; ambos são rejeitados para `doc`/topo. Isso é
limitação declarada no PHPDoc e no README, não bug. `bytes()` devolve exatamente a string recebida
(incluindo espaços e escapes); `json()` é só para inspeção e nunca deve ser resserializado para
gravar.

6. Manter `src/ValidationIssue.php` com construtor público e propriedades
   `string $code`, `int $offset`, `string $path`; documentar que `path === ''` identifica erro de
   parse (o CLI não imprime path nesse caso).

7. Reescrever `src/ValidationReport.php`:

```php
final class ValidationReport
{
    /**
     * @param list<ValidationIssue> $issues
     * @param list<string> $diagnostics
     * @param list<string> $unrecognized
     */
    private function __construct(
        public readonly int $exitCode,
        public readonly string $stdout,
        public readonly string $stderr,
        public readonly array $issues,
        public readonly array $diagnostics,
        public readonly array $unrecognized,
    ) {}

    public static function fromProcess(int $exitCode, string $stdout, string $stderr): self;
    public function ok(): bool;
}
```

`fromProcess` normaliza apenas CRLF (`str_replace("\r\n", "\n", $stderr)`) e faz `explode("\n")`;
ignora linhas vazias. Para cada linha, nesta ordem:

- `/\A([A-Za-z][A-Za-z0-9_]*) at (\d+) \(([^()]*)\)\z/` → `ValidationIssue(code, offset, path)`
  (cobre `I1`, `I23`, `A001`…`A010`);
- `/\Aerror: ([a-z][a-z0-9_]*) at offset (\d+)\z/` → `ValidationIssue(code, offset, '')` (cobre
  `missing_format`, `unsupported_version`, `missing_doc` etc. do CLI);
- começa com `error: ` ou `usage: ` → `diagnostics[]` com a linha crua;
- qualquer outra linha não vazia → `unrecognized[]` com a linha crua.

`ok()` é `$exitCode === 0` (o código de saída é a autoridade; `stdout`/`stderr` são evidência). O
construtor é privado: não existe relatório fabricado fora do parser.

### 3. Transporte plugável

Criar `src/Process/ProcessResult.php`:

```php
final class ProcessResult
{
    public function __construct(
        public readonly int $exitCode,
        public readonly string $stdout,
        public readonly string $stderr,
    ) {}
}
```

Criar `src/Process/ProcessRunner.php`:

```php
interface ProcessRunner
{
    /**
     * @param non-empty-list<string> $command argv without a shell
     * @return ProcessResult
     * @throws \RichEditorAuy\Exception\TransportError
     */
    public function run(array $command, string $stdin): ProcessResult;
}
```

Criar `src/Process/SystemProcessRunner.php` (`final class ... implements ProcessRunner`):

- Construtor `__construct(float $timeoutSeconds = 30.0)`: rejeita `!is_finite($timeoutSeconds) ||
  $timeoutSeconds <= 0.0` com `\InvalidArgumentException('process timeout must be greater than zero')`.
- `run(array $command, string $stdin): ProcessResult`:
  1. `start()`: se `!function_exists('proc_open')`, lança
     `TransportError('proc_open is unavailable; install or enable it, or inject a ProcessRunner')`
     (bloco `@codeCoverageIgnoreStart/End`). Instala `set_error_handler` temporário que lança
     `TransportError('could not start validator process: ' . $message)`; o corpo do handler é
     `@codeCoverageIgnore` (caminho de ambiente). Chama
     `proc_open($command, [0 => ['pipe','r'], 1 => ['pipe','w'], 2 => ['pipe','w']], $pipes)`.
     `catch (\ValueError $error)` → `TransportError('could not start validator process: ' .
     $error->getMessage(), 0, $error)`; `finally` restaura o handler. Se o resultado não for
     `resource`, lança `TransportError('could not start validator process')` (ignorado para
     cobertura). Retorna o resource; o PHPDoc declara `@return resource` e
     `$pipes` como `array{0: resource, 1: resource, 2: resource}`.
  2. Deadline monotônico: `hrtime(true)` + `(int) ($timeoutSeconds * 1_000_000_000)` (o pacote exige
     `php-64bit`).
  3. `pump(array &$pipes, string $stdin): array{0: string, 1: string}` com `stream_set_blocking(...,
     false)` em stdin/stdout/stderr, buffers em `list<string>` e `implode` no fim:
     - se `$stdin === ''`, fecha `$pipes[0]` antes do loop; senão mantém `$stdinOpen = true`,
       `$offset = 0`, `$length = strlen($stdin)`;
     - enquanto houver saída aberta: calcula o restante pelo deadline; se `<= 0.0`, lança
       `TransportError(sprintf('validator timed out after %.3F seconds', $this->timeoutSeconds))`;
       calcula `$seconds`/`$microseconds`; `$read = array_values($openOutput)`;
       `$write = $stdinOpen ? [$pipes[0]] : []`; `$except = null`;
       `stream_select($read, $write, $except, $seconds, $microseconds)`; se retornar `false`, lança
       `TransportError('could not wait for validator process')` (bloco ignorado para cobertura);
     - se `$write !== []` e `$stdinOpen`: `fwrite($pipes[0], substr($stdin, $offset, 65536))` dentro
       de um handler que suprime warnings (o corpo `return true;` é ignorado para cobertura; EPIPE
       é normal e o retorno `false` é tratado). Se `false`/`0`, fecha stdin e `$stdinOpen = false`;
       senão avança `$offset` e fecha stdin ao atingir `$length`;
     - para cada índice 1/2 ainda aberto e presente em `$read`: `fread(..., 65536)`; se `false` ou
       `''`, fecha/remove o pipe quando `feof`, senão continua; caso contrário anexa ao buffer de
       stdout (índice 1) ou stderr (índice 2). Um pipe pode fechar antes do outro.
  4. Em `run`, `try { [$stdout, $stderr] = $this->pump($pipes, $stdin); } catch (TransportError
     $error) { proc_terminate($process, 9); throw $error; } finally { $this->closePipes($pipes);
     $exitCode = proc_close($process); }` (`$exitCode = -1` e buffers `''` inicializados antes).
     `proc_terminate(..., 9)` garante que o timeout não pendura em filho que ignora SIGTERM.
     `closePipes` fecha só `is_resource(...)`. Código de saída negativo/`-1` é preservado e
     `ok()` fica falso (fail closed); não lançar por isso.
- Linhas com `@codeCoverageIgnoreStart/End`: `function_exists` falso, corpo do handler de
  `proc_open`, `proc_open` não-resource, `stream_select === false`, `return true` do handler de
  `fwrite`. Cada bloco tem comentário de uma linha justificando (falha de ambiente/SO, não
  alcançável de forma determinística). Todo o resto (timeout, EPIPE, pipes assimétricos, ValueError
  de NUL) é testado.

Reescrever `src/Validator.php`:

```php
final class Validator
{
    private readonly ProcessRunner $runner;

    /** @throws \InvalidArgumentException */
    public function __construct(string $binary, ?ProcessRunner $runner = null);

    /** @throws \RichEditorAuy\Exception\TransportError */
    public function validate(string $bytes): ValidationReport;
}
```

O construtor rejeita `$binary === ''` (`validator binary must not be empty`) e
`str_contains($binary, "\0")` (`validator binary must not contain NUL bytes`), e usa
`$runner ?? new SystemProcessRunner()`. `validate` chama `run([$this->binary, 'validate'], $bytes)`
e devolve `ValidationReport::fromProcess(...)`. Remover o método público `run()` da classe: a
execução bruta é responsabilidade do port, não da API do validador.

### 4. Testes

Mover para `tests/Unit/` (namespace `RichEditorAuy\Tests\Unit`) e criar `tests/Support/`
(namespace `RichEditorAuy\Tests\Support`). Remover os três arquivos atuais de `tests/` (cutover
limpo, sem duplicatas).

`tests/Support/EnvelopeJson.php` (helper estático, `final`, construtor privado): gera fixtures com
`sprintf` (não `json_encode`, para controlar bytes exatos):

```php
EnvelopeJson::canonical(?string $docId = null, int $version = 2): string
EnvelopeJson::withDoc(string $doc, int $version = 2): string
EnvelopeJson::fingerprint(): string  // str_repeat('a', 64)
```

`tests/Support/StubProcessRunner.php` implementa `ProcessRunner`, grava
`list<array{command: non-empty-list<string>, stdin: string}> $calls` e devolve o `ProcessResult`
recebido no construtor.

Casos mínimos por classe (cada classe de teste tem `#[CoversClass(...)]`; repetir o atributo quando
cobrir duas classes):

- `DocumentTest` (`Document`):
  - bytes preservados com espaços e escapes (`bytes()` idêntico, `json()` decodificado);
  - envelope v1 e v2 aceitos;
  - JSON malformado (`'{'`) → `InvalidDocument` contendo `document must be valid JSON`;
  - topo lista (`'[]'`) e topo escalar (`'1'`) → `document must be a JSON object`;
  - `doc` ausente, `doc: []`, `doc: {}` → `envelope must contain a non-empty doc object`;
  - `doc_id` válido de 128 chars aceito; 129 chars, com espaço, com `\n` escapado
    (`"doc_id":"post-1\n"`) rejeitados (regressão do `$` sem `D`); `doc_id` não-string rejeitado;
  - `doc_id` ausente aceito; campos extras no envelope aceitos e bytes preservados;
  - `Document::fromBytes` propaga `IncompatibleEnvelope` antes de checar `doc` (ex.: formato
    errado com `doc` ausente).
- `EnvelopeTest` (`Envelope`, via `Envelope::fromArray`):
  - `format` ausente/não-string/errado; `version` ausente/0/3/"2"/2.0 (float) rejeitados; 1 e 2
    aceitos;
  - `schema_fingerprint` ausente/63/65/uppercase/não-hex rejeitados; 64 lowercase aceito;
  - `requireCompatible` em v2 com fingerprint diferente → `IncompatibleEnvelope('schema fingerprint
    mismatch')`; igual → sem exceção; em v1 ignora a comparação;
  - construtor privado: não há teste instanciando `new Envelope`.
- `ValidationReportTest` (`ValidationReport`, `ValidationIssue`):
  - sucesso: `fromProcess(0, "ok\n", '')` → `ok()`, `issues`/`diagnostics`/`unrecognized` vazios,
    `stdout` preservado;
  - `I1 at 0 ()`, `I23 at 42 (3/0)`, `A003 at 7 (2/1)` → issues com code/offset/path exatos;
  - `error: missing_format at offset 0` → issue `missing_format`, offset 0, path `''`;
  - `error: content mismatch at schema_fingerprint`, `error: could not read stdin`, `usage: ...`
    → `diagnostics`, nunca `issues`;
  - linha arbitrária → `unrecognized`;
  - CRLF + linhas vazias normalizados; `stderr` cru preservado; exit 1 sem stderr → `ok()` falso.
- `ValidatorTest` (`Validator`):
  - stub registra `[$binary, 'validate']` e o stdin byte a byte (inclusive bytes não UTF-8);
  - relatório do stub é mapeado;
  - binário vazio e binário com NUL → `\InvalidArgumentException`.
- `SystemProcessRunnerTest` (`SystemProcessRunner`, `ProcessResult`):
  - `[PHP_BINARY, '-r', 'fwrite(STDOUT, "out"); fwrite(STDERR, "err"); exit(3);']` →
    `ProcessResult(3, 'out', 'err')`;
  - stdin de 1 MiB ecoado por `echo stream_get_contents(STDIN);` → stdout idêntico (sem deadlock);
  - filho escreve 200 KiB em stdout e 200 KiB em stderr → ambos exatos (drenagem concorrente);
  - filho fecha stdout antes do stderr (`fclose(STDOUT); usleep(100000); fwrite(STDERR, "late");
    exit(0);`) → stdout `''`, stderr `'late'`, exit 0;
  - filho `exit(0);` com 1 MiB de stdin → exit 0 sem `TransportError` (EPIPE);
  - `usleep(2000000);` com timeout `0.05` → `TransportError` contendo `timed out` (o SIGKILL
    impede o teste de durar 2 s);
  - timeout `0.0`, `-1.0`, `INF`, `NAN` → `\InvalidArgumentException` (data provider);
  - comando `["/nonexistent/auy-binary"]` → `exitCode !== 0` (sem depender do valor 127);
  - comando com NUL (`["/bin/true\0"]`) → `TransportError`;
  - bytes binários com NUL/0xFF ecoados → stdout idêntico.

Não criar scripts temporários com `tempnam`/`chmod`: todos os subprocessos usam `PHP_BINARY` +
`-r`. Não usar `@` em testes.

### 5. Ferramentas

Criar `tools/coverage-check.php` (CLI, `declare(strict_types=1)`), adaptado do pacote de referência:

- uso `php tools/coverage-check.php [build/coverage.xml] [100]`;
- valida argumentos com `is_array`/`is_string` (PHPStan `checkImplicitMixed`), `min`/`max` 0–100;
- `is_file`/`simplexml_load_file` com erros explícitos em `STDERR` e `exit(2)`;
- exige `<project><metrics statements= coveredstatements=>`; `statements === 0` é erro;
- imprime `coverage: %d/%d statements (%.2f%%), required %d%%`; falha (`exit(1)`) se
  `($percent + 0.0001) < $minimum`; sucesso `coverage-check: OK`.

Criar `tools/consumer-smoke.sh` (`#!/usr/bin/env bash`, `set -euo pipefail`, `mktemp -d` + `trap`):

- cria um `composer.json` temporário com `repositories: [{type: path, url: <root>, options:
  {symlink: true, versions: {"rich-editor-auy/php": "0.1.0"}}}]` e
  `require: {"rich-editor-auy/php": "^0.1"}`, `minimum-stability: dev`, `prefer-stable: true`;
- `composer install --no-interaction --no-progress --no-dev --no-scripts`;
- roda um `smoke.php` que exige `vendor/autoload.php`, valida
  `Document::fromBytes($bytes)->bytes() === $bytes`, lê `envelope()->version` e implementa um
  `ProcessRunner` anônimo devolvendo `new ProcessResult(0, "ok\n", '')` para provar que a API de
  runtime funciona sem processo e sem `--dev`;
- imprime `consumer-smoke: OK`.

### 6. Infraestrutura do gate

1. Criar `packages/php/docker/bookworm.Dockerfile`:

```dockerfile
# syntax=docker/dockerfile:1
ARG PHP_TAG=8.2-cli-bookworm
ARG PHP_DIGEST=sha256:ce09ca1d4b532edf9b0347b44a1968a80d9175addb581c450ba0c9a43fa0c5ea
FROM php:${PHP_TAG}@${PHP_DIGEST}

ARG WITH_PCOV=0

ENV COMPOSER_ALLOW_SUPERUSER=1 \
    COMPOSER_HOME=/tmp/composer \
    COMPOSER_ROOT_VERSION=0.1.0 \
    LC_ALL=C \
    TZ=UTC

COPY --from=composer:2@sha256:07cb23c4c9d528524a8b9a67c403d6ad7fef2268651eb1448e3874d40f28b789 /usr/bin/composer /usr/local/bin/composer

RUN apt-get update \
    && apt-get install -y --no-install-recommends git unzip $PHPIZE_DEPS \
    && if [ "$WITH_PCOV" = "1" ]; then pecl install pcov-1.0.12 && docker-php-ext-enable pcov; fi \
    && apt-get purge -y --auto-remove $PHPIZE_DEPS \
    && rm -rf /var/lib/apt/lists/* /tmp/pear

RUN { \
        echo 'date.timezone=UTC'; \
        echo 'memory_limit=512M'; \
        echo 'error_reporting=-1'; \
        echo 'display_errors=1'; \
        echo 'zend.assertions=1'; \
        echo 'pcov.enabled=0'; \
        echo 'pcov.directory=/app/src'; \
    } > /usr/local/etc/php/conf.d/zz-auy-dev.ini

WORKDIR /app
```

2. Criar `packages/php/docker/alpine.Dockerfile` com `ARG PHP_TAG=8.2-cli-alpine`,
   `ARG PHP_DIGEST=sha256:212a8b65f28529edcff248b9140edfac3ed8d651917e0eedb1fe6d3fa6950493`,
   o mesmo `COPY --from=composer...`, `apk add --no-cache git unzip autoconf g++ gcc make pkgconf
   re2c linux-headers`, sem PCOV, `apk del` das ferramentas, o mesmo INI e `WORKDIR /app`.

3. Criar `packages/php/docker/images.tsv` (tabs reais; a coluna alpine vazia é `-`):

```text
# version	bookworm digest	alpine digest
8.2	sha256:ce09ca1d4b532edf9b0347b44a1968a80d9175addb581c450ba0c9a43fa0c5ea	sha256:212a8b65f28529edcff248b9140edfac3ed8d651917e0eedb1fe6d3fa6950493
8.3	sha256:232331112d3a7c5b98edeeddc3b85f610172f99352c0635fe6b3e166e47062cb	-
8.4	sha256:b3e907a1108629c562c4bb313ce3449e34d49991f6bb2487913f6146f47200fe	-
8.5	sha256:050713ce008e2360ea9dc96e1f9a075690132388d917fc3552806f7a7be6f2a0	-
```

Remover `packages/php/Dockerfile` (substituído pelos dois acima; o root `Makefile` não deve referenciar
o antigo).

4. Criar `packages/php/Makefile` (`SHELL := /bin/bash`, `.DEFAULT_GOAL := help`, `DOCKER ?= docker`):

```make
VERSIONS := 8.2 8.3 8.4 8.5
ALPINE_VERSION ?= 8.2
DOCKER_RUN = $(DOCKER) run --rm \
	--security-opt label=disable \
	-u "$$(id -u):$$(id -g)" \
	-e HOME=/tmp \
	-e COMPOSER_ROOT_VERSION=0.1.0 \
	-v "$(CURDIR):/app" \
	-w /app

build-%:
	$(DOCKER) build -f docker/bookworm.Dockerfile \
		--build-arg PHP_TAG=$*-cli-bookworm \
		--build-arg PHP_DIGEST=$$(awk -F '\t' '$$1 == "$*" { print $$2 }' docker/images.tsv) \
		-t rich-editor-auy-php:$*-bookworm .

build-pcov:
	$(DOCKER) build -f docker/bookworm.Dockerfile \
		--build-arg PHP_TAG=8.2-cli-bookworm \
		--build-arg PHP_DIGEST=$$(awk -F '\t' '$$1 == "8.2" { print $$2 }' docker/images.tsv) \
		--build-arg WITH_PCOV=1 \
		-t rich-editor-auy-php:8.2-pcov .

build-alpine:
	$(DOCKER) build -f docker/alpine.Dockerfile \
		--build-arg PHP_TAG=$(ALPINE_VERSION)-cli-alpine \
		--build-arg PHP_DIGEST=$$(awk -F '\t' '$$1 == "$(ALPINE_VERSION)" { print $$3 }' docker/images.tsv) \
		-t rich-editor-auy-php:$(ALPINE_VERSION)-alpine .

build: $(addprefix build-,$(VERSIONS)) build-pcov build-alpine

install: build-8.2
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer install --no-interaction --prefer-dist --no-progress

update: build-8.2
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer update --no-interaction --prefer-dist --no-progress

check: install
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer check

check-matrix: build
	@for version in $(VERSIONS); do \
		$(DOCKER_RUN) rich-editor-auy-php:$$version-bookworm \
			sh -c 'composer install --no-interaction --prefer-dist --no-progress && composer check' || exit 1; \
	done
	$(MAKE) coverage
	$(MAKE) check-alpine

check-alpine: build-alpine install
	$(DOCKER_RUN) rich-editor-auy-php:$(ALPINE_VERSION)-alpine \
		sh -c 'composer install --no-interaction --prefer-dist --no-progress && composer check'

coverage: build-pcov install
	$(DOCKER_RUN) rich-editor-auy-php:8.2-pcov composer coverage

smoke: install
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer smoke

fmt: install
	$(DOCKER_RUN) rich-editor-auy-php:8.2-bookworm composer fmt
```

Adicionar `help`, `shell`, `exec` (`cmd='php -v'`), `clean` (`rm -rf vendor build .phpunit.cache`) e
`.PHONY` cobrindo todos os alvos. `help` usa o padrão `grep -hE '^[a-zA-Z0-9_-]+:.*?## '`.

5. Gerar o lock antes de rodar testes: `make -C packages/php update` (imagem 8.2, plataforma 8.2.0).
   O lock resultante é versionado (passo 1.3). Se o Packagist limitar, repetir; não remover pins.

### 7. Ligação no repositório

1. `Makefile` (raiz):

```make
php:
	$(MAKE) -C packages/php check-matrix
```

e, em `check-ecosystem`, trocar o bloco do PHP por:

```make
	@if [ -f packages/php/composer.json ]; then \
		if command -v composer >/dev/null 2>&1; then \
			cd packages/php && composer install --no-interaction --prefer-dist --no-progress && composer check && composer audit; \
		elif command -v docker >/dev/null 2>&1; then \
			$(MAKE) php; \
		else \
			echo "composer and docker unavailable; PHP gate skipped locally"; \
		fi; \
	fi
```

2. `.github/workflows/ci.yml`: substituir o job `php` por uma matriz e adicionar o job musl:

```yaml
  php:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        php: ["8.2", "8.3", "8.4", "8.5"]
    steps:
      - uses: actions/checkout@v4
      - uses: shivammathur/setup-php@v2
        with:
          php-version: ${{ matrix.php }}
          tools: composer
          coverage: ${{ matrix.php == '8.2' && 'pcov' || 'none' }}
      - run: cd packages/php && composer install --no-interaction --prefer-dist --no-progress
      - run: cd packages/php && composer check
      - if: matrix.php == '8.2'
        run: cd packages/php && composer audit
      - if: matrix.php == '8.2'
        run: cd packages/php && composer coverage
      - if: matrix.php == '8.2'
        run: cd packages/php && composer smoke

  php-alpine:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: make -C packages/php check-alpine
```

3. `plan/09_frontend-e-referencias.md`: trocar as três ocorrências de versão para 8.2+
   (tabela §2: `PHP 8.2+`; §4.1: `php >=8.2`; §4.1: matriz `PHP 8.2/8.3/8.4/8.5`) e acrescentar em
   §4.1 a frase: “A execução do CLI é injetável via `ProcessRunner`; o pacote não usa shell,
   arquivos temporários nem `sys_get_temp_dir`.”

4. `AGENTS.md` §7: substituir a linha
   `cd packages/php && composer validate --strict && composer audit` por
   `make php` com o comentário `# validate + Mago + PHPStan + PHPUnit + cobertura + smoke`.

## Arquivos críticos e âncoras

- `packages/php/src/Envelope.php` — construtor privado + `fromArray`; as quatro regras do envelope e
  a regex `\A[\x21-\x7e]{1,128}\z` (a atual aceita `\n` por usar `$`).
- `packages/php/src/Process/SystemProcessRunner.php` — único ponto com `proc_open`/`stream_select`;
  deadline `hrtime`, SIGKILL no timeout, EPIPE tratado como fim de stdin.
- `packages/php/src/ValidationReport.php` — as três famílias de linha do CLI e o parsing tipado.
- `packages/php/Makefile` + `packages/php/docker/images.tsv` — imagens fixadas por digest e o alvo
  único `check-matrix`.
- `.github/workflows/ci.yml` (job `php`) — matriz 8.2–8.5, cobertura/smoke na base 8.2 e job musl.

## Verificação

Pré-requisitos: Docker com rede para `docker.io` e Packagist; não é necessário `php`/`composer` no
host. Todos os comandos abaixo são executados na raiz do repositório.

1. `make -C packages/php update` → gera `packages/php/composer.lock`; `test -s
   packages/php/composer.lock`.
2. `make -C packages/php check-matrix` → constrói as imagens 8.2/8.3/8.4/8.5 bookworm, 8.2 alpine e
   8.2-pcov; cada versão imprime `./composer.json is valid`, Mago sem ocorrências, `[OK] No errors`
   do PHPStan e `OK (...)` do PHPUnit; ao final, `coverage-check: OK` e `consumer-smoke: OK`.
3. `make -C packages/php coverage` → `coverage: N/N statements (100.00%), required 100%` e
   `coverage-check: OK`.
4. `make -C packages/php smoke` → `consumer-smoke: OK` (instalação `--no-dev` em projeto externo).
5. `make php` → o mesmo gate completo via `check-matrix`, sem erro.
6. Prova de comportamento (container 8.2 bookworm, fonte montada):
   - `doc_id` com newline:
     ```sh
     docker run --rm -u "$(id -u):$(id -g)" -e HOME=/tmp \
       -v "$PWD/packages/php:/app" -w /app rich-editor-auy-php:8.2-bookworm \
       php -r 'require "vendor/autoload.php";
       $json = "{\"format\":\"rich-editor-auy\",\"version\":2,\"schema_fingerprint\":\"" . str_repeat("a",64) . "\",\"doc_id\":\"post-1\\n\",\"doc\":{\"type\":\"doc\",\"content\":[]}}";
       try { RichEditorAuy\Document::fromBytes($json); echo "accepted\n"; }
       catch (RichEditorAuy\Exception\IncompatibleEnvelope $e) { echo "rejected: ", $e->getMessage(), "\n"; }'
     ```
     Saída esperada: `rejected: doc_id must be 1-128 graphic ASCII characters`.
   - erro de parse classificado:
     ```sh
     docker run --rm -u "$(id -u):$(id -g)" -e HOME=/tmp \
       -v "$PWD/packages/php:/app" -w /app rich-editor-auy-php:8.2-bookworm \
       php -r 'require "vendor/autoload.php";
       $r = RichEditorAuy\ValidationReport::fromProcess(1, "", "error: missing_format at offset 0\n");
       echo json_encode([$r->ok(), array_map(fn($i) => [$i->code, $i->offset, $i->path], $r->issues), $r->diagnostics, $r->unrecognized]), "\n";'
     ```
     Saída esperada: `[false,[["missing_format",0,""]],[],[]]`.
7. `make check` (gate Rust existente) → verde, sem regressão; `make check-ecosystem` → Deno (inalterado),
   PHP `composer check` + `composer audit` verdes.

## Premissas e contingências

- **Rede/Docker Hub**: as imagens estão fixadas por digest (índice) e o `composer` por digest de
  manifesto `linux/amd64`. Se houver `429 Too Many Requests`, aguardar a janela e repetir; nunca
  trocar digest por tag flutuante.
- **`COPY --from` com digest de índice**: se o BuildKit recusar o digest do Composer no
  `COPY --from`, usar o manifesto amd64 verificado
  `composer:2@sha256:07cb23c4c9d528524a8b9a67c403d6ad7fef2268651eb1448e3874d40f28b789`; não usar
  `composer:2` sem digest.
- **Mago baixa o binário no primeiro uso**: `composer check`/`lint` exigem rede. Se o download
  falhar, o gate falha explicitamente; não pular o lint.
- **PHPUnit 11.5.57 em PHP 8.5**: versão publicada em 2026-10-05 e sem teto de PHP, esperada verde
  em 8.5. Se o PHPUnit falhar em 8.5 **apenas** por deprecação interna do framework, manter a matriz
  8.2/8.3/8.4 com `composer check` completo e, no job 8.5, rodar `composer validate --strict`,
  `composer static`, `composer smoke` e um teste de runtime; registrar a limitação em
  `plan/09` §4.1. Não usar `failOnDeprecation=false` global.
- **Cobertura 100% com caminhos de ambiente**: as únicas linhas excluídas são as cinco regiões
  marcadas com `@codeCoverageIgnore` em `SystemProcessRunner` (falha de `proc_open`/`select` e
  supressão de warning de EPIPE), cada uma impossível de disparar deterministicamente sem
  reconfigurar o processo PHP. Se o PCOV não honrar as anotações, substituir por `@codeCoverageIgnore`
  inline nas linhas de `throw`/`return`; não baixar o limite de 100%.
- **CLI future**: qualquer linha de `stderr` que não case com as duas gramáticas vai para
  `unrecognized`; o pacote não falha por formato desconhecido e não inventa issues. Não alterar o
  CLI neste trabalho.
