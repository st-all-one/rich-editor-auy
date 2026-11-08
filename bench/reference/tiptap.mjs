import {Window} from 'happy-dom';
const window = new Window({url:'https://example.test'});
globalThis.window=window; globalThis.document=window.document; globalThis.MutationObserver=window.MutationObserver; globalThis.HTMLElement=window.HTMLElement; globalThis.Node=window.Node; globalThis.Text=window.Text; globalThis.Element=window.Element;
const {Editor} = await import('@tiptap/core');
const {default: StarterKit} = await import('@tiptap/starter-kit');
const text=(t,attrs)=>({type:'text',text:t,...(attrs?{marks:[{type:'link',attrs}]}:{})});
const rich=[{type:'text',text:'Texto '},{type:'text',text:'forte',marks:[{type:'bold'}]},{type:'text',text:' e '},{type:'text',text:'ênfase',marks:[{type:'italic'}]},{type:'text',text:' com '},{type:'text',text:'código',marks:[{type:'code'}]},{type:'text',text:' e '},text('link',{href:'https://example.test/a'}),{type:'text',text:'.'}];
const content=[];
const N=Number(process.env.N??500);
for(let i=0;i<N;i++){
 content.push({type:'heading',attrs:{level:2},content:[{type:'text',text:'Título '},{type:'text',text:'rico',marks:[{type:'bold'}]}]});
 content.push({type:'paragraph',content:rich});
 for(let j=0;j<2;j++) content.push({type:'bulletList',content:[{type:'listItem',content:[{type:'paragraph',content:[{type:'text',text:'Item de lista'}]}]}]});
 content.push({type:'heading',attrs:{level:3},content:[{type:'text',text:'Subseção'}]});
 content.push({type:'paragraph',content:[{type:'text',text:'Parágrafo longo para medir texto e marcas. '},{type:'text',text:'negrito',marks:[{type:'bold'}]},{type:'text',text:' normal '},{type:'text',text:'itálico',marks:[{type:'italic'}]}]});
}
const doc={type:'doc',content};
const element=document.createElement('div'); document.body.appendChild(element);
const editor=new Editor({element,extensions:[StarterKit],content:doc});
const measure=(label,fn,n=10)=>{for(let i=0;i<2;i++)fn(); const t=performance.now(); for(let i=0;i<n;i++)fn(); console.log(label,((performance.now()-t)/n).toFixed(3),'ms');};
measure('Tiptap setContent',()=>editor.commands.setContent(doc),10);
measure('Tiptap getJSON',()=>editor.getJSON(),10);
measure('Tiptap insertContentAt',()=>editor.commands.insertContentAt(2,'x'),10);
console.log('Tiptap blocks',editor.getJSON().content.length,'json',JSON.stringify(editor.getJSON()).length);
