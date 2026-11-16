//! Optional KatSmithy integration. Its tree navigation reads Forge's input API,
//! which returns the host pad directly and never passes the game's input path.
//! While the wishlist owns input, its four input import slots answer "not held"
//! and centered sticks; buttons held when it closes stay withheld until released.
//! Each slot is wrapped once and never re-linked.
use core::{ptr,sync::atomic::{AtomicBool,AtomicU32,Ordering}};
use alloc::vec::Vec;
use crate::native::rd;
use crate::katsmithy_model::{self as model,INPUT_IMPORTS};
use forge::log::prelude::{info,warn};
static SCANNED:AtomicBool=AtomicBool::new(false);
static MODULE:AtomicU32=AtomicU32::new(0);
static MODULE_SIZE:AtomicU32=AtomicU32::new(0);
static OWNED:AtomicBool=AtomicBool::new(false);
static ORIGINAL:[AtomicU32;4]=[const{AtomicU32::new(0)};4];
static LATCH:[AtomicU32;2]=[const{AtomicU32::new(0)};2];
static INSTALLED:AtomicBool=AtomicBool::new(false);
// KatSmithy's data + bss, where its armor list and weapon tree records live.
// They are found by content (see katsmithy_model), never by fixed address.
static REGION:AtomicU32=AtomicU32::new(0);
static REGION_LEN:AtomicU32=AtomicU32::new(0);
// Region offset + 1 of the last record found (0: none) and frames until a rescan.
static ARMOR_AT:AtomicU32=AtomicU32::new(0);
static ARMOR_IDS:AtomicU32=AtomicU32::new(0);
static TREE_AT:AtomicU32=AtomicU32::new(0);
static ARMOR_WAIT:AtomicU32=AtomicU32::new(0);
static TREE_WAIT:AtomicU32=AtomicU32::new(0);
const RESCAN_CALLS:u32=120;
static STICK_LATCH:[AtomicBool;2]=[const{AtomicBool::new(false)};2];
#[repr(C)]#[derive(Default)]
struct Map{base:u64,size:u64,kind:u32,attr:u32,perm:u32,ipc:u32,device:u32,pad:u32}
// Horizon AArch32 QueryMemory; Recomp implements the same call.
#[unsafe(naked)]
unsafe extern "C" fn query(_info:*mut Map,_address:u32)->u32{
 core::arch::naked_asm!("mov r2, r1","svc #6","bx lr");
}
fn readable(address:u32,size:u32)->bool{
 let Some(end)=address.checked_add(size)else{return false};let mut at=address;
 while at<end{let mut m=Map::default();if unsafe{query(&mut m,at)}!=0||m.perm&1==0{return false}
 let next=m.base.saturating_add(m.size);if next<=at as u64{return false}at=next.min(end as u64) as u32;
 }true
}
fn writable(address:u32)->bool{let mut m=Map::default();let r=unsafe{query(&mut m,address)};r==0&&m.perm&3==3}
unsafe fn word(a:u32)->u32{unsafe{ptr::read_unaligned(a as *const u32)}}
/// Wishlist's input ownership, refreshed by the menu hook and every update.
pub fn set_owned(owned:bool){
 let was=OWNED.swap(owned,Ordering::AcqRel);
 if owned||!INSTALLED.load(Ordering::Acquire){return}
 let down:unsafe extern "C" fn(u32)->u32=unsafe{core::mem::transmute(original(0))};
 let held_of=|bits:u64|(0..model::BUTTONS).filter(|bit|bits&(1<<bit)!=0).fold(0u64,|held,bit|if unsafe{down(bit)}&0xff!=0{held|1<<bit}else{held});
 if was{
  // Snapshot what is still held: KatSmithy may first ask after we closed.
  store_latch(model::release_latch(load_latch(),held_of(u64::MAX)));
 }else{
  // Release from the host state every frame; never wait for KatSmithy to ask
  // about a button while it is up, or a latched B could stay withheld for good.
  let latch=load_latch();if latch!=0{store_latch(model::refresh_latch(latch,held_of(latch)));}
 }
 for (i,latch) in STICK_LATCH.iter().enumerate(){
  if !latch.load(Ordering::Acquire){continue}
  let call:unsafe extern "C" fn(*mut f32,*mut f32)=unsafe{core::mem::transmute(original(2+i))};
  let (mut x,mut y)=(0f32,0f32);unsafe{call(&mut x,&mut y)};
  let mut held=true;model::stick_blocked(x,y,false,&mut held);latch.store(held,Ordering::Release);
 }
}
fn load_latch()->u64{u64::from(LATCH[0].load(Ordering::Acquire))|(u64::from(LATCH[1].load(Ordering::Acquire))<<32)}
fn store_latch(v:u64){LATCH[0].store(v as u32,Ordering::Release);LATCH[1].store((v>>32) as u32,Ordering::Release);}
fn blocked(bit:u32,held:bool)->bool{
 let mut latch=load_latch();
 let blocked=model::button_blocked(bit,held,OWNED.load(Ordering::Acquire),&mut latch);
 store_latch(latch);blocked
}
/// KatSmithy's input imports are wrapped: its navigation takeover then has nothing
/// to refill the menu word with while the wishlist is open.
pub fn input_wrapped()->bool{INSTALLED.load(Ordering::Acquire)}

