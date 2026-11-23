//! Native GU quest IDs and native GU item IDs only; no text matching at runtime.
use crate::model::{List,Recipe,CAPACITY};
const DATA:&[u8]=include_bytes!("../assets/quest-materials.bin");
fn word(data:&[u8],at:usize)->Option<usize>{Some(u32::from_le_bytes(data.get(at..at.checked_add(4)?)?.try_into().ok()?) as usize)}
pub fn items(data:&[u8],quest:u32)->Option<&[u8]>{
 if data.get(..4)!=Some(b"WLQM")||word(data,4)?!=1{return None}
 let n=word(data,8)?;let pool=12usize.checked_add(n.checked_mul(12)?)?;
 if pool>data.len(){return None}
 let(mut lo,mut hi)=(0,n);
 while lo<hi{let mid=(lo+hi)/2;let key=word(data,12+mid*12)?;if key<quest as usize{lo=mid+1}else{hi=mid}}
 if lo==n||word(data,12+lo*12)?!=quest as usize{return None}
 let offset=word(data,16+lo*12)?.checked_mul(2)?;let len=word(data,20+lo*12)?.checked_mul(2)?;
 let start=pool.checked_add(offset)?;data.get(start..start.checked_add(len)?)
}
/// Whether the quest can yield the item (rewards, carves or gathering; not told apart).
pub fn obtainable(quest:u32,id:u16)->bool{
 let Some(ids)=items(DATA,quest) else{return false};let(mut lo,mut hi)=(0,ids.len()/2);
 while lo<hi{let mid=(lo+hi)/2;let v=u16::from_le_bytes([ids[mid*2],ids[mid*2+1]]);if v<id{lo=mid+1}else{hi=mid}}
 lo<ids.len()/2&&u16::from_le_bytes([ids[lo*2],ids[lo*2+1]])==id
}
pub fn matches(data:&[u8],quest:u32,missing:&[u64;64])->bool{
 items(data,quest).is_some_and(|ids|ids.chunks_exact(2).any(|b|{let id=u16::from_le_bytes([b[0],b[1]]) as usize;id<4096&&missing[id/64]&(1u64<<(id%64))!=0}))
}
pub struct Cache{short_groups:[u16;CAPACITY],related_quest:u32,related_mask:u64,log_epoch:u32,logged:bool,recipes:[Option<Recipe>;CAPACITY],dirty:bool,known:bool,missing:[u64;64],ids:[u32;8],hits:[bool;8]}
impl Default for Cache{fn default()->Self{Self{short_groups:[0;CAPACITY],related_quest:0,related_mask:0,log_epoch:0,logged:false,recipes:[None;CAPACITY],dirty:true,known:false,missing:[0;64],ids:[0;8],hits:[false;8]}}}
impl Cache{
 pub fn invalidate(&mut self){self.dirty=true;}
 /// Short across all registrations (the same rule the quest list pins use).
 pub fn is_missing(&self,id:u16)->bool{self.known&&id<4096&&self.missing[id as usize/64]&(1u64<<(id%64))!=0}
 pub fn related(&mut self,quest:u32,provisions:&[(u16,u16,[u16;3])])->u64{
  if !self.known||quest==0{return 0}
  if self.related_quest==quest{return self.related_mask}
  let available=items(DATA,quest).unwrap_or(&[]);
  let acquire=|id:u16|{
   if id==0||id>=4096||self.missing[id as usize/64]&(1u64<<(id%64))==0{return false}
   let(mut lo,mut hi)=(0,available.len()/2);
   while lo<hi{let mid=(lo+hi)/2;let value=u16::from_le_bytes([available[mid*2],available[mid*2+1]]);if value<id{lo=mid+1}else{hi=mid}}
   lo<available.len()/2&&u16::from_le_bytes([available[lo*2],available[lo*2+1]])==id
  };
  self.related_mask=self.recipes.iter().enumerate().filter_map(|(i,r)|r.map(|r|(i,r))).fold(0,|mask,(i,r)|{
   let explicit=r.materials.iter().any(|m|m.count>0&&acquire(m.id));
   let category=r.group!=0&&self.short_groups.contains(&r.group)&&provisions.iter().any(|p|p.1>0&&p.2.contains(&r.group)&&acquire(p.0));
   if explicit||category{mask|(1u64<<i)}else{mask}
  });
  crate::diagnostics::resource_record(format_args!("quest equipment: quest={} related_slots={:012X}",quest,self.related_mask));
  self.related_quest=quest;self.related_mask
 }

