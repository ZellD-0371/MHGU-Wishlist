//! Chain existing input import wrappers, including Better-Skillinfo's filter.
use core::{ptr,sync::atomic::{AtomicU32,Ordering}};
use crate::{STATE,native::rd};
pub const DETAIL:u32=1024;pub const TAB_PREV:u32=2048;pub const TAB_NEXT:u32=4096;
pub const PAD_REGISTER:u32=8192;pub const PAD_LIST:u32=16384;
pub const OPEN_REGISTER:u32=1;pub const OPEN_LIST:u32=2;pub const ACCEPT:u32=4;pub const BACK:u32=8;pub const UP:u32=16;pub const DOWN:u32=32;pub const PREV:u32=64;pub const NEXT:u32=128;pub const DELETE:u32=256;pub const TAB:u32=512;
const GOT:[u32;5]=[0x1838494,0x1838498,0x183849c,0x18384a0,0x18384a4];
const PLT:[u32;5]=[0x13f0af0,0x13f0afc,0x13f0b08,0x13f0b14,0x13f0b20];
static ORIGINAL:[AtomicU32;5]=[const{AtomicU32::new(0)};5];
static HELD:AtomicU32=AtomicU32::new(0);
static RELEASE:AtomicU32=AtomicU32::new(0);
unsafe fn filter(index:usize,state:u32,id:u32){
 let address=ORIGINAL[index].load(Ordering::Acquire);let call:unsafe extern "C" fn(u32,u32)=unsafe{core::mem::transmute(address)};unsafe{call(state,id)};
 if state<0x10000||unsafe{rd(state+32)}&1==0{return}
 let buttons=unsafe{rd(state+8)};let pressed=buttons&!HELD.swap(buttons,Ordering::AcqRel);
 if forge::graphics::Graphics::is_menu_visible(){return}
 let captured=STATE.with(|s|{
  let open=s.mode!=0;
  if s.game_ready&&s.profile.is_some()&&s.installed&&s.input_ready&&s.loaded{
   if !open&&pressed&(1<<5)!=0{s.events|=if buttons&(1<<4)!=0{if s.at_smithy()||crate::game_menu::visible(s.base){PAD_LIST}else{0}}else if s.active().is_some(){PAD_REGISTER}else{0};}
   if open{for (bit,event) in [(0,ACCEPT),(1,BACK),(2,DELETE),(6,TAB_PREV),(7,TAB_NEXT),(12,PREV),(13,UP),(14,NEXT),(15,DOWN),(8,DETAIL),(9,DETAIL)]{if pressed&(1<<bit)!=0{s.events|=event}}}
  }
  open||s.events&(PAD_REGISTER|PAD_LIST)!=0
 });
 let guard=RELEASE.load(Ordering::Acquire);let modal_mask=0x00ffffffu32;
 let mask=if captured{RELEASE.store(buttons&modal_mask,Ordering::Release);modal_mask}else{let held=buttons&guard;RELEASE.store(held,Ordering::Release);held};
 if mask!=0{unsafe{ptr::write_unaligned((state+8) as *mut u32,buttons&!mask);if captured{ptr::write_unaligned((state+12) as *mut u32,0);for off in [16,20,24,28]{ptr::write_unaligned((state+off) as *mut u32,0);}}}}
}
unsafe extern "C" fn full(s:u32,id:u32){unsafe{filter(0,s,id)}}
unsafe extern "C" fn dual(s:u32,id:u32){unsafe{filter(1,s,id)}}
unsafe extern "C" fn left(s:u32,id:u32){unsafe{filter(2,s,id)}}
unsafe extern "C" fn right(s:u32,id:u32){unsafe{filter(3,s,id)}}
unsafe extern "C" fn handheld(s:u32,id:u32){unsafe{filter(4,s,id)}}
pub fn install(base:u32)->bool{
 let detours=[full as *const (),dual as *const (),left as *const (),right as *const (),handheld as *const ()];
 for (i,offset) in GOT.iter().enumerate(){let mut address=unsafe{rd(base+offset)};
  if address==base+0x13efd44{
   #[repr(C,align(8))]struct Buffer([u8;40]);let mut state=Buffer([0;40]);let port=0;
   let bind:unsafe extern "C" fn(*mut Buffer,*const u32)=unsafe{core::mem::transmute(base+PLT[i])};unsafe{bind(&mut state,&port)};address=unsafe{rd(base+offset)};
  }
  if address<0x10000||address&3!=0||address==base+0x13efd44{return false}ORIGINAL[i].store(address,Ordering::Release);
 }
 for (i,offset) in GOT.iter().enumerate(){unsafe{ptr::write_volatile((base+offset) as *mut u32,detours[i] as u32)}}true
}
