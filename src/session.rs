//! Only explicit character selection activates a profile; bulk title save scans do not.
use core::sync::atomic::{AtomicU32,AtomicBool,Ordering};
static SLOT:AtomicU32=AtomicU32::new(u32::MAX);
static TITLE:AtomicBool=AtomicBool::new(true);
static QUIET:AtomicU32=AtomicU32::new(0);
// Continue's interactive sSavedata loader calls +3E0DB8 directly from
// +3E21B8 state 0x0E, then changes to state 0x11. +3E0D58 is a
// separate explicit reload helper and is not called by normal Continue.
fn completed_load(before:u32,after:u32,cancelled:bool,slot:u32)->Option<u32>{
 if before==0x0e&&after==0x11&&!cancelled&&slot<3{Some(slot)}else{None}
}
#[cfg(not(test))]
#[forge::hook(offset=0x3e21b8)]
fn select_character(owner:u32)->u32{
 let before=if crate::native::valid(owner){unsafe{crate::native::rd(owner+0x3c)}}else{u32::MAX};
 let result=original!(owner);
 if crate::native::valid(owner){unsafe{
  let after=crate::native::rd(owner+0x3c);let slot=crate::native::rd(owner+0x40);
  let cancelled=core::ptr::read((owner+0x6b) as *const u8)!=0;
  if let Some(slot)=completed_load(before,after,cancelled,slot){
   SLOT.store(slot,Ordering::Release);QUIET.store(0,Ordering::Release);
   forge::log::prelude::info!("[wishlist] Continue loaded character slot {}",slot+1);
  }
 }}result
}
#[cfg(not(test))]
#[forge::hook(offset=0x3e11a8)]
fn write_character(owner:u32,slot:u32)->u32{let result=original!(owner,slot);if slot<3{SLOT.store(slot,Ordering::Release);}result}
#[cfg(not(test))]
#[forge::hook(offset=0x67b544)]
fn title_update(owner:u32)->u32{
 TITLE.store(true,Ordering::Release);
 // Main menu / logos are earlier states than character selection.
 if crate::native::valid(owner)&&unsafe{core::ptr::read((owner+4) as *const u8)}<=0x11{SLOT.store(u32::MAX,Ordering::Release);}
 original!(owner)
}
pub fn update()->(Option<usize>,bool){
 let age=if TITLE.swap(false,Ordering::AcqRel){QUIET.store(0,Ordering::Release);0}else{{let n=QUIET.load(Ordering::Acquire).min(179)+1;QUIET.store(n,Ordering::Release);n}};
 let slot=SLOT.load(Ordering::Acquire);((slot<3).then_some(slot as usize),slot<3&&age>=90)
}
#[cfg(not(test))]
pub fn install(base:u32)->bool{
 for (offset,word) in [(0x3e21b8,0xe92d41f0),(0x3e11a8,0xe92d4800),(0x67b544,0xe92d47f0)]{if unsafe{crate::native::rd(base+offset)}!=word{return false}}
 forge::install_hook!(base,select_character);forge::install_hook!(base,write_character);forge::install_hook!(base,title_update);true
}

#[cfg(test)]
#[test]fn character_session_blocks_title_and_waits_for_game(){
 SLOT.store(u32::MAX,Ordering::Relaxed);TITLE.store(true,Ordering::Relaxed);
 for _ in 0..120{assert!(!update().1);}
 SLOT.store(1,Ordering::Relaxed);TITLE.store(true,Ordering::Relaxed);
 assert_eq!(update(),(Some(1),false));
 for _ in 0..89{assert_eq!(update(),(Some(1),false));}
 assert_eq!(update(),(Some(1),true));
 TITLE.store(true,Ordering::Relaxed);assert_eq!(update(),(Some(1),false));
 SLOT.store(u32::MAX,Ordering::Relaxed);assert_eq!(update(),(None,false));
}

#[cfg(test)]
#[test]fn continue_completion_uses_actual_load_state_and_rejects_preview_or_cancel(){
 for slot in 0..3{assert_eq!(completed_load(0x0e,0x11,false,slot),Some(slot));}
 for state in [0,0xa,0xb,0xc,0xd,0xf,0x10,0x11]{assert_eq!(completed_load(state,0x11,false,0),None);}
 assert_eq!(completed_load(0xe,0xe,false,0),None);
 assert_eq!(completed_load(0xe,0x11,true,0),None);
 assert_eq!(completed_load(0xe,0x11,false,3),None);
}
