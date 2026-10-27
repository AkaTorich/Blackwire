'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.resolve(__dirname, '..');
const context = vm.createContext({});
vm.runInContext(fs.readFileSync(path.join(root, 'app/ui/js/apps/editor_highlight.js'), 'utf8')+'\nthis.api=EditorHighlight;', context);
const h = context.api;
function text(html) { return html.replace(/<[^>]+>/g, '').replace(/&(amp|lt|gt);/g, (_, key)=>({amp:'&',lt:'<',gt:'>'}[key])); }

test('Adder functions, APIs, numbers, constants, strings and comments have separate tokens', ()=>{
  const source='def sample(value):\n    return net . research ("узел", 0xFF) if true else null # комментарий\n';
  const html=h.highlight(source,'sample.adr'); assert.equal(text(html),source);
  for (const cls of ['tk-kw','tk-def','tk-module','tk-call','tk-str','tk-num','tk-const','tk-comment']) assert.ok(html.includes('class="'+cls+'"'),cls);
  assert.ok(html.includes('class="tk-call">research</span>'));
});
test('f-strings preserve escaped braces, quoted braces and nested expressions', ()=>{
  const samples=[`f"Значение: {net.research('}')[0] + 12:>6} {{готово}}"`, `f'''Строка\n{json.dumps({"x": "}"})}\n'''`, `f"{f'{1+2}'}"`];
  for (const source of samples){const html=h.highlight(source,'x.adr');assert.equal(text(html),source);assert.ok(html.includes('tk-num')||html.includes('tk-module'));}
});
test('multiline strings and HTML-like source remain literal text',()=>{
  const source='text = """<img src=x onerror=bad()>\n# не комментарий\n&amp;"""\n';
  const html=h.highlight(source,'a.adr');assert.equal(text(html),source);assert.ok(!html.includes('<img'));assert.ok(!html.includes('tk-comment'));
});
test('JSON distinguishes keys, escaped strings, numbers and constants',()=>{
  const source='{"ключ":"a\\\"b","active":true,"count":-12.5e+2,"empty":null}\n';
  const html=h.highlight(source,'plugin.json');assert.equal(text(html),source);
  for(const cls of ['tk-key','tk-str','tk-num','tk-const']) assert.ok(html.includes(cls));
});
test('disabling syntax keeps search marks and never changes the text',()=>{
  const source='def тест(): return "<&>"';
  const html=h.highlight(source,'x.adr',[{start:4,end:8,cur:true}],false);
  assert.equal(text(html),source);assert.ok(html.includes('tk-found-cur'));assert.ok(!html.includes('tk-kw'));assert.ok(html.includes('>тест</span>'));
});
test('plain logs and Windows-style paths are supported',()=>{
  assert.equal(h.langFor('C:\\scripts\\MAIN.ADR'),'adder');assert.equal(h.langFor('plugin.json'),'json');
  const source='[service]\nport=7002\nERROR: отказ\n';assert.equal(text(h.highlight(source,'server.log')),source);
});
test('highlighting actual SDK and new plugin scripts round-trips without modifying source',()=>{
  const folders=['sdk/examples','plugins/vulnerability_cases'];let count=0;
  function visit(dir){for(const entry of fs.readdirSync(dir,{withFileTypes:true})){const p=path.join(dir,entry.name);if(entry.isDirectory())visit(p);else if(p.endsWith('.adr')){const source=fs.readFileSync(p,'utf8');assert.equal(text(h.highlight(source,p)),source,p);count++;}}}
  folders.forEach(p=>visit(path.join(root,p)));assert.ok(count>=70,count);
});
