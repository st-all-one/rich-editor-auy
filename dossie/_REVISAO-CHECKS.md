# Checklist da rodada extra de revisão

O revisor é independente do explorador. Ele **corrige** o dossiê; não se limita a opinar.

## Comum a todos

1. Ler `dossie/_GUIA.md`, `dossie/_RUBRICA.md`, o dossiê inteiro e a fonte em `ref/<projeto>/`.
2. Verificar se cada afirmação central tem `caminho:linha`, teste, exemplo ou saída de comando.
3. Conferir se `codigo/INDEX.md` aponta para arquivos que existem, se a licença está correta e se
   os trechos são realmente representativos.
4. Conferir `exemplos/README.md`: comando, saída observada, o que não foi executado e por quê.
5. Auditar a matriz de 85%: instalação, init, edição, seleção/IME/undo/clipboard, schema,
   extensão, persistência, CSS, a11y, colaboração, performance, erros, licença.
6. Procurar o que o explorador não viu: API real, defaults, casos de borda, custo de integração,
   limitações, dependências ocultas, CSS global, acessibilidade real.
7. Corrigir números, caminhos, exemplos e conclusões erradas. Se não der para verificar, marcar
   `não verificado` e baixar a confiança; não preencher com plausibilidade.
8. Preencher `revisao.md` com: achados, correções feitas, confiança final (0–100%), lacunas.
9. Não alterar `ref/`, `plan/`, `AGENTS.md` nem outros dossiês.

## Perguntas específicas

- **lexical**: separe core de React/playground; confirme se o CSS é do playground ou do pacote;
  verifique `EditorState`/node tree/reconciliação; o exemplo mínimo é realmente vanilla?
- **prosemirror**: a cópia local é um stub; confirme que o dossiê usou `/tmp/pm-upstream` ou outra
  fonte oficial e citou versões. Verifique schema/transform/state/view; não venda conveniência.
- **quill**: confirme Delta como fonte de verdade, o papel dos blots e onde `snow`/`bubble`
  injetam CSS; separe dado de DOM.
- **summernote**: confirme que HTML/contenteditable é a fonte de verdade, o acoplamento a
  jQuery/Bootstrap, o uso de execCommand/range e o custo de a11y.
- **tiptap**: confirme o que é “headless”, se há CSS no pacote publicado, o que vem de
  ProseMirror e o custo de React/Vue.
- **taino-edit**: confirme modelo Rust, ownership/steps, renderer Leptos, zero CSS e maturidade;
  não copie a API sem entender o acoplamento ao Leptos.
- **html-auy**: confirme que **não é editor**; avalie valor para render/escape/a11y e não invente
  comparação de edição.
- **knudge**: confirme que **não é editor**; avalie processo, gates, 300 linhas, testes e
  documentação; não invente comparação de edição.
- **marktext**: separe app Electron de engine Muya; confirme Markdown como fonte de verdade,
  temas/CSS e o que é reutilizável.
- **Zettlr**: GPL — garanta trechos pequenos e aviso de não reutilização; confirme o serviço de
  links/IDs, citações, CodeMirror e o que é UX de app, não biblioteca.
