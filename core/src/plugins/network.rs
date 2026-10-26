use super::*;
impl Game {
 pub fn plugin_network_receive(&mut self,packet:Value){
    if packet.to_string().len()>16384{return;}
    let Some(id)=packet["plugin"].as_str() else{return};
    let Some(p)=self.plugins.loaded.get(id).filter(|p|p.error.is_none()&&p.manifest.permissions.contains("network")&&!self.world.plugins.disabled.contains(id)).cloned() else{return};
    if let Some(handler)=p.manifest.events.get("on_network"){self.run_plugin(&p,handler,packet,None);}
 }
}
