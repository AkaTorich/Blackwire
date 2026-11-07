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
        "integer_overflow" => (json!({"word_bits": 8, "header_bytes": 8 + n % 16, "max_allocation": 16, "min_body": 32 + n % 24}), vec![tr!("Размер выделения переполняет игровое целое число.", "The allocation size overflows a game integer."),tr!("Сложение declared и header_bytes выполняется по модулю 2 ** word_bits.", "declared + header_bytes is computed modulo 2 ** word_bits."),tr!("Добейся маленького выделения, но передай body длиной не меньше min_body.", "Get a small allocation, but send a body at least min_body long.")], vec!["declared","body"]),
        "signedness" => (json!({"word_bits": 16, "table_size": 8, "target_unsigned": 65535 - n % 3}), vec![tr!("Служба смешивает знаковый и беззнаковый индекс.", "The service mixes signed and unsigned indexes."),tr!("Отрицательный index проходит верхнюю проверку, затем преобразуется в беззнаковый.", "A negative index passes the upper bound check, then gets converted to unsigned."),tr!("Вычти 2 ** word_bits из target_unsigned.", "Subtract 2 ** word_bits from target_unsigned.")], vec!["index"]),
        "length_confusion" => (json!({"symbol": "Ж", "min_chars": 8 + n % 8, "byte_limit": 48, "suffix": nonce}), vec![tr!("Разборщик путает число символов и число байтов UTF-8.", "The parser confuses character count with UTF-8 byte count."),tr!("Составь text из многобайтового symbol; declared_chars должен совпасть с числом символов.", "Build text from the multibyte symbol; declared_chars must equal the character count."),tr!("Символ Ж занимает два байта. Передай от min_chars символов, оставаясь в byte_limit.", "The symbol Ж takes two bytes. Send at least min_chars characters while staying within byte_limit.")], vec!["text","declared_chars"]),
        "use_after_free" => (json!({"victim": format!("job_{}",n%97), "replacement": format!("reuse_{}",n%97), "marker": format!("privileged:{}",nonce)}), vec![tr!("Старая ссылка используется после освобождения объекта.", "A stale reference is used after the object is freed."),tr!("Освободи victim и займи тот же слот новым объектом replacement.", "Free victim and take the same slot with a new replacement object."),tr!("Передай alloc victim, free victim, alloc replacement с marker, use victim.", "Send alloc victim, free victim, alloc replacement with marker, use victim.")], vec!["operations"]),
        "double_free" => (json!({"owner": format!("owner_{}",n%97), "writer": "writer", "reader": "reader", "marker": format!("alias:{}",nonce)}), vec![tr!("Повторное освобождение создаёт два указателя на один слот.", "A double free creates two pointers to one slot."),tr!("Дважды освободи owner; выдели writer и reader, затем запиши через writer.", "Free owner twice; allocate writer and reader, then write through writer."),tr!("В operations используй alloc, free, write и read. Через reader должен читаться marker.", "Use alloc, free, write and read in operations. marker must be readable through reader.")], vec!["operations"]),
        "type_confusion" => (json!({"base": 4096+n%512, "stride": 16, "record_index": 2+n%7, "wire_type": "integer", "runtime_type": "privileged_record"}), vec![tr!("Число обрабатывается как адрес записи другого типа.", "A number is treated as the address of a record of another type."),tr!("Разные wire_type и runtime_type обходят проверку типа.", "Different wire_type and runtime_type bypass the type check."),tr!("Вычисли address как base + stride * record_index; типы возьми из layout.", "Compute address as base + stride * record_index; take the types from layout.")], vec!["wire_type","runtime_type","address"]),
        "object_reference" => (json!({"account": user, "victim_owner": "archive_admin", "records": [{"id":100+n%100,"owner":user},{"id":300+n%100,"owner":"archive_admin"},{"id":500+n%100,"owner":"public"}]}), vec![tr!("Проверяется вход пользователя, но не владелец записи.", "The user's login is checked, but not the record's owner."),tr!("Сохрани actor своего account, выбери запись другого владельца.", "Keep actor as your own account, pick a record of another owner."),tr!("Найди среди records запись владельца victim_owner и передай её id.", "Find the record owned by victim_owner among records and send its id.")], vec!["actor","record"]),
        "mass_assignment" => (json!({"allowed": ["title","note"], "hidden_field":format!("role_{}",n%97), "privileged_value":"administrator"}), vec![tr!("Проверенные поля и скрытые свойства применяются раздельно.", "Validated fields and hidden properties are applied separately."),tr!("В checked передай обычное поле, а в extra — скрытое поле роли.", "Send a regular field in checked and the hidden role field in extra."),tr!("Используй hidden_field и privileged_value; скрытое поле не должно находиться в checked.", "Use hidden_field and privileged_value; the hidden field must not be in checked.")], vec!["checked","extra"]),
        "token_confusion" => (json!({"issuer":format!("issuer_{}",n%17), "audience":format!("service_{}",n%23), "nonce":nonce, "role":"maintenance"}), vec![tr!("Служба путает издателя и получателя игрового токена.", "The service confuses the issuer and audience of a game token."),tr!("Алгоритм bw-none пропускает подпись, но поля токена всё ещё проверяются.", "The bw-none algorithm skips the signature, but the token fields are still checked."),tr!("В claims поменяй issuer и audience местами; передай role и nonce.", "Swap issuer and audience in claims; send role and nonce.")], vec!["algorithm","claims"]),
        "request_smuggling" => (json!({"prefix":format!("BW-PUBLIC {}",n%100), "route":format!("/internal/{}",n%97), "marker":nonce}), vec![tr!("Два игровых разборщика видят разные границы запроса.", "Two game parsers see different request boundaries."),tr!("Первая часть должна иметь длину front_length; вторая начинается после новой строки.", "The first part must be front_length long; the second starts after a newline."),tr!("request: prefix + новая строка + BW-NEXT route + новая строка + marker.", "request: prefix + newline + BW-NEXT route + newline + marker.")], vec!["front_length","request"]),
        "cache_poisoning" => (json!({"public_host":format!("public-{}",n%97), "origin_host":format!("internal-{}",n%97), "route":format!("/record/{}",n%53), "marker":nonce}), vec![tr!("Ключ общего кеша не учитывает имя хоста.", "The shared cache key ignores the host name."),tr!("Сначала положи ответ внутреннего узла в кеш, затем прочитай тот же ключ с публичного.", "First put the internal host's response into the cache, then read the same key from the public host."),tr!("requests: prime с origin_host/key/value=marker, затем fetch с public_host и тем же key.", "requests: prime with origin_host/key/value=marker, then fetch with public_host and the same key.")], vec!["requests"]),
        "header_injection" => (json!({"prefix":"guest", "header":format!("X-BW-{}",n%97), "role":"maintenance", "nonce":nonce}), vec![tr!("Управляющие символы внутри значения создают дополнительные заголовки.", "Control characters inside a value create extra headers."),tr!("Используй перевод строки CRLF между исходным значением, ролью и доказательством.", "Use a CRLF line break between the original value, the role and the proof."),"value: prefix + CRLF + header: role + CRLF + X-BW-Proof: nonce."], vec!["value"]),
        "unsafe_deserialization" => (json!({"type_alias":format!("report_{}",n%97), "hook":format!("restore_{}",n%13), "argument":nonce}), vec![tr!("Восстановление виртуального объекта вызывает небезопасный hook.", "Restoring a virtual object calls an unsafe hook."),tr!("object должен содержать type и вложенный state.", "object must contain type and a nested state."),tr!("В state передай hook и argument из layout; это данные игрового объекта, не код ОС.", "Put hook and argument from layout into state; this is game object data, not OS code.")], vec!["object"]),
        "prototype_pollution" => (json!({"flag":format!("allow_{}",n%97), "base_kind":"guest"}), vec![tr!("Объект наследует флаг из игрового прототипа.", "The object inherits a flag from the game prototype."),tr!("Сам объект не должен содержать собственный флаг доступа.", "The object itself must not have its own access flag."),tr!("path: список __proto__ и flag; value=true; object содержит только kind=base_kind.", "path: a list of __proto__ and flag; value=true; object contains only kind=base_kind.")], vec!["object","path","value"]),
        "template_injection" => (json!({"numbers":[2+n%7,3+(n>>8)%11,5+(n>>16)%13], "factor":2+n%4, "role":"maintenance"}), vec![tr!("Игровой шаблон вычисляет выражение в служебном контексте.", "A game template evaluates an expression in a service context."),tr!("Сложи numbers и умножь сумму на factor.", "Add up numbers and multiply the sum by factor."),tr!("Собери строку {{bw:результат;role=role}} с вычисленным числом и ролью из layout.", "Build the string {{bw:result;role=role}} with the computed number and the role from layout.")], vec!["template"]),
        "xml_entity" => (json!({"entity":format!("entry_{}",n%97), "target_file":path}), vec![tr!("Игровая XML-служба разрешает внешнюю сущность как путь к файлу.", "A game XML service resolves an external entity as a file path."),tr!("Объяви entities: имя сущности → словарь system с target_file.", "Declare entities: entity name → a dict with system set to target_file."),tr!("document: <bw>&имя_сущности;</bw>. Сеть и файловая система Windows не используются.", "document: <bw>&entity_name;</bw>. The Windows network and file system are not used.")], vec!["entities","document"]),
        "regex_backtracking" => (json!({"letter":"a", "budget":256+(n%4)*128, "terminator":"!"}), vec![tr!("Неудачное совпадение раскрывает диагностический режим службы.", "A failed match exposes the service's diagnostic mode."),tr!("Оценка числа состояний равна 2 ** число повторений letter; реальный regexp не запускается.", "The state count estimate is 2 ** the number of letter repeats; no real regexp runs."),tr!("Найди число повторений, превышающее budget, добавь terminator. Не более 32 повторений.", "Find a repeat count that exceeds budget, append terminator. At most 32 repeats.")], vec!["input"]),
        "fragment_overlap" => (json!({"prefix":format!("BW-{}:",n%97), "safe":"guest", "privileged":"admin"}), vec![tr!("Фрагменты с перекрытием дают разные сообщения двум сборщикам.", "Overlapping fragments give two reassemblers different messages."),tr!("Первый фрагмент: prefix + safe. Второй перекрывает слово safe.", "First fragment: prefix + safe. The second overlaps the word safe."),tr!("Во втором фрагменте offset=len(prefix), data=privileged.", "In the second fragment offset=len(prefix), data=privileged.")], vec!["fragments"]),
        "weak_rng" => (rng_layout(n), vec![tr!("Игровые токены создаёт слабый линейный генератор.", "Game tokens come from a weak linear generator."),tr!("Найди a и c: следующий = (a * текущий + c) % modulus.", "Find a and c: next = (a * current + c) % modulus."),tr!("Перебери a, вычисляя c по первым двум observed, проверь третье значение и предскажи четвёртое.", "Try each a, computing c from the first two observed values, check the third and predict the fourth.")], vec!["token"]),
        "permission_mask" => (json!({"visible_bits":4, "required_low":1+n%3, "admin_bit":5+n%3}), vec![tr!("Проверка видит только младшие биты, применение — всю маску.", "The check sees only the low bits, the apply step sees the whole mask."),tr!("Сохрани required_low в младших visible_bits.", "Keep required_low in the low visible_bits."),tr!("Добавь 2 ** admin_bit к required_low.", "Add 2 ** admin_bit to required_low.")], vec!["mask"]),
        _=>return None
    })
}
fn s<'a>(v:&'a Value,k:&str)->Result<&'a str,String>{v[k].as_str().ok_or_else(||trf!("{} должен быть строкой", "{} must be a string", k))}
fn u(v:&Value,k:&str)->Result<u64,String>{v[k].as_u64().ok_or_else(||trf!("{} должен быть неотрицательным целым", "{} must be a non-negative integer", k))}
fn list<'a>(v:&'a Value,k:&str)->Result<&'a Vec<Value>,String>{v[k].as_array().ok_or_else(||trf!("{} должен быть списком", "{} must be a list", k))}
fn check(ok:bool,message:&str)->Result<(),String>{if ok{Ok(())}else{Err(message.into())}}
fn heap(kind:&str,l:&Value,p:&Value)->Result<(),String>{
    let ops=list(p,"operations")?;if ops.len()>16{return Err(tr!("не больше 16 операций с кучей", "at most 16 heap operations").into());}
    let mut handles:BTreeMap<String,(usize,bool)>=BTreeMap::new();
    let mut memory:BTreeMap<usize,String>=BTreeMap::new();
    let mut free=Vec::new();let mut next=0;let mut success=false;
    for op in ops {
        let action=s(op,"op")?;let name=s(op,"handle")?;
        match action {
            "alloc"=>{
                if handles.contains_key(name){return Err(tr!("имя handle уже использовалось", "handle name already used").into());}
                let slot=free.pop().unwrap_or_else(||{let v=next;next+=1;v});
                handles.insert(name.into(),(slot,true));memory.insert(slot,s(op,"value")?.into());
            }
            "free"=>{
                let record=handles.get_mut(name).ok_or(tr!("free: неизвестный handle", "free: unknown handle"))?;
                if !record.1 && kind!="double_free"{return Err(tr!("повторное free не относится к этому типу", "a repeated free does not belong to this type").into());}
                record.1=false;free.push(record.0);
            }
            "use" if kind=="use_after_free"=>{
                let record=handles.get(name).ok_or(tr!("use: неизвестный handle", "use: unknown handle"))?;
                success=name==s(l,"victim")? && !record.1 && handles.iter().any(|(h,r)|h!=name&&r.1&&r.0==record.0) && memory.get(&record.0).map(String::as_str)==Some(s(l,"marker")?);
            }
            "write"=>{
                let record=handles.get(name).filter(|r|r.1).ok_or(tr!("write: handle не выделен", "write: handle not allocated"))?;
                memory.insert(record.0,s(op,"value")?.into());
            }
            "read" if kind=="double_free"=>{
                let record=handles.get(name).filter(|r|r.1).ok_or(tr!("read: handle не выделен", "read: handle not allocated"))?;
                let aliases=handles.values().filter(|r|r.1 && r.0==record.0).count();
                success=aliases>1 && memory.get(&record.0).map(String::as_str)==Some(s(l,"marker")?);
            }
            _=>return Err(tr!("операция не подходит к модели кучи", "operation does not fit the heap model").into())
        }
    }
    check(success,tr!("Ссылка не попала в переиспользованный слот с marker; проверь историю alloc/free.", "The reference did not land in the reused slot with marker; check the alloc/free history."))
}
pub fn validate(profile:&Value,p:&Value)->Result<(),String>{
    let l=&profile["layout"];
    match s(profile,"kind")? {
        "integer_overflow"=>{
            let modulus=1u64<<u(l,"word_bits")?;let declared=u(p,"declared")?;
            let body=s(p,"body")?;
            check(declared<modulus && (declared+u(l,"header_bytes")?)%modulus<u(l,"max_allocation")? &&
                  body.len()>=u(l,"min_body")? as usize && body.len()<=256,tr!("Размер не переполнился либо body не выходит за маленькое выделение.", "The size did not overflow, or body does not exceed the small allocation."))
        }
        "signedness"=>{
            let index=p["index"].as_i64().ok_or(tr!("index должен быть знаковым целым", "index must be a signed integer"))?;
            let modulus=1i64<<u(l,"word_bits")?;
            check(index<0 && index>=-modulus/2 && (index+modulus) as u64==u(l,"target_unsigned")?,tr!("Знаковый индекс не преобразуется в нужную беззнаковую запись.", "The signed index does not convert to the required unsigned record."))
        }
        "length_confusion"=>{
            let text=s(p,"text")?;let chars=text.chars().count();
            check(u(p,"declared_chars")?==chars as u64 && chars>=u(l,"min_chars")? as usize &&
                  text.len()>chars && text.len()<=u(l,"byte_limit")? as usize &&
                  text.chars().all(|c|s(l,"symbol").unwrap().starts_with(c)),tr!("Не совпали число символов, байтовая длина или разрешённый многобайтовый symbol.", "Character count, byte length or the allowed multibyte symbol did not match."))
        }
        "use_after_free"|"double_free"=>heap(s(profile,"kind")?,l,p),
        "type_confusion"=>check(p["wire_type"]==l["wire_type"] && p["runtime_type"]==l["runtime_type"] &&
            u(p,"address")?==u(l,"base")?+u(l,"stride")?*u(l,"record_index")?,tr!("Теги типов или адрес записи не совпали.", "Type tags or record address did not match.")),
        "object_reference"=>check(p["actor"]==l["account"] && list(l,"records")?.iter().any(|r|r["owner"]==l["victim_owner"] && r["id"]==p["record"]),tr!("Выбери запись victim_owner, сохранив собственный actor.", "Pick the victim_owner record while keeping your own actor.")),
        "mass_assignment"=>{
            let checked=p["checked"].as_object().ok_or(tr!("checked должен быть словарём", "checked must be a dict"))?;
            check(!checked.is_empty() && checked.keys().all(|k|list(l,"allowed").unwrap().iter().any(|v|v==k)) &&
                  p["extra"][s(l,"hidden_field")?]==l["privileged_value"],tr!("checked должен содержать обычные поля; скрытая роль передаётся через extra.", "checked must contain regular fields; the hidden role goes through extra."))
        }
        "token_confusion"=>check(p["algorithm"]=="bw-none" && p["claims"]["issuer"]==l["audience"] &&
            p["claims"]["audience"]==l["issuer"] && p["claims"]["role"]==l["role"] && p["claims"]["nonce"]==l["nonce"],tr!("Алгоритм или перепутанные issuer/audience/role/nonce не подошли.", "The algorithm or the swapped issuer/audience/role/nonce did not fit.")),
        "request_smuggling"=>{
            let prefix=s(l,"prefix")?;
            check(u(p,"front_length")?==prefix.len() as u64 && s(p,"request")?==format!("{}\nBW-NEXT {}\n{}",prefix,s(l,"route")?,s(l,"marker")?),tr!("Два разборщика не разделили запрос на публичную и внутреннюю части.", "The two parsers did not split the request into public and internal parts."))
        }
        "cache_poisoning"=>{
            let r=list(p,"requests")?;if r.len()!=2{return Err(tr!("нужны prime и fetch", "prime and fetch are required").into());}
            check(r[0]["op"]=="prime" && r[1]["op"]=="fetch" && r[0]["host"]==l["origin_host"] &&
                r[1]["host"]==l["public_host"] && r[0]["key"]==l["route"] && r[1]["key"]==l["route"] && r[0]["value"]==l["marker"],tr!("Не создалась коллизия ключа кеша между внутренним и публичным хостом.", "No cache key collision between the internal and public host."))
        }
        "header_injection"=>check(s(p,"value")?==format!("{}\r\n{}: {}\r\nX-BW-Proof: {}",s(l,"prefix")?,s(l,"header")?,s(l,"role")?,s(l,"nonce")?),tr!("Неверны CRLF или дополнительные игровые заголовки.", "Wrong CRLF or extra game headers.")),
        "unsafe_deserialization"=>check(p["object"]["type"]==l["type_alias"] && p["object"]["state"]["hook"]==l["hook"] &&
            p["object"]["state"]["argument"]==l["argument"],tr!("Не сработало восстановление нужного виртуального типа и hook.", "Restoring the required virtual type and hook did not work.")),
        "prototype_pollution"=>check(p["path"]==json!(["__proto__",s(l,"flag")?]) && p["value"]==true &&
            p["object"]["kind"]==l["base_kind"] && p["object"].as_object().is_some_and(|o|!o.contains_key(s(l,"flag").unwrap())),tr!("Нужно изменить наследуемый флаг; собственный флаг объекта должен отсутствовать.", "Change the inherited flag; the object must not have its own flag.")),
        "template_injection"=>{
            let sum:u64=list(l,"numbers")?.iter().map(|v|v.as_u64().unwrap()).sum();
            check(s(p,"template")?==format!("{{{{bw:{};role={}}}}}",sum*u(l,"factor")?,s(l,"role")?),tr!("Игровой шаблон не вычислил нужное значение в служебной роли.", "The game template did not compute the required value in the service role."))
        }
        "xml_entity"=>check(p["entities"][s(l,"entity")?]["system"]==l["target_file"] &&
            s(p,"document")?==format!("<bw>&{};</bw>",s(l,"entity")?),tr!("Не совпали объявление внешней сущности и ссылка на неё.", "The external entity declaration and its reference did not match.")),
        "regex_backtracking"=>{
            let text=s(p,"input")?;let prefix=text.strip_suffix(s(l,"terminator")?).ok_or(tr!("нужен завершающий символ неудачного совпадения", "a terminating character for the failed match is required"))?;
            let count=prefix.chars().count();
            check(count<=32 && prefix.chars().all(|c|s(l,"letter").unwrap().starts_with(c)) &&
                (1u64<<count)>u(l,"budget")?,tr!("Виртуальное число возвратов не превысило budget.", "The virtual backtrack count did not exceed budget."))
        }
        "fragment_overlap"=>{
            let fragments=list(p,"fragments")?;
            if fragments.len()>8{return Err(tr!("не больше 8 фрагментов", "at most 8 fragments").into());}
            let mut first=vec![None;128];let mut last=vec![None;128];let mut end=0;
            for fragment in fragments {
                let offset=u(fragment,"offset")? as usize;let data=s(fragment,"data")?.as_bytes();
                if offset>128 || data.len()>128-offset{return Err(tr!("фрагмент выходит за границу игрового сообщения", "fragment goes past the game message boundary").into());}
                end=end.max(offset+data.len());
                for (i,b) in data.iter().enumerate(){first[offset+i].get_or_insert(*b);last[offset+i]=Some(*b);}
            }
            let a:Option<Vec<u8>>=first[..end].iter().copied().collect();let b:Option<Vec<u8>>=last[..end].iter().copied().collect();
            check(a==Some(format!("{}{}",s(l,"prefix")?,s(l,"safe")?).into_bytes()) &&
                b==Some(format!("{}{}",s(l,"prefix")?,s(l,"privileged")?).into_bytes()),tr!("Перекрытие не дало безопасное и привилегированное сообщения двум сборщикам.", "The overlap did not give the safe and privileged messages to the two reassemblers."))
        }
        "weak_rng"=>{
            let x=list(l,"observed")?;let m=u(l,"modulus")?;let mut expected=None;
            for a in 1..m {let c=(x[1].as_u64().unwrap()+m-(a*x[0].as_u64().unwrap())%m)%m;
                if (a*x[1].as_u64().unwrap()+c)%m==x[2].as_u64().unwrap(){expected=Some((a*x[2].as_u64().unwrap()+c)%m);break;}}
            check(p["token"].as_u64()==expected,tr!("Не предсказан следующий токен слабого генератора.", "The weak generator's next token was not predicted."))
        }
        "permission_mask"=>{
            let mask=u(p,"mask")?;
            check(mask<65536 && mask%(1u64<<u(l,"visible_bits")?)==u(l,"required_low")? &&
                mask&(1u64<<u(l,"admin_bit")?)!=0,tr!("Младшие разрешения и скрытый привилегированный бит не совпали.", "Low permissions and the hidden privileged bit did not match."))
        }
        _=>Err(tr!("неизвестная дополнительная механика", "unknown additional mechanic").into())
    }
}
