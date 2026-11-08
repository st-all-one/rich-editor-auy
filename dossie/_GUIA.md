# Guia dos dossiês de referência

Este diretório é uma coleção de **estudos de caso verificáveis**, não de resumos de marketing.
Cada projeto em `ref/` recebe uma pasta com o mesmo nome, e cada afirmação relevante deve apontar
para um caminho de arquivo e, quando possível, linhas.

## Regra de ouro

O dossiê deve responder, com evidência:

1. **Para que serve** e qual problema resolve.
2. **Como pensa** (modelo mental, arquitetura, tradeoffs).
3. **Como é de verdade** (código, API pública, exemplos, testes, comportamento observado).
4. **O que o usuário vê** ao integrar: instalação, primeira edição, formatação, extensão,
   persistência, estilo, erro e depuração.
5. **O que ganha e o que perde** como editor de texto rico, inclusive quando não é um editor.
6. **O que devemos copiar, adaptar ou evitar** no `rich-editor-auy`.

## Estrutura obrigatória por projeto

```
dossie/<projeto>/
  README.md          # visão geral, propósito, resultado, uso, prós/contras
  arquitetura.md     # modelo, internals, extensibilidade, rendering, a11y, performance
  avaliacao.md       # rubrica, evidências, cobertura de usabilidade (>=85%), limites
  codigo/
    INDEX.md         # origem -> cópia, por que foi copiado, licença
    *.ts|*.js|*.rs|*.md
  exemplos/
    README.md        # como rodar; comandos e saída observada; o que não foi possível rodar
    *.*
  revisao.md         # preenchido na rodada extra de revisão
```

`codigo/` deve conter **código real copiado**, não pseudocódigo. Prefira 2–6 arquivos ou trechos
substantivos que expliquem o coração do projeto: modelo de documento, transação, schema,
renderer, parser/serializer, plugin, seleção ou comando. Preserve cabeçalhos de licença e
identifique a origem. Para projetos GPL, copie apenas trechos pequenos para análise e deixe claro
que **nada será reutilizado**; para MIT/Apache/BSD, arquivos completos são permitidos com aviso.

`exemplos/` deve conter um exemplo mínimo **real** (código de inicialização, schema, comando,
persistência) e o comando para executá-lo. Se as dependências não estiverem instaladas, não invente
saída: registre `não executado` e explique como verificar. Se for barato rodar sem instalar, rode e
cole a saída.

## Cobertura de usabilidade (meta >= 85%)

A pontuação não é "quantas linhas li". É uma matriz de evidências. Marque cada item como
`verificado no código`, `verificado em teste`, `verificado em exemplo`, `verificado em doc` ou
`não verificado`:

- instalação e pré-requisitos;
- inicialização mínima;
- edição básica (texto, parágrafo, negrito/itálico/link);
- seleção, teclado, undo/redo, clipboard, IME;
- schema e restrições de conteúdo;
- custom node/mark/plugin;
- persistência e formato de serialização;
- renderização e possibilidade de não interferir no CSS;
- acessibilidade (ARIA, foco, leitor de tela, conteúdo editável);
- colaboração (se existir);
- performance e limites conhecidos;
- erros, migração e diagnóstico;
- licença e maturidade.

O `avaliacao.md` deve ter uma tabela com nota 0–5, evidência e comentário para:
**usabilidade, API, neutralidade de estilo, modelo de dados, extensibilidade, performance,
acessibilidade, documentação, manutenção e adequação ao nosso caso**. A nota sem evidência é
inválida.

## O que evitar

- Repetir o README oficial.
- Dizer "é rápido", "é flexível" ou "é fácil" sem um caminho de código, teste ou exemplo.
- Confundir o produto (app) com a biblioteca.
- Ignorar o custo de DOM/contenteditable, o modelo de dados e a serialização.
- Copiar código sem indicar licença e origem.
- Fingir que rodou algo que não rodou.

## Rodada extra de revisão

Cada pasta passa por um revisor independente. O revisor deve:

1. Reexecutar ou contestar as afirmações centrais.
2. Procurar o que o dossiê não viu: API real, casos de borda, limitações, custo de integração.
3. Corrigir números, caminhos e exemplos.
4. Preencher `revisao.md` com: lacunas encontradas, correções feitas, confiança final (0–100%) e
   o que ainda não foi verificado.
5. Só considerar 85% quando a matriz de usabilidade estiver majoritariamente evidenciada, não
   quando o texto estiver bonito.
