//! Связь игры с онлайн-сервером: построчный JSON поверх TLS (по умолчанию) или
//! обычного TCP (адрес вида `tcp://хост:порт` — для своей локальной сети).
//!
//! Соединение делится на `Writer` (можно клонировать и писать из любых потоков)
//! и `Reader` (читает строки в одном потоке). TLS — rustls: чтение сырого сокета
//! идёт без замка, замок берётся только на разбор и шифрование, поэтому чтение
//! и запись не мешают друг другу.

use std::io::{self, ErrorKind, Read, Write};
use std::net::{Shutdown, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, ClientConnection, Connection, RootCertStore, ServerConfig, ServerConnection};

pub use rustls::ServerConfig as TlsServerConfig;

/// Порт сервера по умолчанию.
pub const DEFAULT_PORT: u16 = 7777;
/// Версия протокола (сервер и игра должны совпадать).
pub const PROTOCOL: u32 = 1;
/// Самая длинная строка, которую принимаем (файлы в редакторе бывают большими).
pub const MAX_LINE: usize = 4 * 1024 * 1024;

struct Inner {
    sock: TcpStream,
    tls: Option<Mutex<Connection>>,
    plain: Mutex<()>,
}

/// Отправитель строк.
#[derive(Clone)]
pub struct Writer(Arc<Inner>);

/// Приёмник строк.
pub struct Reader {
    inner: Arc<Inner>,
    buf: Vec<u8>,
}

fn other<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::new(ErrorKind::Other, e.to_string())
}

impl Writer {
    /// Отправить одну строку (перевод строки добавляется сам).
    pub fn send(&self, line: &str) -> io::Result<()> {
        let mut data = Vec::with_capacity(line.len() + 1);
        data.extend_from_slice(line.as_bytes());
        data.push(b'\n');
        match &self.0.tls {
            None => {
                let _g = self.0.plain.lock().map_err(other)?;
                (&self.0.sock).write_all(&data)?;
                (&self.0.sock).flush()
            }
            Some(conn) => {
                let mut c = conn.lock().map_err(other)?;
                c.writer().write_all(&data)?;
                while c.wants_write() {
                    c.write_tls(&mut &self.0.sock)?;
                }
                Ok(())
            }
        }
    }

    /// Закрыть соединение (приёмник на той стороне получит конец).
    pub fn close(&self) {
        if let Some(conn) = &self.0.tls {
            if let Ok(mut c) = conn.lock() {
                c.send_close_notify();
                while c.wants_write() {
                    if c.write_tls(&mut &self.0.sock).is_err() {
                        break;
                    }
                }
            }
        }
        let _ = self.0.sock.shutdown(Shutdown::Both);
    }

    /// Адрес собеседника.
    pub fn peer(&self) -> String {
        self.0.sock.peer_addr().map(|a| a.to_string()).unwrap_or_default()
    }
}

impl Reader {
    /// Следующая строка; `None` — соединение закрыто.
    pub fn next_line(&mut self) -> io::Result<Option<String>> {
        let mut tmp = vec![0u8; 16 * 1024];
        loop {
            if let Some(pos) = self.buf.iter().position(|b| *b == b'\n') {
                let rest = self.buf.split_off(pos + 1);
                let mut line = std::mem::replace(&mut self.buf, rest);
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
            }
            if self.buf.len() > MAX_LINE {
                return Err(other("слишком длинная строка"));
            }
            match &self.inner.tls {
                None => {
                    let n = (&self.inner.sock).read(&mut tmp)?;
                    if n == 0 {
                        return Ok(None);
                    }
                    self.buf.extend_from_slice(&tmp[..n]);
                }
                Some(conn) => {
                    // сперва то, что уже расшифровано
                    {
                        let mut c = conn.lock().map_err(other)?;
                        match c.reader().read(&mut tmp) {
                            Ok(0) => return Ok(None),
                            Ok(n) => {
                                self.buf.extend_from_slice(&tmp[..n]);
                                continue;
                            }
                            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
                            Err(e) => return Err(e),
                        }
                    }
                    // потом новые байты из сети — без замка
                    let n = (&self.inner.sock).read(&mut tmp)?;
                    if n == 0 {
                        return Ok(None);
                    }
                    let mut c = conn.lock().map_err(other)?;
                    let mut slice = &tmp[..n];
                    while !slice.is_empty() {
                        c.read_tls(&mut slice)?;
                        c.process_new_packets().map_err(other)?;
                    }
                    while c.wants_write() {
                        c.write_tls(&mut &self.inner.sock)?;
                    }
                }
            }
        }
    }
}

fn split(sock: TcpStream, tls: Option<Connection>) -> (Writer, Reader) {
    let inner = Arc::new(Inner { sock, tls: tls.map(Mutex::new), plain: Mutex::new(()) });
    (Writer(inner.clone()), Reader { inner, buf: Vec::new() })
}

