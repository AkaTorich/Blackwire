'use strict';
(function(){
 function mount(body,id){
  body.style.cssText='padding:20px;overflow:auto';var content=document.createElement('div'),log=document.createElement('pre');log.style.cssText='white-space:pre-wrap;overflow-wrap:anywhere';body.appendChild(content);body.appendChild(log);
  function fail(e){log.textContent=String(e);}
  function refresh(){return BW.plugins_panel(id).then(function(spec){
   content.replaceChildren();var h=document.createElement('h2');h.textContent=spec.title;content.appendChild(h);
   var p=document.createElement('p');p.textContent=spec.text||'';p.style.whiteSpace='pre-wrap';content.appendChild(p);
   var values={};(spec.fields||[]).forEach(function(f){var row=document.createElement('label');row.style.cssText='display:block;margin:12px 0';row.appendChild(document.createTextNode(f.label+' '));var input=document.createElement('input');input.type=['number','checkbox','text'].indexOf(f.type)>=0?f.type:'text';input.style.padding='8px';input.value=f.value==null?'':f.value;if(input.type==='checkbox')input.checked=!!f.value;row.appendChild(input);values[f.id]=input;content.appendChild(row);});
   if(spec.rows&&spec.rows.length){var table=document.createElement('table');table.style.cssText='width:100%;border-collapse:collapse;margin:12px 0';spec.rows.forEach(function(row){var tr=document.createElement('tr');(Array.isArray(row)?row:[row]).forEach(function(v){var td=document.createElement('td');td.textContent=String(v);td.style.cssText='padding:8px;border-bottom:1px solid #777';tr.appendChild(td);});table.appendChild(tr);});content.appendChild(table);}
   (spec.buttons||[]).forEach(function(b){var btn=document.createElement('button');btn.textContent=b.label;btn.style.cssText='padding:10px 16px;margin:6px;cursor:pointer';btn.onclick=function(){btn.disabled=true;var data={};Object.keys(values).forEach(function(k){var v=values[k];data[k]=v.type==='checkbox'?v.checked:(v.type==='number'?Number(v.value):v.value);});BW.plugins_ui(id,b.id,data).then(function(lines){log.textContent=lines.map(function(l){return l.text;}).join('\n');return refresh();}).catch(fail).finally(function(){btn.disabled=false;});};content.appendChild(btn);});
  }).catch(fail);}
  refresh();
 }
 Apps.register('plugin_panel',{title:'Приложение плагина',icon:'plugins',minWidth:520,minHeight:320,mount:mount});
})();
