---
id: fact_01e7uvl1
type: fact
statement: A seleção sobrevive a um round-trip de serialização
created_at: 2026-11-08T01:18:56.807Z
body_hash: c015c6e4
schema_version: 3
tags: [selecao]
anchors: [src/selection.rs, src/serializer.rs]
---
Por quê: offsets de seleção podem apontar para o meio de um grafo de nós se a serialização reordenar ou colapsar nós vazios.
Evidência: proptest de round-trip com seleção não vazia.
Consequência: todo nó precisa de chave estável para a seleção ser endereçável.
