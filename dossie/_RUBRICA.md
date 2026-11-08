# Rubrica de avaliação (0–5)

Use os mesmos âncoras em todos os dossiês. A nota é sobre **o projeto como editor de texto rico
para o nosso caso**, não sobre popularidade. Toda nota precisa de evidência.

| Dimensão | 0 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| Usabilidade de integração | impossível sem reescrever | muitos passos e configuração | funciona, com atrito | init simples e API clara | init mínimo, erros claros, ótima DX |
| API | inexistente/instável | baixo nível, pouca docs | utilizável | bem tipada e documentada | pequena, tipada, difícil de usar errado |
| Neutralidade de estilo | exige CSS/tema | traz tema pesado | permite trocar, mas interfere | pouco CSS e classes | nenhum CSS/estrutura imposta |
| Modelo de dados | DOM/string sem semântica | árvore frouxa | modelo claro com lacunas | árvore/Delta tipado, bom mapeamento | modelo canônico, normalizado, validável |
| Extensibilidade | fork | plugins limitados | plugins razoáveis | nodes/marks/plugins | extensão de primeira classe, sem tocar no core |
| Performance | inviável em docs grandes | lento/patológico | aceitável | bom com tuning | previsível, benchmarks, sem cópias ocultas |
| Acessibilidade | inacessível | ARIA mínimo | contenteditable básico | boa base, faltam casos | WCAG-aware, teclado/leitor/IME documentados |
| Documentação | ausente | superficial | cobre o básico | boa e atualizada | profunda, exemplos, migração e limites |
| Manutenção | morto | raro | ativo | ativo e testado | governança, CI, releases, segurança |
| Adequação ao `rich-editor-auy` | não serve | referência indireta | adaptável com custo | boa referência | quase encaixa (headless, Rust, zero CSS) |

## Como calcular a confiança

- **90–100%**: código, testes e exemplo executado/observado; lacunas menores.
- **75–89%**: código e docs fortes; execução parcial ou ausente em um eixo.
- **60–74%**: boa leitura estática; usabilidade inferida de docs/testes.
- **<60%**: só README/marketing ou fonte incompleta.

A meta do usuário é >=85% de **usabilidade/resultado**, não de linhas lidas. Se um eixo não puder
ser verificado, a confiança cai e o dossiê deve dizer exatamente o que falta.