fn original(index:usize)->u32{ORIGINAL[index].load(Ordering::Acquire)}
unsafe extern "C" fn is_down(bit:u32)->u32{
 let call:unsafe extern "C" fn(u32)->u32=unsafe{core::mem::transmute(original(0))};
 let down=unsafe{call(bit)}&0xff!=0;
 u32::from(down&&!blocked(bit,down))
}
unsafe extern "C" fn is_pressed(bit:u32)->u32{
 let call:unsafe extern "C" fn(u32)->u32=unsafe{core::mem::transmute(original(1))};
 let pressed=unsafe{call(bit)}&0xff!=0;
 // The held state, from the same host frame, drives the shared latch.
 let down:unsafe extern "C" fn(u32)->u32=unsafe{core::mem::transmute(original(0))};
 let held=unsafe{down(bit)}&0xff!=0;
 u32::from(pressed&&!blocked(bit,held))
}
unsafe fn stick(index:usize,latch:&AtomicBool,x:*mut f32,y:*mut f32){
 let call:unsafe extern "C" fn(*mut f32,*mut f32)=unsafe{core::mem::transmute(original(index))};
 unsafe{call(x,y)};
 let read=|p:*mut f32|if p.is_null(){0.}else{unsafe{ptr::read_unaligned(p)}};
 let mut held=latch.load(Ordering::Acquire);
 let blocked=model::stick_blocked(read(x),read(y),OWNED.load(Ordering::Acquire),&mut held);
 latch.store(held,Ordering::Release);
 if blocked{for p in [x,y]{if !p.is_null(){unsafe{ptr::write_unaligned(p,0.)}}}}
}
unsafe extern "C" fn stick_left(x:*mut f32,y:*mut f32){unsafe{stick(2,&STICK_LATCH[0],x,y)}}
unsafe extern "C" fn stick_right(x:*mut f32,y:*mut f32){unsafe{stick(3,&STICK_LATCH[1],x,y)}}

fn discover(own:u32)->Option<(u32,u32,u32)>{
 let mut at=0u32;
 // QueryMemory skips unmapped gaps; never blindly dereference the address space.
 for _ in 0..16384{
  let mut m=Map::default();if unsafe{query(&mut m,at)}!=0{break}
  let next=m.base.saturating_add(m.size);if next<=at as u64{break}
  if m.perm&5==5&&m.base>=0x10000&&m.size>=0x80&&m.base<=u32::MAX as u64{
   let base=m.base as u32;
   if readable(base,0x80)&&unsafe{word(base+0x10)}==0x304f524e{
    let (ro,len,image,bss)=unsafe{(word(base+0x28),word(base+0x2c),word(base+0x18),word(base+0x38))};
    if let Some(total)=image.checked_add(bss){
     if !(base..base.saturating_add(total)).contains(&own)&&(32..=16*1024*1024).contains(&len)&&image<=total{
      if let Some(start)=base.checked_add(ro){if readable(start,len)&&model::identifies(unsafe{core::slice::from_raw_parts(start as *const u8,len as usize)}){return Some((base,image,total))}}
     }
    }
   }
  }
  if next>=0x1_0000_0000{break}at=next as u32;
 }
 None
}

