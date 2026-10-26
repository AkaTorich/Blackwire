use super::*;
use crate::{World,missions::Goal,gigs::GigGoal};
pub fn world(game:&Game,w:&World)->Result<(),String>{
 let ids:BTreeSet<_>=w.machines.iter().map(|m|m.id).collect();
 let check=|id:&u32|if ids.contains(id){Ok(())}else{Err(format!("ссылка на неизвестное устройство {id}"))};
 for id in w.player.known_machines.iter().chain(&w.player.scanned).chain(&w.player.owned).chain(&w.botnet).chain(&w.investigation.watched).chain(&w.fw_down){check(id)?;}
 for c in &w.player.found_creds{check(&c.machine)?;}
 for k in &w.keyloggers{check(&k.machine)?;if let Some(c)=&k.captured{check(&c.machine)?;}}
 for (id,_) in &w.down_until{check(id)?;}
 for relay in &w.base.rented{check(&relay.machine)?;}
 for m in &w.missions.missions {if m.reward<0||m.reward>1_000_000_000{return Err("неверная награда миссии".into());}match &m.goal {Goal::ReadDeliver{target,..}|Goal::Own{target}|Goal::Plant{target,..}=>check(target)?,Goal::CoverTracks{heat_below}=>if !heat_below.is_finite()||*heat_below<0.0||*heat_below>100.0{return Err("неверный порог подозрения".into());}}}
 for gig in &w.gigs.gigs {if gig.reward<0||gig.reward>1_000_000_000{return Err("неверная награда подработки".into());}match &gig.goal {GigGoal::Deliver{target,..}|GigGoal::Own{target}=>check(target)?}}
 if let Some(id)=w.investigation.lead_org{if !w.orgs.iter().any(|o|o.id==id){return Err("неизвестный орган расследования".into());}}
 if !w.player.heat.is_finite()||!(0.0..=100.0).contains(&w.player.heat)||!w.investigation.progress.is_finite()||!(0.0..=100.0).contains(&w.investigation.progress){return Err("подозрение и прогресс расследования: 0..100".into());}
 if !(-1_000_000_000_000i64..=1_000_000_000_000i64).contains(&w.player.money)||w.finance.debt<0||w.finance.debt>1_000_000_000_000||!w.finance.rate_per_day.is_finite()||!(0.0..=100.0).contains(&w.finance.rate_per_day){return Err("неверное состояние экономики".into());}
 for m in &w.machines{
  if !m.users.exists("root"){return Err(format!("у устройства {} должен быть root",m.id));}
  let mut ports=BTreeSet::new();for s in &m.services{if s.port==0||s.name.is_empty()||!ports.insert(s.port){return Err("порты устройства должны быть уникальными, 1..65535".into());}}
  if let Some(t)=&m.device_type{if !w.plugins.device_types.contains_key(t){return Err(format!("неизвестный тип {t}"));}}
  let mut seen=BTreeSet::new();let mut current=Some(m.id);
  while let Some(id)=current {if !seen.insert(id){return Err("цикл маршрутизаторов".into());}current=w.machine(id).router.filter(|next|*next!=id);}
 }
 for session in game.sessions.values(){for hop in &session.hops{check(&hop.machine)?;if !w.machine(hop.machine).users.exists(&hop.user){return Err("удалён пользователь активного сеанса".into());}}}
 Ok(())
}