fn handshake(conn: &mut Connection, sock: &TcpStream) -> io::Result<()> {
    sock.set_read_timeout(Some(Duration::from_secs(10)))?;
    while conn.is_handshaking() {
        conn.complete_io(&mut &*sock)?;
    }
    while conn.wants_write() {
        conn.write_tls(&mut &*sock)?;
    }
    sock.set_read_timeout(None)?;
    Ok(())
}

/// Разобрать адрес: `хост[:порт]`, `tcp://хост[:порт]` (без шифрования),
/// `tls://хост[:порт]`. Возвращает (хост, порт, шифровать ли).
pub fn parse_addr(addr: &str) -> (String, u16, bool) {
    let a = addr.trim();
    let (rest, tls) = if let Some(r) = a.strip_prefix("tcp://") {
        (r, false)
    } else if let Some(r) = a.strip_prefix("tls://") {
        (r, true)
    } else {
        (a, true)
    };
    let rest = rest.trim_end_matches('/');
    match rest.rsplit_once(':') {
        Some((h, p)) if !h.is_empty() && p.parse::<u16>().is_ok() => (h.trim_matches(|c| c == '[' || c == ']').to_string(), p.parse().unwrap_or(DEFAULT_PORT), tls),
        _ => (rest.to_string(), DEFAULT_PORT, tls),
    }
}

fn client_config() -> Arc<ClientConfig> {
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let cfg = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("протоколы TLS")
        .with_root_certificates(roots)
        .with_no_client_auth();
    Arc::new(cfg)
}

/// Подключиться к серверу.
pub fn connect(addr: &str) -> io::Result<(Writer, Reader)> {
    let (host, port, tls) = parse_addr(addr);
    if host.is_empty() {
        return Err(other("пустой адрес"));
    }
    let mut last = other("адрес не найден");
    let mut sock = None;
    for sa in (host.as_str(), port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&sa, Duration::from_secs(8)) {
            Ok(s) => {
                sock = Some(s);
                break;
            }
            Err(e) => last = e,
        }
    }
    let sock = sock.ok_or(last)?;
    sock.set_nodelay(true)?;
    sock.set_write_timeout(Some(Duration::from_secs(20)))?;
    if !tls {
        return Ok(split(sock, None));
    }
    let name = ServerName::try_from(host.clone()).map_err(other)?;
    let mut conn = Connection::Client(ClientConnection::new(client_config(), name).map_err(other)?);
    handshake(&mut conn, &sock)?;
    Ok(split(sock, Some(conn)))
}

/// Настройка TLS сервера из PEM-файлов сертификата (цепочки) и ключа.
pub fn server_config(cert: &str, key: &str) -> io::Result<Arc<ServerConfig>> {
    let chain: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(cert)
        .map_err(other)?
        .collect::<Result<_, _>>()
        .map_err(other)?;
    let key = PrivateKeyDer::from_pem_file(key).map_err(other)?;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let cfg = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(other)?
        .with_no_client_auth()
        .with_single_cert(chain, key)
        .map_err(other)?;
    Ok(Arc::new(cfg))
}

/// Принять подключение на сервере (`tls` = None — без шифрования).
pub fn accept(sock: TcpStream, tls: Option<Arc<ServerConfig>>) -> io::Result<(Writer, Reader)> {
    sock.set_nodelay(true)?;
    sock.set_write_timeout(Some(Duration::from_secs(20)))?;
    match tls {
        None => Ok(split(sock, None)),
        Some(cfg) => {
            let mut conn = Connection::Server(ServerConnection::new(cfg).map_err(other)?);
            handshake(&mut conn, &sock)?;
            Ok(split(sock, Some(conn)))
        }
    }
}

/// Задать молчание, после которого чтение обрывается (сервер — от зависших).
pub fn set_idle_timeout(r: &Reader, secs: u64) {
    let _ = r.inner.sock.set_read_timeout(Some(Duration::from_secs(secs)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addr_forms() {
        assert_eq!(parse_addr("game.example:9000"), ("game.example".into(), 9000, true));
        assert_eq!(parse_addr("tcp://10.0.0.5"), ("10.0.0.5".into(), DEFAULT_PORT, false));
        assert_eq!(parse_addr("example"), ("example".into(), DEFAULT_PORT, true));
    }

    #[test]
    fn plain_round_trip() {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let t = std::thread::spawn(move || {
            let (s, _) = l.accept().unwrap();
            let (w, mut r) = accept(s, None).unwrap();
            let line = r.next_line().unwrap().unwrap();
            w.send(&format!("эхо {}", line)).unwrap();
        });
        let (w, mut r) = connect(&format!("tcp://127.0.0.1:{}", port)).unwrap();
        w.send("привет").unwrap();
        assert_eq!(r.next_line().unwrap().unwrap(), "эхо привет");
        t.join().unwrap();
    }
}