/// Once, after every plugin was loaded and its imports bound.
pub fn install_once(){
 if SCANNED.swap(true,Ordering::AcqRel){return}
 let own=install_once as *const () as u32;
 let Some((base,image,total))=discover(own) else{info!("[wishlist] KatSmithy not detected");return};
 MODULE_SIZE.store(total,Ordering::Release);MODULE.store(base,Ordering::Release);
 info!("[wishlist] KatSmithy detected at {:08X}",base);
 // Records are located later by content in data + bss; no build-specific address.
 let data=unsafe{word(base+0x30)};
 if data<total&&readable(base+data,total-data){REGION.store(base+data,Ordering::Release);REGION_LEN.store(total-data,Ordering::Release);info!("[wishlist] KatSmithy data region {:X} bytes ready for list lookup",total-data);}
 else{warn!("[wishlist] KatSmithy data region unreadable; its lists keep the game's pins");}
 if !readable(base,image){warn!("[wishlist] KatSmithy image unreadable; input left untouched");return}
 let bytes=unsafe{core::slice::from_raw_parts(base as *const u8,image as usize)};
 let slots=model::import_slots(bytes,base,INPUT_IMPORTS);
 let detours=[is_down as *const () as u32,is_pressed as *const () as u32,stick_left as *const () as u32,stick_right as *const () as u32];
 // isPressed's latch also needs isDown; wrap all four or none.
 let mut targets=[0u32;4];
 for i in 0..4{
  let Some(slot)=slots[i] else{warn!("[wishlist] KatSmithy import {} not found; input left untouched",i);return};
  let target=unsafe{word(base+slot)};
  let inside=|a:u32,start:u32,len:u32|a.wrapping_sub(start)<len;
  if !writable(base+slot){warn!("[wishlist] KatSmithy import {} slot is read-only; input left untouched",i);return}
  if target<0x10000||target&3!=0||inside(target,base,image)||detours.contains(&target){
   warn!("[wishlist] KatSmithy import {} slot holds {:08X}; input left untouched",i,target);return
  }
  targets[i]=target;
 }
 for i in 0..4{
  ORIGINAL[i].store(targets[i],Ordering::Release);
  unsafe{ptr::write_volatile((base+slots[i].unwrap_or(0)) as *mut u32,detours[i])};
 }
 INSTALLED.store(true,Ordering::Release);
 info!("[wishlist] KatSmithy input imports wrapped: input withheld while the wishlist is open");
}

fn region()->Option<&'static [u8]>{
 let (at,len)=(REGION.load(Ordering::Acquire),REGION_LEN.load(Ordering::Acquire));
 (at!=0).then(||unsafe{core::slice::from_raw_parts(at as *const u8,len as usize)})
}
/// The strict content search (`find`) runs until the record is found, at most
/// every RESCAN_CALLS. KatSmithy's records are static globals, so afterwards only
/// the light per-frame check (`live`) runs; a frame where it fails (a list being
/// rebuilt) just draws nothing, without searching again.
fn locate<T>(m:&[u8],cache:&AtomicU32,wait:&AtomicU32,live:impl Fn(&[u8],usize)->Option<T>,find:impl Fn(&[u8])->Option<usize>)->Option<T>{
 let at=cache.load(Ordering::Acquire);
 if at!=0{return live(m,at as usize-1)}
 let left=wait.load(Ordering::Acquire);if left>0{wait.store(left-1,Ordering::Release);return None}
 wait.store(RESCAN_CALLS,Ordering::Release);
 let found=find(m)?;cache.store(found as u32+1,Ordering::Release);live(m,found)
}
fn hunter_list(base:u32,kinds:core::ops::RangeInclusive<u16>)->Option<u16>{
 let window=crate::native_badges::open_hunter(base)?;let kind=unsafe{half(window+0x5c)};
 (unsafe{rd(window+4)}&0xff==3&&kinds.contains(&kind)).then_some(kind)
}
unsafe fn half(a:u32)->u16{unsafe{ptr::read_unaligned(a as *const u16)}}

/// KatSmithy's armor list while it replaces the hunter smithy's armor production list.
#[derive(Clone,Copy)]
pub struct ArmorView{state:model::ArmorState}
pub fn armor_view(base:u32)->Option<ArmorView>{
 hunter_list(base,1..=5)?;let m=region()?;
 let find=|m:&[u8]|{let at=model::find_armor(m)?;ARMOR_IDS.store(model::armor_at(m,at)?.ids as u32,Ordering::Release);Some(at)};
 let live=|m:&[u8],at:usize|model::armor_live(m,at,ARMOR_IDS.load(Ordering::Acquire) as usize);
 locate(m,&ARMOR_AT,&ARMOR_WAIT,live,find).map(|state|ArmorView{state})
}
fn registered(list:&crate::model::List,kind:u8,id:u16)->bool{
 list.slots.iter().flatten().any(|e|e.recipe.kind==kind&&e.recipe.id==id&&e.recipe.level==0)
}
/// Pins on KatSmithy's rows for registered pieces, in its 1280x720 space.
pub fn armor_pins(view:ArmorView,list:&crate::model::List)->Vec<(f32,f32)>{
 let mut out=Vec::new();let Some(m)=region() else{return out};let s=view.state;
 for row in model::visible_rows(s.top,s.count){
  let Some(id)=model::armor_id(m,s,row) else{continue};
  for piece in 0..5u32{if registered(list,piece as u8+1,id){if let Some(p)=model::armor_pin(s.top,row,piece){out.push(p);}}}
 }
 out
}
/// The piece under KatSmithy's cursor. The game's own selection does not follow
/// pieces that cannot be crafted yet, so this one is the registration target.
pub fn armor_selection(view:ArmorView)->Option<crate::model::Recipe>{
 let s=view.state;let id=model::armor_id(region()?,s,s.row)?;let kind=s.piece as u8+1;
 crate::generated::RECIPES.iter().find(|r|(r.kind==kind||r.kind==0)&&r.id==id&&r.level==0).map(|r|crate::model::Recipe{kind,..*r})
}

