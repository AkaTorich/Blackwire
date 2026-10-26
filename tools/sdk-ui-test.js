// Test-only integration runner. Injected only with --sdk-ui-test and --data-dir.
(async function(){
 var stage=BW.app==='plugins'?'manager':'panel',checks=[];
 function assert(value,message){if(!value)throw Error(message);checks.push(message);}
 async function wait(test){for(var i=0;i<300;i++){if(await test())return;await new Promise(function(r){setTimeout(r,100);});}throw Error('Тайм-аут проверки UI');}
 function button(label){return Array.from(document.querySelectorAll('button')).find(function(b){return b.textContent===label;});}
 try{
  await wait(function(){return window.BW&&document.querySelector('.win-body');});
  if(stage==='manager'){
   await wait(function(){return button('district_create');});
   var info=await BW.plugins_list();assert(info.plugins.length===2&&info.plugins.every(function(p){return !p.error;}),'Пакеты загружены без ошибок');
   button('district_create').click();await wait(function(){return document.querySelector('pre').textContent.includes('Создан район 1');});
   var panel=await BW.plugins_panel('district:control');assert(panel.rows[0][1]===1,'Кнопка команды создаёт район');
   var district=Array.from(document.querySelectorAll('section')).find(function(s){return s.querySelector('h3').textContent.includes('Линия-7');});
   Array.from(district.querySelectorAll('button')).find(function(b){return b.textContent==='Выключить';}).click();
   await wait(async function(){return !(await BW.plugins_list()).plugins.find(function(p){return p.id==='district';}).enabled;});assert(true,'Выключение через карточку');
   await wait(function(){return Array.from(document.querySelectorAll('section')).some(function(s){return s.querySelector('h3').textContent.includes('Линия-7')&&Array.from(s.querySelectorAll('button')).some(function(b){return b.textContent==='Включить';});});});
   district=Array.from(document.querySelectorAll('section')).find(function(s){return s.querySelector('h3').textContent.includes('Линия-7');});Array.from(district.querySelectorAll('button')).find(function(b){return b.textContent==='Включить';}).click();
   await wait(async function(){return (await BW.plugins_list()).plugins.find(function(p){return p.id==='district';}).enabled;});assert(true,'Включение через карточку');
   await wait(function(){return button('lab_create')&&!button('lab_create').disabled;});button('lab_create').click();await wait(function(){return document.querySelector('pre').textContent.includes('lab-camera-1');});assert(true,'Кнопка создаёт новый тип устройства');
   assert((await BW.plugins_panel('district:control')).rows[0][1]===1,'Повторная загрузка не дублирует контент');
  }else{
   await wait(async function(){return !!(await BW.invoke('sdk_ui_status',{})).manager;});
   await wait(function(){return button('Создать район');});button('Создать район').click();
   await wait(function(){return document.querySelector('pre').textContent.includes('Создан район 2');});
   assert((await BW.plugins_panel('district:control')).rows[0][1]===2,'Кнопка собственного окна вызывает Adder');
   assert(document.body.textContent.includes('Создано районов'),'Таблица окна отрисована');
   button('Проверить задания').click();await wait(function(){return document.querySelector('pre').textContent.includes('Архив района 2');});assert(true,'Повторная кнопка обновляет вывод');
  }
  await BW.invoke('sdk_ui_report',{stage:stage,result:{pass:true,checks:checks}});
 }catch(e){await BW.invoke('sdk_ui_report',{stage:stage,result:{pass:false,checks:checks,error:String(e)}});}
})();
