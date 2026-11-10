//! The native +/START menu, not Forge's Insert overlay.
pub const QUEST_VTABLE:u32=0x177aeb8;
pub const LOBBY_VTABLE:u32=0x177e424;
pub fn is_main_menu(vtable:u32,phase:u32,substate:u32,pane_visible:bool)->bool{
 matches!(vtable,QUEST_VTABLE|LOBBY_VTABLE)&&matches!(phase,2|3)&&substate==0&&pane_visible
}
#[cfg(not(test))]
mod native{
 use super::*;
 use core::sync::atomic::{AtomicBool,AtomicU32,Ordering};
 use crate::native::{rd,valid};
 static INSTALLED:AtomicBool=AtomicBool::new(false);
 static QUEST:AtomicU32=AtomicU32::new(0);
 static LOBBY:AtomicU32=AtomicU32::new(0);
 #[forge::hook(offset=0x66ed24)]
 fn update_start_menu(owner:u32)->u32{
  if valid(owner){let base=forge::mem::text_addr();let vtable=unsafe{rd(owner)}.wrapping_sub(base);
   match vtable{QUEST_VTABLE=>QUEST.store(owner,Ordering::Relaxed),LOBBY_VTABLE=>LOBBY.store(owner,Ordering::Relaxed),_=>{}}
  }
  original!(owner)
 }
 pub fn visible(base:u32)->bool{
  if !INSTALLED.load(Ordering::Acquire){return false}
  for pointer in [&QUEST,&LOBBY]{let owner=pointer.load(Ordering::Relaxed);if !valid(owner){continue}
   unsafe{
    let vt=rd(owner).wrapping_sub(base);if !matches!(vt,QUEST_VTABLE|LOBBY_VTABLE){continue}
    if !matches!(rd(owner+0x38),2|3)||rd(owner+0x48)!=0{continue}
    let panes=rd(owner+0x34);if !valid(panes){continue}let pane=rd(panes);if !valid(pane){continue}
    let visible=crate::native::visible(pane,base);
    if is_main_menu(vt,rd(owner+0x38),rd(owner+0x48),visible){return true}
   }
  }false
 }
 pub fn install(base:u32)->bool{
  // Both native vtables use this common updater; lobby's override calls it.
  if unsafe{rd(base+0x66ed24)}!=0xe92d4830{return false}
  if unsafe{rd(base+QUEST_VTABLE+0x24)}!=base+0x66ed24||unsafe{rd(base+LOBBY_VTABLE+0x24)}!=base+0x78f38c{return false}
  forge::install_hook!(base,update_start_menu);INSTALLED.store(true,Ordering::Release);true
 }
}
#[cfg(not(test))]pub use native::{install,visible};
