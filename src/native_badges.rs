//! Native icon anchors are read from the game's own world matrices. No GUI or save
//! memory is modified. Bindings are bounded, and visibility/owner/layout are rechecked.
use alloc::vec::Vec;
use core::{cell::UnsafeCell,sync::atomic::{AtomicBool,AtomicU32,Ordering}};
use crate::{native::{rd,valid,visible},model::{List,Recipe,Material},badge_model::Gear};
const LIST_VTABLE:u32=0x176e8ac;
const HUNTER_VTABLE:u32=0x177d988;
const PALICO_VTABLE:u32=0x177dfb0;
static LISTS:[AtomicU32;4]=[const{AtomicU32::new(0)};4];
static HUNTER:AtomicU32=AtomicU32::new(0);
#[derive(Clone,Copy,Default)]
struct Binding{owner:u32,vtable:u32,layout:u32,object:u32,gear:Gear}
struct Cache{lock:AtomicBool,data:UnsafeCell<([Binding;256],usize)>}
unsafe impl Sync for Cache{}
impl Cache{fn with<R>(&self,f:impl FnOnce(&mut ([Binding;256],usize))->R)->R{while self.lock.compare_exchange_weak(false,true,Ordering::Acquire,Ordering::Relaxed).is_err(){core::hint::spin_loop()}let r=f(unsafe{&mut*self.data.get()});self.lock.store(false,Ordering::Release);r}}
const EMPTY:Binding=Binding{owner:0,vtable:0,layout:0,object:0,gear:Gear{kind:0,id:0,level_index:0}};
static CACHE:Cache=Cache{lock:AtomicBool::new(false),data:UnsafeCell::new(([EMPTY;256],0))};
unsafe fn half(p:u32)->u16{unsafe{core::ptr::read_unaligned(p as *const u16)}}
unsafe fn byte(p:u32)->u8{unsafe{core::ptr::read(p as *const u8)}}
unsafe fn float(p:u32)->f32{f32::from_bits(unsafe{rd(p)})}
pub fn observe_hunter(owner:u32){HUNTER.store(owner,Ordering::Release);}
#[forge::hook(offset=0x6f8e48)]
fn hunter_update(owner:u32)->u32{let result=original!(owner);if valid(owner){observe_hunter(owner);}result}
pub fn clear(){for a in &LISTS{a.store(0,Ordering::Release);}HUNTER.store(0,Ordering::Release);CACHE.with(|c|{c.0.fill(EMPTY);c.1=0;});}
fn list_parent(owner:u32,base:u32)->Option<(u32,bool)>{
 if !valid(owner)||unsafe{rd(owner)}!=base+LIST_VTABLE{return None}
 let setup=unsafe{rd(owner+0x50)};
 for (offset,vt,palico) in [(0x2d50,HUNTER_VTABLE,false),(0x11cc,PALICO_VTABLE,true)]{
  let Some(parent)=setup.checked_sub(offset) else{continue};
  if valid(parent)&&unsafe{rd(parent)}==base+vt{return Some((parent,palico))}
 }None
}
// Both hunter and Palico eight-row production/upgrade lists share this class.
#[forge::hook(offset=0x58b908)]
fn list_update(owner:u32)->u32{
 let result=original!(owner);
 if valid(owner){let mut found=false;for p in &LISTS{if p.load(Ordering::Relaxed)==owner{found=true;break}}
  if !found{for p in &LISTS{let old=p.load(Ordering::Relaxed);if old==0||!valid(old)||unsafe{rd(old)}!=unsafe{rd(owner)}||unsafe{byte(old+0x18)}!=1{p.store(owner,Ordering::Release);break}}}
 }result
}
// The equipment-box grid passes exact kind/ID/level even for repeated icon shapes.
#[forge::hook(offset=0x56ea38)]
fn grid_icon(owner:u32,group:u32,object:u32,equip:u32,a4:u32,a5:u32)->u32{
 let result=original!(owner,group,object,equip,a4,a5);
 if group!=12||object>=200||!valid(owner){return result}
 let gear=if valid(equip){Gear::from_native(unsafe{half(equip+4)},unsafe{half(equip+6)})}else{Gear::default()};
 let binding=Binding{owner,vtable:unsafe{rd(owner)},layout:unsafe{rd(owner+0x24)},object,gear};
 CACHE.with(|cache|{let index=cache.0.iter().position(|b|b.owner==owner&&b.object==object).unwrap_or_else(||{let n=cache.1;cache.1=(cache.1+1)%cache.0.len();n});cache.0[index]=binding;});result
}
// The base destructor runs before layout memory is released for both list and box classes.
#[forge::hook(offset=0x56da30)]
fn destroy_ui(owner:u32)->u32{for p in &LISTS{let _=p.compare_exchange(owner,0,Ordering::AcqRel,Ordering::Relaxed);}CACHE.with(|cache|{for b in &mut cache.0{if b.owner==owner{*b=EMPTY;}}});original!(owner)}
pub fn install(base:u32)->bool{
 if unsafe{rd(base+0x56da30)}!=0xe92d4010||unsafe{rd(base+0x6f8e48)}!=0xe92d4ff0||unsafe{rd(base+0x58b908)}!=0xe92d4ff0||unsafe{rd(base+0x56ea38)}!=0xe92d4bf0{return false}
 forge::install_hook!(base,destroy_ui);forge::install_hook!(base,hunter_update);forge::install_hook!(base,list_update);forge::install_hook!(base,grid_icon);true
}
fn object(layout:u32,group:u32,index:u32)->Option<u32>{
 if !valid(layout)||group>512||index>200{return None}
 let count=unsafe{rd(layout+0x298)};if count>512{return None}
 let (table,i)=if group<count{(unsafe{rd(layout+0xf8)},group)}else{(unsafe{rd(layout+0x294)},group-count)};
 if !valid(table){return None}let node=unsafe{rd(table+i*4)};if !valid(node)||unsafe{rd(node+0x54)}&1==0{return None}
 let table=unsafe{rd(node+0xd8)};if !valid(table){return None}let obj=unsafe{rd(table+index*4)};
 if !valid(obj)||unsafe{rd(obj+0x54)}&1==0{return None}
 // Ancestors carry the group visibility; do not leave badges on a hidden subpage.
 let mut parent=node;for _ in 0..32{if !valid(parent){return None}if unsafe{rd(parent+0x54)}&1==0{return None}let next=unsafe{rd(parent+0x68)};if next==0{break}parent=next;}
 Some(obj)
}
#[derive(Clone,Copy)]pub struct Marker{pub x:f32,pub y:f32}
fn anchor(layout:u32,group:u32,index:u32)->Option<Marker>{
 let obj=object(layout,group,index)?;let x=unsafe{float(obj+0x40)};let y=unsafe{float(obj+0x44)};
 if !x.is_finite()||!y.is_finite()||!(-64. ..=1344.).contains(&x)||!(-64. ..=784.).contains(&y){return None}
 Some(Marker{x:x+25.,y:y+1.})
}
fn hunter_registered(list:&List,parent:u32,row:u32)->bool{
 let p=parent+row*48;let kind=unsafe{half(parent+0x5c)} as u8;let kind=if kind==6{25}else{kind};let id=unsafe{half(p+0x28d6)};
 // Compare only the at-most-48 registrations, not the full equipment database every frame.
 list.slots.iter().flatten().any(|entry|{let r=entry.recipe;if r.kind!=kind||r.id!=id{return false}
 let level=unsafe{byte(p+0x28dd)};if r.level!=0&&r.level!=level+1{return false}if r.level==0&&level!=0{return false}
 if r.group!=unsafe{half(p+0x28da)}||r.points!=unsafe{byte(p+0x28de)} as u16{return false}
 let raw=Recipe{kind,id,materials:core::array::from_fn(|i|Material{id:unsafe{half(p+0x28e8+i as u32*4)},count:unsafe{byte(p+0x28ea+i as u32*4)} as u32}),..Recipe::default()};r.totals()==raw.totals()
 })
}
fn grid_owner(base:u32)->Option<u32>{
 let parent=HUNTER.load(Ordering::Acquire);if !valid(parent)||unsafe{rd(parent)}!=base+HUNTER_VTABLE||unsafe{rd(parent+4)}&255!=7{return None}
 let children=unsafe{rd(parent+0x34)};if !valid(children){return None}let pane=unsafe{rd(children+8*4)};if !visible(pane,base){return None}
 Some(pane)
}
pub fn markers(list:&List,base:u32)->Vec<Marker>{
 let mut out=Vec::new();
 for cached in &LISTS{let owner=cached.load(Ordering::Acquire);let Some((parent,palico))=list_parent(owner,base) else{continue};if !palico{HUNTER.store(parent,Ordering::Release);}if !visible(owner,base){continue}
  let layout=unsafe{rd(owner+0x24)};let setup=unsafe{rd(owner+0x50)};
  for row in 0..8{if unsafe{byte(setup+0x16+row*24)}==0{continue}
   let registered=if palico{let kind=unsafe{half(parent+0x10d2+row*24)} as u8;let id=unsafe{half(parent+0x10d4+row*24)};list.slots.iter().flatten().any(|e|e.recipe.kind==kind&&e.recipe.id==id&&e.recipe.level==0)}else{hunter_registered(list,parent,row)};
   if registered{if let Some(marker)=anchor(layout,7+row,1){out.push(marker);}}
  }
 }
 if let Some(owner)=grid_owner(base){CACHE.with(|cache|{for b in &cache.0{if b.owner==owner&&visible(b.owner,base)&&unsafe{rd(b.owner)}==b.vtable&&unsafe{rd(b.owner+0x24)}==b.layout&&b.gear.registered(list).is_some(){if let Some(marker)=anchor(b.layout,12,b.object){out.push(marker);}}}});}
 out
}
pub fn available()->bool{LISTS.iter().any(|p|p.load(Ordering::Relaxed)!=0)}

pub fn selected_registered(list:&List,base:u32)->Option<(u32,Recipe)>{
 let owner=grid_owner(base)?;let cursor=unsafe{rd(owner+0x58)};if !valid(cursor){return None}
 let x=unsafe{byte(cursor+0x14)} as u32;let y=unsafe{byte(cursor+0x15)} as u32;let cols=unsafe{byte(cursor+0x16)} as u32;let rows=unsafe{byte(cursor+0x17)} as u32;
 if x>=cols||y>=rows||cols*rows>100{return None}let object=2+x+cols*y;
 CACHE.with(|cache|cache.0.iter().find(|b|b.owner==owner&&b.object==object&&unsafe{rd(owner)}==b.vtable&&unsafe{rd(owner+0x24)}==b.layout).and_then(|b|b.gear.registered(list)).and_then(|slot|list.slots[slot]).map(|entry|(owner,entry.recipe)))
}

pub fn smithy_visible(base:u32)->bool{grid_owner(base).is_some()||LISTS.iter().any(|cached|{let owner=cached.load(Ordering::Acquire);visible(owner,base)&&list_parent(owner,base).is_some()})}
