//! Two visible notifications with independent lifetimes; remaining entries wait.
extern crate alloc;
use alloc::vec::Vec;
use crate::{model::Recipe,presentation::NOTICE_DURATION};
pub const VISIBLE_LIMIT:usize=2;
pub const TOP:f32=390.;pub const HEIGHT:f32=80.;pub const GAP:f32=8.;
pub fn top(slot:usize)->f32{TOP+slot.min(VISIBLE_LIMIT-1) as f32*(HEIGHT+GAP)}
#[derive(Clone,Copy)]pub struct Notice{pub recipe:Recipe,pub seconds:f32}
#[derive(Default)]pub struct Queue{entries:Vec<Notice>}
impl Queue{
 pub fn push(&mut self,recipe:Recipe){if !self.entries.iter().any(|n|n.recipe==recipe){self.entries.push(Notice{recipe,seconds:NOTICE_DURATION});}}
 pub fn retain_registered(&mut self,list:&crate::model::List){self.entries.retain(|n|list.registered(n.recipe).is_some());}
 pub fn clear(&mut self){self.entries.clear();}
 pub fn is_empty(&self)->bool{self.entries.is_empty()}
 pub fn visible(&self)->impl Iterator<Item=(usize,&Notice)>{self.entries.iter().take(VISIBLE_LIMIT).enumerate()}
 pub fn visible_count(&self)->usize{self.entries.len().min(VISIBLE_LIMIT)}
 pub fn waiting_count(&self)->usize{self.entries.len().saturating_sub(VISIBLE_LIMIT)}
 pub fn advance(&mut self,dt:f32){
  if !dt.is_finite(){return}let dt=dt.clamp(0.,0.1);
  // Waiting entries do not spend their display time. Advance only entries that
  // were drawn, then promote waiting entries on the next rendered frame.
  for n in self.entries.iter_mut().take(VISIBLE_LIMIT){n.seconds-=dt;}
  self.entries.retain(|n|n.seconds>0.0001);
 }
}
pub struct Demo{pub queue:Queue,pub recipes:[Recipe;4],pub scenario:usize,pub elapsed:f32,second_launched:bool}
impl Demo{
 pub fn new(recipes:[Recipe;4],scenario:usize)->Self{let mut demo=Self{queue:Queue::default(),recipes,scenario:scenario.min(2),elapsed:0.,second_launched:false};demo.restart();demo}
 pub fn restart(&mut self){self.queue.clear();self.elapsed=0.;self.second_launched=false;
  let count=match self.scenario{0=>2,1=>1,_=>4};for recipe in self.recipes.iter().take(count){self.queue.push(*recipe);}
 }
 pub fn advance(&mut self,dt:f32){if !dt.is_finite(){return}let dt=dt.clamp(0.,0.1);self.queue.advance(dt);self.elapsed+=dt;
  if self.scenario==1&&!self.second_launched&&self.elapsed>=2.{self.queue.push(self.recipes[1]);self.second_launched=true;}
 }
}
