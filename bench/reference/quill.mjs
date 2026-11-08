import {Window} from 'happy-dom';
const window = new Window({url:'https://example.test'});
globalThis.window=window; globalThis.document=window.document; globalThis.MutationObserver=window.MutationObserver; globalThis.HTMLElement=window.HTMLElement; globalThis.Node=window.Node; globalThis.Text=window.Text; globalThis.Element=window.Element; globalThis.getSelection=window.getSelection.bind(window);
const {default: Quill} = await import('quill');
const el=document.createElement('div'); document.body.appendChild(el);
const quill=new Quill(el,{theme:false, modules:{toolbar:false}});
const ops=[];
const text=(t,attrs)=>({insert:t,...(attrs?{attributes:attrs}:{})});
const N=Number(process.env.N??500);
for(let i=0;i<N;i++){
  ops.push(text('Título ',{header:2,bold:true}),text('rico'),{insert:'\n',attributes:{header:2}});
  ops.push(text('Texto '),text('forte',{bold:true}),text(' e '),text('ênfase',{italic:true}),text(' com '),text('código',{code:true}),text(' e '),text('link',{link:'https://example.test/a'}),text('.'),{insert:'\n'});
  for(let j=0;j<2;j++){ops.push(text('Item de lista',{list:'bullet'}),{insert:'\n',attributes:{list:'bullet'}});}
  ops.push(text('Subseção',{header:3}),{insert:'\n',attributes:{header:3}});
  ops.push(text('Parágrafo longo para medir texto e marcas. '),text('negrito',{bold:true}),text(' normal '),text('itálico',{italic:true}),{insert:'\n'});
}
const delta={ops};
const measure=(label,fn,n=20)=>{for(let i=0;i<3;i++)fn(); const t=performance.now(); for(let i=0;i<n;i++)fn(); console.log(label,((performance.now()-t)/n).toFixed(3),'ms');};
measure('Quill setContents',()=>quill.setContents(delta),20);
measure('Quill getContents',()=>quill.getContents(),20);
measure('Quill updateContents insert',()=>quill.updateContents([{retain:1},{insert:'x'}]),20);
console.log('Quill ops',delta.ops.length,'getContents length',quill.getContents().length());
