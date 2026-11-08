// Probe 3 — micro-medição de custo. NÃO é o benchmark oficial do projeto
// (packages/lexical/src/__bench__, que roda em vitest e não foi executado).
//
// Dependência: lexical@0.53.0. Node 22, sem DOM.
import {$createParagraphNode, $createTextNode, $getRoot, createEditor} from 'lexical';

const mk = n => {
  const ed = createEditor({namespace: 'perf'});
  ed.update(() => {
    const ps = [];
    for (let i = 0; i < n; i++) {
      ps.push($createParagraphNode().append($createTextNode('paragraph ' + i + ' lorem ipsum dolor sit amet')));
    }
    $getRoot().append(...ps);
  }, {discrete: true});
  return ed;
};

console.log('n | build | toJSON | stringify | jsonBytes');
for (const n of [100, 1000, 5000]) {
  const t0 = performance.now();
  const ed = mk(n);
  const t1 = performance.now();
  const t2 = performance.now();
  const json = ed.getEditorState().toJSON();
  const t3 = performance.now();
  const s = JSON.stringify(json);
  const t4 = performance.now();
  console.log(
    `${n} | ${(t1 - t0).toFixed(1)}ms | ${(t3 - t2).toFixed(1)}ms | ${(t4 - t3).toFixed(1)}ms | ${s.length}`,
  );
}

const ed = mk(1000);
const t0 = performance.now();
for (let i = 0; i < 200; i++) {
  ed.update(() => {
    const p = $getRoot().getChildAtIndex(i % 1000);
    if (p) {
      const t = p.getFirstChild();
      if (t) t.setTextContent('x' + i);
    }
  }, {discrete: true});
}
const t1 = performance.now();
console.log(
  `200 updates discretos em 1000 parágrafos: ${(t1 - t0).toFixed(1)}ms total (${((t1 - t0) / 200).toFixed(2)}ms cada)`,
);
