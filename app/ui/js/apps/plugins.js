'use strict';
(function(){
  function mount(body){
    body.style.padding='18px';body.style.overflow='auto';
    var title=document.createElement('h2');title.textContent='Плагины Adder';body.appendChild(title);
    var toolbar=document.createElement('div');toolbar.style.cssText='display:flex;gap:10px;flex-wrap:wrap;margin-bottom:18px';body.appendChild(toolbar);
    var path=document.createElement('p');path.style.cssText='overflow-wrap:anywhere;opacity:.7';body.appendChild(path);
    var list=document.createElement('div');body.appendChild(list);
    var log=document.createElement('pre');log.style.cssText='white-space:pre-wrap;overflow-wrap:anywhere;padding:12px';body.appendChild(log);
    function error(e){log.textContent=String(e);}
    function button(parent,label,action){var b=document.createElement('button');b.textContent=label;b.style.cssText='padding:8px 12px;margin:4px;cursor:pointer';b.onclick=function(){b.disabled=true;Promise.resolve().then(action).catch(error).finally(function(){b.disabled=false;});};parent.appendChild(b);return b;}
    function render(info){
      list.replaceChildren();if(info.folder)path.textContent=info.folder;
      (info.errors||[]).forEach(function(e){var p=document.createElement('p');p.textContent=e;p.style.color='var(--danger, #f88)';list.appendChild(p);});
      if(!info.plugins.length){var empty=document.createElement('p');empty.textContent='Плагинов пока нет. Добавьте папку плагина или установите пример.';list.appendChild(empty);}
      info.plugins.forEach(function(p){
        var card=document.createElement('section');card.style.cssText='padding:16px;margin:12px 0;border:1px solid currentColor;border-radius:8px';list.appendChild(card);
        var h=document.createElement('h3');h.textContent=p.name+' · '+p.version;card.appendChild(h);
        var status=document.createElement('p');status.textContent=p.error?'Ошибка: '+p.error:(p.enabled?'Включён':'Выключен');card.appendChild(status);
        var perm=document.createElement('p');perm.textContent='Доступ: '+(p.permissions.join(', ')||'данные плагина');card.appendChild(perm);
        button(card,p.enabled?'Выключить':'Включить',function(){return BW.plugins_enable(p.id,!p.enabled).then(render);});
        if(p.error)button(card,'Повторить запуск',function(){return BW.plugins_enable(p.id,true).then(render);});
        (p.windows||[]).forEach(function(w){button(card,w.title,function(){return Apps.openOrFocus('plugin_panel',w.id);});});
        p.commands.forEach(function(c){var row=document.createElement('div');var input=document.createElement('input');input.placeholder='Аргументы';input.style.padding='8px';row.appendChild(input);button(row,c,function(){return BW.plugins_run(c+(input.value?' '+input.value:'')).then(function(lines){log.textContent=lines.map(function(l){return l.text;}).join('\n');return BW.plugins_list().then(render);});});card.appendChild(row);});
      });
    }
    button(toolbar,'Открыть папку',function(){return BW.plugins_folder();});
    button(toolbar,'Перечитать',function(){return BW.plugins_reload().then(render);});
    button(toolbar,'Добавить пример устройств',function(){return BW.plugins_example('device_lab').then(render);});
    button(toolbar,'Добавить пример района',function(){return BW.plugins_example('district').then(render);});
    BW.plugins_list().then(render).catch(error);
  }
  Apps.register('plugins',{title:'Плагины',icon:'plugins',minWidth:520,minHeight:320,mount:mount});
})();