/// KatSmithy's weapon tree while it replaces the hunter smithy's weapon list.
#[derive(Clone,Copy)]
pub struct WeaponView{tree:model::TreeState,kind:u8,scroll_x:f32,scroll_y:f32}
fn tree(base:u32)->Option<(&'static [u8],model::TreeState,u16)>{
 let kind=hunter_list(base,7..=21)?;let m=region()?;
 Some((m,locate(m,&TREE_AT,&TREE_WAIT,model::tree_live,model::find_tree)?,kind))
}
/// KatSmithy's material selection dialog over its weapon tree: the wishlist then
/// shows no pins or shortcut bars and takes no registration.
pub fn tree_dialog(base:u32)->bool{
 tree(base).is_some_and(|(m,t,_)|model::tree_u32(m,t,model::TREE_DIALOG)==Some(1))
}
pub fn weapon_view(base:u32)->Option<WeaponView>{
 let (m,tree,kind)=tree(base)?;
 if model::tree_u32(m,tree,model::TREE_MODE)?!=1||model::tree_u32(m,tree,model::TREE_DIALOG)?==1{return None}
 let scroll_x=f32::from_bits(model::tree_u32(m,tree,model::TREE_SCROLL_X)?);let scroll_y=f32::from_bits(model::tree_u32(m,tree,model::TREE_SCROLL_Y)?);
 // Scrolling stays within the tree's own extent; anything else is not a scroll value.
 let ok=|v:f32,limit:f32|v.is_finite()&&(-1.0..=limit).contains(&v);
 (ok(scroll_x,64.*54.*1.25)&&ok(scroll_y,(model::TREE_MAX_ROWS as f32+4.)*18.*1.25)).then_some(WeaponView{tree,kind:kind as u8,scroll_x,scroll_y})
}
/// Pins on KatSmithy's tree nodes for registered weapons, in its 1280x720 space.
pub fn weapon_pins(view:WeaponView,list:&crate::model::List)->Vec<(f32,f32)>{
 let mut out=Vec::new();let Some(m)=region() else{return out};
 for i in 0..view.tree.rows{
  let Some((id,col,row))=model::tree_line(m,view.tree,i) else{continue};
  for e in list.slots.iter().flatten(){
   if e.recipe.kind!=view.kind||e.recipe.id!=id{continue}
   if let Some(p)=model::weapon_pin(col,row,model::weapon_node_level(e.recipe.level),view.tree.columns,view.scroll_x,view.scroll_y){out.push(p);}
  }
 }
 out
}
/// The weapon under KatSmithy's cursor as (kind, line id, node level), when it
/// is one of the displayed lines.
pub fn weapon_selection(view:WeaponView)->Option<(u8,u16,u32)>{
 let m=region()?;let id=model::tree_u32(m,view.tree,model::TREE_SELECTED_ID)?;let level=model::tree_u32(m,view.tree,model::TREE_SELECTED_LEVEL)?;
 if !(1..=20).contains(&level)||!(0..view.tree.rows).any(|i|model::tree_line(m,view.tree,i).is_some_and(|l|u32::from(l.0)==id)){return None}
 Some((view.kind,id as u16,level))
}
/// A recipe for a tree node: level 1 is forged (level 0) where possible.
pub fn weapon_recipe(kind:u8,id:u16,level:u32)->Option<crate::model::Recipe>{
 let at=|l:u32|crate::generated::RECIPES.iter().find(|r|r.kind==kind&&r.id==id&&u32::from(r.level)==l).copied();
 if level==1{at(0).or_else(||at(1))}else{at(level)}
}
