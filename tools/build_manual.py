from pathlib import Path
import json,html,re
p=Path(__file__).resolve().parents[1]
s=p/'sdk/schema'
objects={f.stem:__import__('json').loads(f.read_text(encoding='utf-8-sig')) for f in s.glob('*.json')}
# Basic Markdown renderer for a static, script-free CHM. All inline text is escaped first.
def inline(t):
 t=html.escape(t,quote=False)
 t=re.sub(r'`([^`]+)`',r'<code>\1</code>',t)
 t=re.sub(r'\*\*([^*]+)\*\*',r'<strong>\1</strong>',t)
 t=re.sub(r'\[([^]]+)\]\(([^)]+)\)',lambda m:m[1]+' ('+m[2]+')',t)
 return t

def render(md):
 out=[];lines=md.splitlines();i=0;para=[]
 def flush():
  if para:out.append('<p>'+inline(' '.join(para))+'</p>');para.clear()
 while i<len(lines):
  t=lines[i]
  if t.startswith('```'):
   flush();code=[];i+=1
   while i<len(lines) and not lines[i].startswith('```'):code.append(lines[i]);i+=1
   out.append('<pre>'+html.escape('\n'.join(code))+'</pre>')
  elif re.match(r'^#{1,6} ',t):
   flush();n=len(t)-len(t.lstrip('#'));out.append(f'<h{n}>'+inline(t[n+1:])+f'</h{n}>')
  elif t.startswith('|'):
   flush();rows=[]
   while i<len(lines) and lines[i].startswith('|'):
    cells=[c.strip() for c in lines[i].strip().strip('|').split('|')]
    if not all(re.fullmatch(r':?-+:?',c.replace(' ','')) for c in cells):rows.append(cells)
    i+=1
   out.append('<table>')
   for j,row in enumerate(rows):
    tag='th' if j==0 else 'td';out.append('<tr>'+''.join(f'<{tag}>'+inline(c)+f'</{tag}>' for c in row)+'</tr>')
   out.append('</table>');i-=1
  elif re.match(r'^\s*[-*] ',t) or re.match(r'^\d+\. ',t):
   flush();items=[];ordered=bool(re.match(r'^\d+\. ',t));tag='ol' if ordered else 'ul'
   pattern=r'^\d+\. ' if ordered else r'^\s*[-*] '
   while i<len(lines) and re.match(pattern,lines[i]):items.append(re.sub(pattern,'',lines[i]));i+=1
   out.append('<'+tag+'>'+''.join('<li>'+inline(item)+'</li>' for item in items)+'</'+tag+'>');i-=1
  elif not t.strip():flush()
  else:para.append(t)
  i+=1
 flush();return '\n'.join(out)
manual=p/'manual';manual.mkdir(exist_ok=True)
books=[('Игра','GAME.md'),('SDK плагинов','SDK.md'),('Язык Adder','ADDER.md'),('Команды терминала','COMMANDS.md'),('Механики взлома','HACK.md'),('Легенда','STORY.md')]
pages=[];groups=[]
for book,file in books:
 text=(p/'docs'/file).read_text(encoding='utf-8-sig')
 chunks=re.split(r'(?m)(?=^## )',text);entries=[]
 for j,chunk in enumerate(chunks):
  if not chunk.strip():continue
  title=(re.search(r'(?m)^#{1,2} (.+)',chunk).group(1) if re.search(r'(?m)^#{1,2} (.+)',chunk) else book)
  name=f'page{len(pages):03}.html';pages.append((name,title,chunk));entries.append((name,title))
 groups.append((book,entries))
for book,folder in [('Пример: устройства','device_lab'),('Пример: район','district')]:
 entries=[]
 for file in ['plugin.json','main.adr']:
  text=(p/'sdk/examples'/folder/file).read_text(encoding='utf-8-sig');title=f'{book} — {file}';name=f'page{len(pages):03}.html';pages.append((name,title,'# '+title+'\n\n```\n'+text+'\n```'));entries.append((name,title))
 groups.append((book,entries))
entries=[]
for f in sorted(s.glob('*.json')):
 name=f'page{len(pages):03}.html';title='Данные: '+f.stem;pages.append((name,title,'# '+title+'\n\nПример сериализованного объекта текущего ядра.\n\n```json\n'+f.read_text(encoding='utf-8')+'\n```'));entries.append((name,title))
