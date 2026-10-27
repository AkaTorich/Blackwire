//! Additional, bounded simulations of vulnerability classes. No host network/OS access.
use serde_json::{json, Value};
use std::collections::BTreeMap;
pub type Puzzle = (Value, Vec<&'static str>, Vec<&'static str>);
pub const KINDS: &[&str] = &["integer_overflow", "signedness", "length_confusion", "use_after_free", "double_free", "type_confusion", "object_reference", "mass_assignment", "token_confusion", "request_smuggling", "cache_poisoning", "header_injection", "unsafe_deserialization", "prototype_pollution", "template_injection", "xml_entity", "regex_backtracking", "fragment_overlap", "weak_rng", "permission_mask"];
fn rng_layout(n:u64)->Value {
    let modulus=97;let a=2+n%16;let c=1+(n>>8)%9;let x=n%modulus;
    let y=(a*x+c)%modulus;let z=(a*y+c)%modulus;
    json!({"modulus":modulus,"observed":[x,y,z]})
}
pub fn profile(kind:&str,n:u64,_difficulty:u64,user:&str,path:&str,nonce:&str)->Option<Puzzle>{
    Some(match kind {
        "integer_overflow" => (json!({"word_bits": 8, "header_bytes": 8 + n % 16, "max_allocation": 16, "min_body": 32 + n % 24}), vec!["Размер выделения переполняет игровое целое число.","Сложение declared и header_bytes выполняется по модулю 2 ** word_bits.","Добейся маленького выделения, но передай body длиной не меньше min_body."], vec!["declared","body"]),
        "signedness" => (json!({"word_bits": 16, "table_size": 8, "target_unsigned": 65535 - n % 3}), vec!["Служба смешивает знаковый и беззнаковый индекс.","Отрицательный index проходит верхнюю проверку, затем преобразуется в беззнаковый.","Вычти 2 ** word_bits из target_unsigned."], vec!["index"]),
        "length_confusion" => (json!({"symbol": "Ж", "min_chars": 8 + n % 8, "byte_limit": 48, "suffix": nonce}), vec!["Разборщик путает число символов и число байтов UTF-8.","Составь text из многобайтового symbol; declared_chars должен совпасть с числом символов.","Символ Ж занимает два байта. Передай от min_chars символов, оставаясь в byte_limit."], vec!["text","declared_chars"]),
        "use_after_free" => (json!({"victim": format!("job_{}",n%97), "replacement": format!("reuse_{}",n%97), "marker": format!("privileged:{}",nonce)}), vec!["Старая ссылка используется после освобождения объекта.","Освободи victim и займи тот же слот новым объектом replacement.","Передай alloc victim, free victim, alloc replacement с marker, use victim."], vec!["operations"]),
        "double_free" => (json!({"owner": format!("owner_{}",n%97), "writer": "writer", "reader": "reader", "marker": format!("alias:{}",nonce)}), vec!["Повторное освобождение создаёт два указателя на один слот.","Дважды освободи owner; выдели writer и reader, затем запиши через writer.","В operations используй alloc, free, write и read. Через reader должен читаться marker."], vec!["operations"]),
        "type_confusion" => (json!({"base": 4096+n%512, "stride": 16, "record_index": 2+n%7, "wire_type": "integer", "runtime_type": "privileged_record"}), vec!["Число обрабатывается как адрес записи другого типа.","Разные wire_type и runtime_type обходят проверку типа.","Вычисли address как base + stride * record_index; типы возьми из layout."], vec!["wire_type","runtime_type","address"]),
        "object_reference" => (json!({"account": user, "victim_owner": "archive_admin", "records": [{"id":100+n%100,"owner":user},{"id":300+n%100,"owner":"archive_admin"},{"id":500+n%100,"owner":"public"}]}), vec!["Проверяется вход пользователя, но не владелец записи.","Сохрани actor своего account, выбери запись другого владельца.","Найди среди records запись владельца victim_owner и передай её id."], vec!["actor","record"]),
        "mass_assignment" => (json!({"allowed": ["title","note"], "hidden_field":format!("role_{}",n%97), "privileged_value":"administrator"}), vec!["Проверенные поля и скрытые свойства применяются раздельно.","В checked передай обычное поле, а в extra — скрытое поле роли.","Используй hidden_field и privileged_value; скрытое поле не должно находиться в checked."], vec!["checked","extra"]),
        "token_confusion" => (json!({"issuer":format!("issuer_{}",n%17), "audience":format!("service_{}",n%23), "nonce":nonce, "role":"maintenance"}), vec!["Служба путает издателя и получателя игрового токена.","Алгоритм bw-none пропускает подпись, но поля токена всё ещё проверяются.","В claims поменяй issuer и audience местами; передай role и nonce."], vec!["algorithm","claims"]),
        "request_smuggling" => (json!({"prefix":format!("BW-PUBLIC {}",n%100), "route":format!("/internal/{}",n%97), "marker":nonce}), vec!["Два игровых разборщика видят разные границы запроса.","Первая часть должна иметь длину front_length; вторая начинается после новой строки.","request: prefix + новая строка + BW-NEXT route + новая строка + marker."], vec!["front_length","request"]),
        "cache_poisoning" => (json!({"public_host":format!("public-{}",n%97), "origin_host":format!("internal-{}",n%97), "route":format!("/record/{}",n%53), "marker":nonce}), vec!["Ключ общего кеша не учитывает имя хоста.","Сначала положи ответ внутреннего узла в кеш, затем прочитай тот же ключ с публичного.","requests: prime с origin_host/key/value=marker, затем fetch с public_host и тем же key."], vec!["requests"]),
        "header_injection" => (json!({"prefix":"guest", "header":format!("X-BW-{}",n%97), "role":"maintenance", "nonce":nonce}), vec!["Управляющие символы внутри значения создают дополнительные заголовки.","Используй перевод строки CRLF между исходным значением, ролью и доказательством.","value: prefix + CRLF + header: role + CRLF + X-BW-Proof: nonce."], vec!["value"]),
        "unsafe_deserialization" => (json!({"type_alias":format!("report_{}",n%97), "hook":format!("restore_{}",n%13), "argument":nonce}), vec!["Восстановление виртуального объекта вызывает небезопасный hook.","object должен содержать type и вложенный state.","В state передай hook и argument из layout; это данные игрового объекта, не код ОС."], vec!["object"]),
        "prototype_pollution" => (json!({"flag":format!("allow_{}",n%97), "base_kind":"guest"}), vec!["Объект наследует флаг из игрового прототипа.","Сам объект не должен содержать собственный флаг доступа.","path: список __proto__ и flag; value=true; object содержит только kind=base_kind."], vec!["object","path","value"]),
        "template_injection" => (json!({"numbers":[2+n%7,3+(n>>8)%11,5+(n>>16)%13], "factor":2+n%4, "role":"maintenance"}), vec!["Игровой шаблон вычисляет выражение в служебном контексте.","Сложи numbers и умножь сумму на factor.","Собери строку {{bw:результат;role=role}} с вычисленным числом и ролью из layout."], vec!["template"]),
        "xml_entity" => (json!({"entity":format!("entry_{}",n%97), "target_file":path}), vec!["Игровая XML-служба разрешает внешнюю сущность как путь к файлу.","Объяви entities: имя сущности → словарь system с target_file.","document: <bw>&имя_сущности;</bw>. Сеть и файловая система Windows не используются."], vec!["entities","document"]),
        "regex_backtracking" => (json!({"letter":"a", "budget":256+(n%4)*128, "terminator":"!"}), vec!["Неудачное совпадение раскрывает диагностический режим службы.","Оценка числа состояний равна 2 ** число повторений letter; реальный regexp не запускается.","Найди число повторений, превышающее budget, добавь terminator. Не более 32 повторений."], vec!["input"]),
        "fragment_overlap" => (json!({"prefix":format!("BW-{}:",n%97), "safe":"guest", "privileged":"admin"}), vec!["Фрагменты с перекрытием дают разные сообщения двум сборщикам.","Первый фрагмент: prefix + safe. Второй перекрывает слово safe.","Во втором фрагменте offset=len(prefix), data=privileged."], vec!["fragments"]),
        "weak_rng" => (rng_layout(n), vec!["Игровые токены создаёт слабый линейный генератор.","Найди a и c: следующий = (a * текущий + c) % modulus.","Перебери a, вычисляя c по первым двум observed, проверь третье значение и предскажи четвёртое."], vec!["token"]),
        "permission_mask" => (json!({"visible_bits":4, "required_low":1+n%3, "admin_bit":5+n%3}), vec!["Проверка видит только младшие биты, применение — всю маску.","Сохрани required_low в младших visible_bits.","Добавь 2 ** admin_bit к required_low."], vec!["mask"]),
        _=>return None
    })
}
fn s<'a>(v:&'a Value,k:&str)->Result<&'a str,String>{v[k].as_str().ok_or_else(||format!("{} должен быть строкой",k))}
fn u(v:&Value,k:&str)->Result<u64,String>{v[k].as_u64().ok_or_else(||format!("{} должен быть неотрицательным целым",k))}
fn list<'a>(v:&'a Value,k:&str)->Result<&'a Vec<Value>,String>{v[k].as_array().ok_or_else(||format!("{} должен быть списком",k))}
fn check(ok:bool,message:&str)->Result<(),String>{if ok{Ok(())}else{Err(message.into())}}
fn heap(kind:&str,l:&Value,p:&Value)->Result<(),String>{
    let ops=list(p,"operations")?;if ops.len()>16{return Err("не больше 16 операций с кучей".into());}
    let mut handles:BTreeMap<String,(usize,bool)>=BTreeMap::new();
    let mut memory:BTreeMap<usize,String>=BTreeMap::new();
    let mut free=Vec::new();let mut next=0;let mut success=false;
    for op in ops {
        let action=s(op,"op")?;let name=s(op,"handle")?;
        match action {
            "alloc"=>{
                if handles.contains_key(name){return Err("имя handle уже использовалось".into());}
                let slot=free.pop().unwrap_or_else(||{let v=next;next+=1;v});
                handles.insert(name.into(),(slot,true));memory.insert(slot,s(op,"value")?.into());
            }
            "free"=>{
                let record=handles.get_mut(name).ok_or("free: неизвестный handle")?;
                if !record.1 && kind!="double_free"{return Err("повторное free не относится к этому типу".into());}
                record.1=false;free.push(record.0);
            }
            "use" if kind=="use_after_free"=>{
                let record=handles.get(name).ok_or("use: неизвестный handle")?;
                success=name==s(l,"victim")? && !record.1 && handles.iter().any(|(h,r)|h!=name&&r.1&&r.0==record.0) && memory.get(&record.0).map(String::as_str)==Some(s(l,"marker")?);
            }
            "write"=>{
                let record=handles.get(name).filter(|r|r.1).ok_or("write: handle не выделен")?;
                memory.insert(record.0,s(op,"value")?.into());
            }
            "read" if kind=="double_free"=>{
                let record=handles.get(name).filter(|r|r.1).ok_or("read: handle не выделен")?;
                let aliases=handles.values().filter(|r|r.1 && r.0==record.0).count();
                success=aliases>1 && memory.get(&record.0).map(String::as_str)==Some(s(l,"marker")?);
            }
            _=>return Err("операция не подходит к модели кучи".into())
        }
    }
    check(success,"Ссылка не попала в переиспользованный слот с marker; проверь историю alloc/free.")
}
pub fn validate(profile:&Value,p:&Value)->Result<(),String>{
    let l=&profile["layout"];
    match s(profile,"kind")? {
        "integer_overflow"=>{
            let modulus=1u64<<u(l,"word_bits")?;let declared=u(p,"declared")?;
            let body=s(p,"body")?;
            check(declared<modulus && (declared+u(l,"header_bytes")?)%modulus<u(l,"max_allocation")? &&
                  body.len()>=u(l,"min_body")? as usize && body.len()<=256,"Размер не переполнился либо body не выходит за маленькое выделение.")
        }
        "signedness"=>{
            let index=p["index"].as_i64().ok_or("index должен быть знаковым целым")?;
            let modulus=1i64<<u(l,"word_bits")?;
            check(index<0 && index>=-modulus/2 && (index+modulus) as u64==u(l,"target_unsigned")?,"Знаковый индекс не преобразуется в нужную беззнаковую запись.")
        }
        "length_confusion"=>{
            let text=s(p,"text")?;let chars=text.chars().count();
            check(u(p,"declared_chars")?==chars as u64 && chars>=u(l,"min_chars")? as usize &&
                  text.len()>chars && text.len()<=u(l,"byte_limit")? as usize &&
                  text.chars().all(|c|s(l,"symbol").unwrap().starts_with(c)),"Не совпали число символов, байтовая длина или разрешённый многобайтовый symbol.")
        }
        "use_after_free"|"double_free"=>heap(s(profile,"kind")?,l,p),
        "type_confusion"=>check(p["wire_type"]==l["wire_type"] && p["runtime_type"]==l["runtime_type"] &&
            u(p,"address")?==u(l,"base")?+u(l,"stride")?*u(l,"record_index")?,"Теги типов или адрес записи не совпали."),
        "object_reference"=>check(p["actor"]==l["account"] && list(l,"records")?.iter().any(|r|r["owner"]==l["victim_owner"] && r["id"]==p["record"]),"Выбери запись victim_owner, сохранив собственный actor."),
        "mass_assignment"=>{
            let checked=p["checked"].as_object().ok_or("checked должен быть словарём")?;
            check(!checked.is_empty() && checked.keys().all(|k|list(l,"allowed").unwrap().iter().any(|v|v==k)) &&
                  p["extra"][s(l,"hidden_field")?]==l["privileged_value"],"checked должен содержать обычные поля; скрытая роль передаётся через extra.")
        }
        "token_confusion"=>check(p["algorithm"]=="bw-none" && p["claims"]["issuer"]==l["audience"] &&
            p["claims"]["audience"]==l["issuer"] && p["claims"]["role"]==l["role"] && p["claims"]["nonce"]==l["nonce"],"Алгоритм или перепутанные issuer/audience/role/nonce не подошли."),
        "request_smuggling"=>{
            let prefix=s(l,"prefix")?;
            check(u(p,"front_length")?==prefix.len() as u64 && s(p,"request")?==format!("{}\nBW-NEXT {}\n{}",prefix,s(l,"route")?,s(l,"marker")?),"Два разборщика не разделили запрос на публичную и внутреннюю части.")
        }
        "cache_poisoning"=>{
            let r=list(p,"requests")?;if r.len()!=2{return Err("нужны prime и fetch".into());}
            check(r[0]["op"]=="prime" && r[1]["op"]=="fetch" && r[0]["host"]==l["origin_host"] &&
                r[1]["host"]==l["public_host"] && r[0]["key"]==l["route"] && r[1]["key"]==l["route"] && r[0]["value"]==l["marker"],"Не создалась коллизия ключа кеша между внутренним и публичным хостом.")
        }
        "header_injection"=>check(s(p,"value")?==format!("{}\r\n{}: {}\r\nX-BW-Proof: {}",s(l,"prefix")?,s(l,"header")?,s(l,"role")?,s(l,"nonce")?),"Неверны CRLF или дополнительные игровые заголовки."),
        "unsafe_deserialization"=>check(p["object"]["type"]==l["type_alias"] && p["object"]["state"]["hook"]==l["hook"] &&
            p["object"]["state"]["argument"]==l["argument"],"Не сработало восстановление нужного виртуального типа и hook."),
        "prototype_pollution"=>check(p["path"]==json!(["__proto__",s(l,"flag")?]) && p["value"]==true &&
            p["object"]["kind"]==l["base_kind"] && p["object"].as_object().is_some_and(|o|!o.contains_key(s(l,"flag").unwrap())),"Нужно изменить наследуемый флаг; собственный флаг объекта должен отсутствовать."),
        "template_injection"=>{
            let sum:u64=list(l,"numbers")?.iter().map(|v|v.as_u64().unwrap()).sum();
            check(s(p,"template")?==format!("{{{{bw:{};role={}}}}}",sum*u(l,"factor")?,s(l,"role")?),"Игровой шаблон не вычислил нужное значение в служебной роли.")
        }
        "xml_entity"=>check(p["entities"][s(l,"entity")?]["system"]==l["target_file"] &&
            s(p,"document")?==format!("<bw>&{};</bw>",s(l,"entity")?),"Не совпали объявление внешней сущности и ссылка на неё."),
        "regex_backtracking"=>{
            let text=s(p,"input")?;let prefix=text.strip_suffix(s(l,"terminator")?).ok_or("нужен завершающий символ неудачного совпадения")?;
            let count=prefix.chars().count();
            check(count<=32 && prefix.chars().all(|c|s(l,"letter").unwrap().starts_with(c)) &&
                (1u64<<count)>u(l,"budget")?,"Виртуальное число возвратов не превысило budget.")
        }
        "fragment_overlap"=>{
            let fragments=list(p,"fragments")?;
            if fragments.len()>8{return Err("не больше 8 фрагментов".into());}
            let mut first=vec![None;128];let mut last=vec![None;128];let mut end=0;
            for fragment in fragments {
                let offset=u(fragment,"offset")? as usize;let data=s(fragment,"data")?.as_bytes();
                if offset>128 || data.len()>128-offset{return Err("фрагмент выходит за границу игрового сообщения".into());}
                end=end.max(offset+data.len());
                for (i,b) in data.iter().enumerate(){first[offset+i].get_or_insert(*b);last[offset+i]=Some(*b);}
            }
            let a:Option<Vec<u8>>=first[..end].iter().copied().collect();let b:Option<Vec<u8>>=last[..end].iter().copied().collect();
            check(a==Some(format!("{}{}",s(l,"prefix")?,s(l,"safe")?).into_bytes()) &&
                b==Some(format!("{}{}",s(l,"prefix")?,s(l,"privileged")?).into_bytes()),"Перекрытие не дало безопасное и привилегированное сообщения двум сборщикам.")
        }
        "weak_rng"=>{
            let x=list(l,"observed")?;let m=u(l,"modulus")?;let mut expected=None;
            for a in 1..m {let c=(x[1].as_u64().unwrap()+m-(a*x[0].as_u64().unwrap())%m)%m;
                if (a*x[1].as_u64().unwrap()+c)%m==x[2].as_u64().unwrap(){expected=Some((a*x[2].as_u64().unwrap()+c)%m);break;}}
            check(p["token"].as_u64()==expected,"Не предсказан следующий токен слабого генератора.")
        }
        "permission_mask"=>{
            let mask=u(p,"mask")?;
            check(mask<65536 && mask%(1u64<<u(l,"visible_bits")?)==u(l,"required_low")? &&
                mask&(1u64<<u(l,"admin_bit")?)!=0,"Младшие разрешения и скрытый привилегированный бит не совпали.")
        }
        _=>Err("неизвестная дополнительная механика".into())
    }
}
