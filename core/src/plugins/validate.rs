use super::*;
use crate::{World,missions::Goal,gigs::GigGoal};
pub fn world(game:&Game,w:&World)->Result<(),String>{
 let ids:BTreeSet<_>=w.machines.iter().map(|m|m.id).collect();
 let check=|id:&u32|if ids.contains(id){Ok(())}else{Err(trf!("ссылка на неизвестное устройство {id}", "reference to unknown device {id}"))};
 for id in w.player.known_machines.iter().chain(&w.player.scanned).chain(&w.player.owned).chain(&w.botnet).chain(&w.investigation.watched).chain(&w.fw_down){check(id)?;}
 for c in &w.player.found_creds{check(&c.machine)?;}
 for k in &w.keyloggers{check(&k.machine)?;if let Some(c)=&k.captured{check(&c.machine)?;}}
 for (id,_) in &w.down_until{check(id)?;}
 for relay in &w.base.rented{check(&relay.machine)?;}
 for m in &w.missions.missions {if m.reward<0||m.reward>1_000_000_000{return Err(tr!("неверная награда миссии", "invalid mission reward").into());}match &m.goal {Goal::ReadDeliver{target,..}|Goal::Own{target}|Goal::Plant{target,..}=>check(target)?,Goal::CoverTracks{heat_below}=>if !heat_below.is_finite()||*heat_below<0.0||*heat_below>100.0{return Err(tr!("неверный порог подозрения", "invalid heat threshold").into());}}}
 for gig in &w.gigs.gigs {if gig.reward<0||gig.reward>1_000_000_000{return Err(tr!("неверная награда подработки", "invalid gig reward").into());}match &gig.goal {GigGoal::Deliver{target,..}|GigGoal::Own{target}=>check(target)?}}
 if let Some(id)=w.investigation.lead_org{if !w.orgs.iter().any(|o|o.id==id){return Err(tr!("неизвестный орган расследования", "unknown investigating agency").into());}}
 if !w.player.heat.is_finite()||!(0.0..=100.0).contains(&w.player.heat)||!w.investigation.progress.is_finite()||!(0.0..=100.0).contains(&w.investigation.progress){return Err(tr!("подозрение и прогресс расследования: 0..100", "heat and investigation progress: 0..100").into());}
 if !(-1_000_000_000_000i64..=1_000_000_000_000i64).contains(&w.player.money)||w.finance.debt<0||w.finance.debt>1_000_000_000_000||!w.finance.rate_per_day.is_finite()||!(0.0..=100.0).contains(&w.finance.rate_per_day){return Err(tr!("неверное состояние экономики", "invalid economy state").into());}
 for m in &w.machines{
  if !m.users.exists("root"){return Err(trf!("у устройства {} должен быть root", "device {} must have root",m.id));}
  let mut ports=BTreeSet::new();for s in &m.services{if s.port==0||s.name.is_empty()||!ports.insert(s.port){return Err(tr!("порты устройства должны быть уникальными, 1..65535", "device ports must be unique, 1..65535").into());}}
  if let Some(t)=&m.device_type{if !w.plugins.device_types.contains_key(t){return Err(trf!("неизвестный тип {t}", "unknown type {t}"));}}
  let mut seen=BTreeSet::new();let mut current=Some(m.id);
  while let Some(id)=current {if !seen.insert(id){return Err(tr!("цикл маршрутизаторов", "router cycle").into());}current=w.machine(id).router.filter(|next|*next!=id);}
 }
 for session in game.sessions.values(){for hop in &session.hops{check(&hop.machine)?;if !w.machine(hop.machine).users.exists(&hop.user){return Err(tr!("удалён пользователь активного сеанса", "user of an active session was removed").into());}}}
 Ok(())
}