groups.append(('Схемы данных',entries))
style='body{font:15px Segoe UI,Arial,sans-serif;margin:26px;color:#172434;background:#fafbfc;line-height:1.55}h1{color:#173a57;font-size:27px}h2{color:#225875}pre{background:#e9eef3;padding:14px;white-space:pre-wrap;word-wrap:break-word;font:13px Consolas,monospace}code{font-family:Consolas,monospace}table{border-collapse:collapse;width:100%;margin:16px 0}th,td{border:1px solid #bcc8d2;padding:8px;text-align:left;vertical-align:top}th{background:#dce7ef}a{color:#125a87}.nav{font-size:13px;border-bottom:1px solid #ccd8e0;padding-bottom:10px}'
for i,(name,title,md) in enumerate(pages):
 nav='<a href="index.html">Оглавление</a>'
 if i:nav+=' | <a href="'+pages[i-1][0]+'">Назад</a>'
 if i+1<len(pages):nav+=' | <a href="'+pages[i+1][0]+'">Далее</a>'
 document='<!DOCTYPE html><html lang="ru"><head><meta http-equiv="Content-Type" content="text/html; charset=windows-1251"><title>'+html.escape(title)+'</title><style>'+style+'</style></head><body><div class="nav">'+nav+'</div>'+render(md)+'</body></html>'
 (manual/name).write_bytes(document.encode('cp1251',errors='xmlcharrefreplace'))
index='<h1>Blackwire — игра и SDK Adder</h1><p>Руководство игрока, язык, команды и разработка расширений. API 1. 26 октября 2026.</p>'
for title,entries in groups:index+='<h2>'+title+'</h2><ul>'+''.join('<li><a href="'+name+'">'+html.escape(label)+'</a></li>' for name,label in entries)+'</ul>'
(manual/'index.html').write_bytes(('<html><head><meta http-equiv="Content-Type" content="text/html; charset=windows-1251"><title>Blackwire — руководство</title><style>'+style+'</style></head><body>'+index+'</body></html>').encode('cp1251',errors='xmlcharrefreplace'))
def obj(title,local):return '<LI><OBJECT type="text/sitemap"><param name="Name" value="'+html.escape(title,quote=True)+'"><param name="Local" value="'+local+'"></OBJECT>'
contents='<HTML><HEAD><meta http-equiv="Content-Type" content="text/html; charset=windows-1251"></HEAD><BODY><UL>'+obj('Blackwire — руководство','index.html')
for title,entries in groups:contents+=obj(title,entries[0][0])+'<UL>'+''.join(obj(label,name) for name,label in entries)+'</UL>'
contents+='</UL></BODY></HTML>'
(manual/'contents.hhc').write_bytes(contents.encode('cp1251',errors='xmlcharrefreplace'))
indexhh='<HTML><HEAD><meta http-equiv="Content-Type" content="text/html; charset=windows-1251"></HEAD><BODY><UL>'+''.join(obj(title,name) for name,title,_ in sorted(pages,key=lambda x:x[1]))+'</UL></BODY></HTML>'
(manual/'index.hhk').write_bytes(indexhh.encode('cp1251',errors='xmlcharrefreplace'))
hhp='[OPTIONS]\nCompatibility=1.1 or later\nCompiled file=Blackwire.chm\nContents file=contents.hhc\nIndex file=index.hhk\nDefault topic=index.html\nDisplay compile progress=No\nFull-text search=Yes\nLanguage=0x419 Russian\nTitle=Blackwire — игра и SDK Adder\n[FILES]\nindex.html\n'+'\n'.join(name for name,_,_ in pages)+'\n'
(manual/'Blackwire.hhp').write_bytes(hhp.encode('cp1251',errors='xmlcharrefreplace'))
# Verify every internal HTML link before compilation.
for f in manual.glob('*.html'):
 text=f.read_text(encoding='cp1251')
 for target in re.findall(r'href="([^"]+)"',text):assert (manual/target).is_file(),(f,target)
print(json.dumps({'pages':len(pages)+1,'groups':len(groups),'schema_objects':len(objects)},ensure_ascii=False))