 pub fn rows(&mut self,list:&List,stock:&[u32;4096],known:bool,ids:[u32;8],provisions:&[(u16,u16,[u16;3])])->[bool;8]{
  let changed=list.slots.iter().enumerate().any(|(i,e)|e.map(|e|e.recipe)!=self.recipes[i]);
  let refresh=changed||self.dirty||known!=self.known;
  if refresh{
   self.short_groups.fill(0);self.related_quest=0;
   self.missing.fill(0);self.known=known;self.dirty=false;
   self.recipes=core::array::from_fn(|i|list.slots[i].map(|e|e.recipe));
   if known{
    let totals=list.totals();
    // Short per gear: a material counts when one registration alone needs more than
    // the box holds (owning enough for each gear by itself is not short, even when
    // all of them together need more; the detail page notes that case separately).
    for r in self.recipes.iter().flatten(){for m in r.materials.iter().filter(|m|m.count>0&&m.id<4096){if stock[m.id as usize]<m.count{self.missing[m.id as usize/64]|=1u64<<(m.id%64);}}}
    // Category points are alternatives: mark their providers only if the points
    // left after paying explicit ingredients cannot cover all registrations.
    let mut groups:alloc::vec::Vec<(u16,u32)>=alloc::vec::Vec::new();
    for r in self.recipes.iter().flatten().filter(|r|r.group!=0&&r.points>0){if let Some(g)=groups.iter_mut().find(|g|g.0==r.group){g.1+=r.points as u32}else{groups.push((r.group,r.points as u32));}}
    for (group,needed) in groups{
     let value:u64=provisions.iter().filter(|p|p.2.contains(&group)&&p.0<4096).map(|p|{let used=totals.iter().find(|m|m.id==p.0).map_or(0,|m|m.count);u64::from(stock[p.0 as usize].saturating_sub(used))*p.1 as u64}).sum();
     if value<needed as u64{if let Some(slot)=self.short_groups.iter_mut().find(|g|**g==0){*slot=group;}for p in provisions.iter().filter(|p|p.0>0&&p.0<4096&&p.1>0&&p.2.contains(&group)){self.missing[p.0 as usize/64]|=1u64<<(p.0%64);}}
    }
   }
  }
  let result_changed=refresh||ids!=self.ids;
  if result_changed{self.ids=ids;self.hits=ids.map(|id|known&&id!=0&&matches(DATA,id,&self.missing));}
  if crate::diagnostics::enabled(){
   if result_changed||!self.logged||self.log_epoch!=crate::diagnostics::epoch(){
    let missing:alloc::vec::Vec<u16>=(1..4096).filter(|id|self.missing[id/64]&(1u64<<(id%64))!=0).map(|id|id as u16).collect();
    crate::diagnostics::resource_record(format_args!("quest material result: inventory_known={} quests={:?} hits={:?} missing_count={} missing_ids(first 64)={:?}",known,ids,self.hits,missing.len(),&missing[..missing.len().min(64)]));
   }
   self.log_epoch=crate::diagnostics::epoch();self.logged=true;
  }else{self.logged=false;}
  self.hits
 }
}
#[cfg(test)]mod tests{
 use super::*;use crate::model::{Material,Recipe};
 #[test]fn known_native_ids_and_rank_specific_monster_sources(){
  // Native 917 = Rathalos Topping; Database quest 277 is a different ID space.
  // Native item 318 = Iron Ore, 1203 = Rathalos Scale+, 1204 = Rathalos Shard.
  let has=|quest:u32,item:usize|{let mut bits=[0;64];bits[item/64]|=1u64<<(item%64);matches(DATA,quest,&bits)};
  assert!(has(101,318),"Jurassic Frontier low-rank mining");
  assert!(has(917,320),"Ruined Pinnacle high-rank Machalite Ore");
  assert!(has(917,1203),"high-rank Rathalos");
  assert!(!has(512,1203),"low-rank Rathalos must not get high-rank scales");
  assert!(!has(917,1204),"high-rank quest must not get G-rank shards");
 }
 #[test]fn related_equipment_uses_each_recipes_matching_deficit_and_refreshes(){
  let mut list=List::default();
  for (slot,id) in [(0,1203),(1,1204)]{list.insert(slot,Recipe{kind:7,id:slot as u16+1,materials:[Material{id,count:1},Material::default(),Material::default(),Material::default()],..Default::default()}).unwrap();}
  let mut c=Cache::default();let mut stock=[0;4096];c.rows(&list,&stock,true,[917;8],&[]);
  assert_eq!(c.related(917,&[]),1,"high-rank quest links only the scale+, not the G-rank shard gear");
  stock[1203]=1;c.invalidate();c.rows(&list,&stock,true,[917;8],&[]);assert_eq!(c.related(917,&[]),0);
 }
 #[test]fn native_ids_and_truncated_or_unknown_data(){
  assert!(items(DATA,917).is_some());assert!(items(DATA,0xffffffff).is_none());
  assert!(items(&DATA[..11],917).is_none());assert!(items(&DATA[..20],917).is_none());
 }
 #[test]fn deficits_are_per_gear_and_clear_when_stock_is_sufficient(){
  let id=u16::from_le_bytes(items(DATA,917).unwrap()[..2].try_into().unwrap());
  let mut list=List::default();for slot in 0..2{list.insert(slot,Recipe{kind:7,id:slot as u16+1,materials:[Material{id,count:2},Material::default(),Material::default(),Material::default()],..Default::default()}).unwrap();}
  let mut c=Cache::default();let mut stock=[0;4096];stock[id as usize]=1;
  assert!(c.rows(&list,&stock,true,[917;8],&[])[0]);
  // Enough for each gear alone (3 >= 2), though not for both together: not short.
  stock[id as usize]=3;c.invalidate();assert!(!c.rows(&list,&stock,true,[917;8],&[])[0]);
  assert!(!c.rows(&list,&stock,false,[917;8],&[])[0]);
  list.slots.fill(None);assert!(!c.rows(&list,&stock,true,[917;8],&[])[0]);
 }
 #[test]fn category_providers_are_marked_only_when_remaining_points_are_short(){
  let id=u16::from_le_bytes(items(DATA,917).unwrap()[..2].try_into().unwrap());
  let mut list=List::default();list.insert(0,Recipe{kind:7,id:1,group:1,points:8,..Default::default()}).unwrap();
  let mut c=Cache::default();let mut stock=[0;4096];let p=[(id,4,[1,0,0])];
  assert!(c.rows(&list,&stock,true,[917;8],&p)[0]);stock[id as usize]=2;c.invalidate();assert!(!c.rows(&list,&stock,true,[917;8],&p)[0]);
 }
 #[test]fn all_records_are_sorted_bounded_and_native_items_are_nonzero(){
  let n=word(DATA,8).unwrap();let mut last=0;
  for i in 0..n{let id=word(DATA,12+i*12).unwrap() as u32;assert!(id>last);last=id;let b=items(DATA,id).unwrap();let mut prev=0;for p in b.chunks_exact(2){let m=u16::from_le_bytes([p[0],p[1]]);assert!(m>prev&&m<2992);prev=m;}}
 }
}
