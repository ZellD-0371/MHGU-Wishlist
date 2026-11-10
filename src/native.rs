//! MHGU global 1.4.0 adapter. Native pointers never enter the saved wishlist.
use core::{ptr,sync::atomic::{AtomicU32,Ordering}};
use crate::{STATE,model::Material,generated::RECIPES,smithy::SelectedRow};
static LAST_SELECTION:AtomicU32=AtomicU32::new(0);
pub fn valid(p:u32)->bool{(0x10000..0xeffff000).contains(&p)}
pub unsafe fn rd(p:u32)->u32{unsafe{ptr::read_unaligned(p as *const u32)}}
unsafe fn half(p:u32)->u16{unsafe{ptr::read_unaligned(p as *const u16)}}
unsafe fn byte(p:u32)->u8{unsafe{ptr::read(p as *const u8)}}
pub fn visible(owner:u32,base:u32)->bool{
 if !valid(owner){return false}
 let vt=unsafe{rd(owner)};
 vt>=base+0x1400000&&vt<base+0x1830000&&unsafe{byte(owner+0x18)}==1
}
// Hunter smithy updates this exact selected recipe, including alternate jewel recipes.
// This is separate from Better-Skillinfo's text/detail hooks.
#[forge::hook(offset=0x6fa3d8)]
fn smith_materials(owner:u32)->u32{
 let result=original!(owner);
 if !valid(owner){return result}
 crate::native_badges::observe_hunter(owner);
 let index=unsafe{byte(owner+0x1f71)} as u32;
 if index>=8{return result}
 let row=owner+index*48;
 let selected=SelectedRow{
  list_kind:unsafe{half(owner+0x5c)},id:unsafe{half(row+0x28d6)},
  level_index:unsafe{byte(row+0x28dd)},group:unsafe{half(row+0x28da)},points:unsafe{byte(row+0x28de)},
  materials:core::array::from_fn(|i|Material{id:unsafe{half(row+0x28e8+i as u32*4)},count:unsafe{byte(row+0x28ea+i as u32*4)} as u32}),
 };
 let resolved=selected.resolve(RECIPES,crate::generated::ARMOR_UPGRADES);
 let key=(selected.list_kind as u32).wrapping_mul(16777619)^(selected.id as u32).wrapping_mul(65537)^(selected.level_index as u32).wrapping_mul(257)^selected.group as u32;
 if selected.id!=0&&LAST_SELECTION.swap(key,Ordering::Relaxed)!=key{
  forge::log::prelude::info!("[wishlist] smithy list={} id={} level_index={} group={} points={} matched={}",selected.list_kind,selected.id,selected.level_index,selected.group,selected.points,resolved.is_some());
 }
 if let Some(r)=resolved{
  let base=STATE.with(|s|s.base);
  let get:unsafe extern "C" fn(u32,u32)->u32=unsafe{core::mem::transmute(base+0x65cb3c)};
  let pane=unsafe{get(owner,selected.pane())};
  if valid(pane){STATE.with(|s|s.capture(pane,r,unsafe{rd(row+0x28e4)}));}
 }else{
  // An unrecognized row must not leave the preceding equipment registerable.
  STATE.with(|s|s.target=None);
 }
 result
}
// Standard equipment setup also covers the three Palico production lists.
#[forge::hook(offset=0x57f468)]
fn equipment(owner:u32,equip:u32,compare:u32,a3:u32,a4:u32,flags:u32,a6:u32,a7:u32)->u32{
 let result=original!(owner,equip,compare,a3,a4,flags,a6,a7);
 if !valid(owner)||!valid(equip){return result}
 let kind=unsafe{half(equip+4)} as u8&31;
 if !(22..=24).contains(&kind)||unsafe{rd(owner+0x58)}!=1{return result}
 let id=unsafe{half(equip+6)};
 if let Some(r)=RECIPES.iter().find(|r|r.kind==kind&&r.id==id&&r.level==0){STATE.with(|s|s.capture(owner,*r,crate::equipment::cost(*r)));}
 result
}
// Inventory loading is a transaction. Do not emit false completion events midway through it.
#[forge::hook(offset=0x19a4ec)]
fn load_box(owner:u32,stream:u32)->u32{
 crate::completed::clear();crate::native_badges::clear();
 STATE.with(|s|{s.cooldown=180;s.target=None;s.mode=0;s.notices.clear();for e in s.list.slots.iter_mut().flatten(){e.ready=None}});
 original!(owner,stream)
}
pub fn install(base:u32)->bool{
 let signatures=[(0x6fa3d8,0xe92d4ff0),(0x57f468,0xe92d4ff0),(0x19a4ec,0xe92d48f0),(0x54fe8c,0xe92d4830)];
 if signatures.iter().any(|&(p,w)|unsafe{rd(base+p)}!=w){return false}
 forge::install_hook!(base,smith_materials);forge::install_hook!(base,equipment);forge::install_hook!(base,load_box);true
}
pub fn inventory(base:u32,stock:&mut [u32;4096])->bool{
 // Globals referenced by the game's own combined pouch+box counter (+54FE8C).
 let box_ptr=unsafe{rd(base+0x1858438)};let pouch=unsafe{rd(base+0x18584b8)};
 if !valid(box_ptr)||!valid(pouch){return false}
 // Read the decoded arrays directly. The box has at most 2300 slots; the pouch has 50.
 // Reading every allocated slot avoids calling the O(n) native counter for every item.
 stock.fill(0);
 for (p,count) in [(box_ptr,2300),(pouch,50)]{for i in 0..count{
  let id=unsafe{half(p+0x1a+i*4)} as usize;let n=unsafe{half(p+0x1c+i*4)} as u32;
  if id>=2992||n>99{return false}
  if id>0{stock[id]+=n}
 }}true
}
