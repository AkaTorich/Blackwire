use super::*;
use crate::World;
pub struct Stamp {missions:usize,gigs:u32,ach:BTreeSet<String>,upgrades:BTreeSet<String>,bought:BTreeSet<String>,stage:String,scenario:bool,machines:usize,orgs:usize,mail:BTreeSet<u32>}
impl Stamp {pub fn new(w:&World)->Self{Self{missions:w.missions.active,gigs:w.gigs.done,ach:w.achievements.unlocked.clone(),upgrades:w.player.upgrades.clone(),bought:w.market.bought.clone(),stage:format!("{:?}",w.investigation.stage),scenario:w.scenarios.done,machines:w.machines.len(),orgs:w.orgs.len(),mail:w.machines.first().map(|m|m.mail.iter().map(|m|m.id).collect()).unwrap_or_default()}}}
impl Game {
 pub fn plugin_changes(&mut self,before:Stamp){
  if self.plugins.busy{return;}
  let mut events=Vec::new();
  let pending=std::mem::take(&mut self.world.plugin_events);
  for(name,payload)in pending.into_iter().take(64){self.plugin_event(&name,payload);}
  if self.world.missions.active!=before.missions{events.push(("on_mission_done",json!({"before":before.missions,"active":self.world.missions.active})));}
  if self.world.gigs.done!=before.gigs{events.push(("on_gig_done",json!({"before":before.gigs,"done":self.world.gigs.done})));}
  for id in self.world.achievements.unlocked.difference(&before.ach){events.push(("on_achievement",json!({"id":id})));}
  for id in self.world.player.upgrades.difference(&before.upgrades){events.push(("on_purchase",json!({"catalog":"shop","id":id})));}
  for id in self.world.market.bought.difference(&before.bought){events.push(("on_purchase",json!({"catalog":"market","id":id})));}
  if format!("{:?}",self.world.investigation.stage)!=before.stage{events.push(("on_investigation",json!({"before":before.stage,"stage":self.world.investigation.stage,"progress":self.world.investigation.progress})));}
  if self.world.scenarios.done&&!before.scenario{events.push(("on_scenario_done",json!({"scenario":self.world.scenarios.active})));}
  if self.world.machines.len()!=before.machines||self.world.orgs.len()!=before.orgs{events.push(("on_world",json!({"machines":self.world.machines.len(),"orgs":self.world.orgs.len()})));}
  if let Some(m)=self.world.machines.first(){for mail in &m.mail{if !before.mail.contains(&mail.id)&&mail.folder==crate::net::machine::MailFolder::Inbox{events.push(("on_mail",json!({"direction":"inbox","mail":mail})));}}}
  for(name,payload)in events{self.plugin_event(name,payload);}
 }
}
