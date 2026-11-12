//! Проверить онлайн-сервер так же, как его видит игра (TLS игры + протокол),
//! без учётной записи: вход несуществующим именем должен вернуть отказ.
//!   cargo run -p blackwire-wire --example probe -- game.globthe.top
fn main() {
    let addr = std::env::args().nth(1).unwrap_or_else(|| "game.globthe.top".into());
    match blackwire_wire::connect(&addr) {
        Ok((w, mut r)) => {
            let auth = format!(r#"{{"id":0,"cmd":"auth","args":{{"user":"probe-check","pass":"xxxxxx","register":false,"lang":"ru","proto":{}}}}}"#, blackwire_wire::PROTOCOL);
            w.send(&auth).expect("отправка");
            println!("связь есть ({}), ответ: {:?}", w.peer(), r.next_line());
        }
        Err(e) => {
            println!("нет связи: {}", e);
            std::process::exit(1);
        }
    }
}
