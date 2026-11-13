//! Derived display data only. Sorting/registration and saved slot order are never changed here.
extern crate alloc;
use alloc::vec::Vec;
use crate::model::{List,Recipe,Material,CAPACITY};
pub struct Cache{recipes:[Option<Recipe>;CAPACITY],initialized:bool,dirty_stock:bool,known:bool,ready:[Option<bool>;CAPACITY],totals:Vec<Material>,related:Vec<(u16,u64)>}
impl Default for Cache{fn default()->Self{Self{recipes:[None;CAPACITY],initialized:false,dirty_stock:true,known:false,ready:[None;CAPACITY],totals:Vec::new(),related:Vec::new()}}}
impl Cache{
 /// Call after changing inventory quantities. Availability changes are detected by refresh too.
 pub fn inventory_changed(&mut self){self.dirty_stock=true;}
 /// Returns true only when derived quantities/readiness actually need recalculation.
 pub fn refresh(&mut self,list:&List,stock:&[u32;4096],known:bool,provisions:&[(u16,u16,[u16;3])])->bool{
  let changed=!self.initialized||list.slots.iter().enumerate().any(|(i,e)|e.map(|e|e.recipe)!=self.recipes[i]);
  if changed{
   self.recipes=core::array::from_fn(|i|list.slots[i].map(|e|e.recipe));self.totals=list.totals();self.related.clear();
   for material in &self.totals{let mut slots=0;for (slot,r) in self.recipes.iter().enumerate(){if r.is_some_and(|r|r.materials.iter().any(|m|m.id==material.id&&m.count>0)){slots|=1u64<<slot;}}self.related.push((material.id,slots));}
  }
  if !changed&&!self.dirty_stock&&known==self.known{return false}
  self.ready=core::array::from_fn(|i|self.recipes[i].and_then(|r|known.then(||r.ready(stock,provisions))));
  self.totals.sort_by_key(|m|(known&&stock[m.id as usize]>=m.count,m.id));self.initialized=true;self.known=known;self.dirty_stock=false;true
 }
 pub fn totals(&self)->&[Material]{&self.totals}
 pub fn ready(&self,slot:usize,recipe:Recipe)->Option<bool>{if self.recipes.get(slot).copied().flatten()==Some(recipe){self.ready[slot]}else{None}}
 pub fn related(&self,id:u16)->u64{self.related.iter().find(|r|r.0==id).map_or(0,|r|r.1)}
}
