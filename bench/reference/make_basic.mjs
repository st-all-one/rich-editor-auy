import {writeFileSync} from 'node:fs';
const text=(text,marks)=>({type:'text',text,...(marks?{marks}:{})});
const rich=[text('Texto '),text('forte',[{type:'strong'}]),text(' e '),text('ênfase',[{type:'em'}]),text(' com '),text('código',[{type:'code'}]),text(' e '),text('link',[{type:'link','href':'https://example.test/a'}]),text('.')];
const ref=[text('Ver '),text('referência',[{type:'ref',target:{doc:'pub-2',node:'node-7',version:'v3'}}]),text('.')];
const item=()=>({type:'list_item',content:[{type:'paragraph',content:[text('Item de lista')]}]});
const blocks=[
 {type:'heading',attrs:{level:2},content:[text('Título '),text('rico',[{type:'strong'}])]},
 {type:'paragraph',content:rich},
 {type:'bullet_list',content:[item(),item()]},
 {type:'paragraph',content:ref},
 {type:'heading',attrs:{level:3},content:[text('Subseção')]},
 {type:'paragraph',content:[text('Parágrafo longo para medir texto e marcas. '),text('negrito',[{type:'strong'}]),text(' normal '),text('itálico',[{type:'em'}])]},
];
const doc={type:'doc',attrs:{lang:'pt-BR',dir:'ltr'},content:blocks};
const big={...doc,content:blocks.flatMap((block)=>Array.from({length:500},()=>block))};
writeFileSync('/tmp/complex.json',JSON.stringify(big));
const noRef=JSON.parse(JSON.stringify(big));
const strip=(node)=>{if(Array.isArray(node)){node.forEach(strip);}else if(node&&typeof node==='object'){if(node.marks)node.marks=node.marks.filter((mark)=>mark.type!=='ref');if(node.marks&&node.marks.length===0)delete node.marks;Object.values(node).forEach(strip);}};
strip(noRef);
writeFileSync('/tmp/complex-taino.json',JSON.stringify(noRef));
