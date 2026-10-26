//! Взлом — механики второго этапа (см. `docs/HACK.md`). Всё происходит только
//! в выдуманной игровой сети: это игровые действия над машинами модели мира.
//! Логика здесь; команды оболочки (`shell/cmds_hack.rs`) и функции языка
//! (`script_host.rs`) её только зовут и рисуют вывод.
//!
//! - `vuln`    — классификация слабых мест по версии службы и по ОС.
//! - `exploit` — получить доступ к удалённой машине с текущего hop'а.
//! - `crack`   — подобрать пароль по словарю (ssh).
//! - `privesc` — повысить права на текущей машине до root.
//! - `tracks`  — следы, чистка журналов, подозрение (heat).
//! - `proxy`   — цепочка подключений и выходной адрес.

pub mod vuln;
pub mod exploit;
pub mod crack;
pub mod privesc;
pub mod tracks;
pub mod proxy;
pub mod watch;
pub mod defense;
pub mod tools;
pub mod cipher;
pub mod counter;

pub use vuln::{access_from_str, local_vulns, remote_vulns, scan_vulns, VulnAccess, VulnInfo};
pub use exploit::{exploit, ExploitOutcome, ExploitReport};
pub use crack::{crack, CrackReport};
pub use privesc::{privesc, PrivescReport};
pub use tracks::{add_heat, clear_logs, decay, evidence, heat, is_monitored, source_ip, ClearReport};
pub use proxy::{proxy_info, ProxyInfo};
pub use watch::{is_locked, lock_remaining, take_pending, InvEvent, InvStage, Investigation};
