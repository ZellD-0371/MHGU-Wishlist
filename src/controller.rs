//! Chain existing input import wrappers, including Better-Skillinfo's filter.
use core::{ptr,sync::atomic::{AtomicU32,AtomicBool,Ordering}};
use crate::{STATE,native::rd};
pub const SORT:u32=32768;pub const DETAIL:u32=1024;pub const TAB_PREV:u32=2048;pub const TAB_NEXT:u32=4096;
pub const PAD_REGISTER:u32=8192;pub const PAD_LIST:u32=16384;
pub const OPEN_REGISTER:u32=1;pub const OPEN_LIST:u32=2;pub const ACCEPT:u32=4;pub const BACK:u32=8;pub const UP:u32=16;pub const DOWN:u32=32;pub const PREV:u32=64;pub const NEXT:u32=128;pub const DELETE:u32=256;pub const TAB:u32=512;
const GOT:[u32;5]=[0x1838494,0x1838498,0x183849c,0x18384a0,0x18384a4];
// Do not call an unresolved input import during plugin startup.
const UNRESOLVED:u32=0x13efd44;
static ORIGINAL:[AtomicU32;5]=[const{AtomicU32::new(0)};5];
static HELD:AtomicU32=AtomicU32::new(0);
static KEYBOARD_WAITING:AtomicBool=AtomicBool::new(false);
static INTERNAL_HELD:AtomicU32=AtomicU32::new(0);
static RELEASE:AtomicU32=AtomicU32::new(0);
static mut NATIVE_MODAL:crate::native_modal::Gate=crate::native_modal::Gate::new();
static SHOULDER_HELD:AtomicU32=AtomicU32::new(0);
static NAV_RELEASE:AtomicU32=AtomicU32::new(0);
unsafe fn filter(index:usize,state:u32,id:u32){
 let address=ORIGINAL[index].load(Ordering::Acquire);let call:unsafe extern "C" fn(u32,u32)=unsafe{core::mem::transmute(address)};unsafe{call(state,id)};
 if state<0x10000||unsafe{rd(state+32)}&1==0{return}
 let buttons=unsafe{rd(state+8)};
 let raw=crate::bindings::snapshot(buttons,unsafe{rd(state+16)} as i32,unsafe{rd(state+20)} as i32,unsafe{rd(state+24)} as i32,unsafe{rd(state+28)} as i32);
 crate::bindings::publish(raw);
 let keyboard=crate::bindings::keyboard_active();let raw=if keyboard{KEYBOARD_WAITING.store(true,Ordering::Release);0}else if KEYBOARD_WAITING.load(Ordering::Acquire){if raw==0{KEYBOARD_WAITING.store(false,Ordering::Release);}0}else{raw};
 let menu=forge::graphics::Graphics::is_menu_visible();let capture=crate::bindings::capturing();
 let captured=STATE.with(|s|{
  let active=crate::binding_model::active(raw,&s.list.pad_bindings);
  let pressed=active&!HELD.swap(active,Ordering::AcqRel);
  let open=s.mode!=0;
  if !menu&&!capture&&s.game_ready&&s.profile.is_some()&&s.installed&&s.input_ready&&s.loaded{
   if !open{
    if pressed&1!=0&&(s.at_smithy()||crate::game_menu::visible(s.base)){s.events|=PAD_LIST;}
    else if pressed&2!=0&&s.active().is_some(){s.events|=PAD_REGISTER;}
   }
  }
  
  if menu{false}else{capture||open||s.events&(PAD_REGISTER|PAD_LIST)!=0}
 });
 crate::diagnostics::event(1,"pad input: raw / guarded / keyboard blocked / modal",[buttons,raw,u32::from(keyboard),u32::from(captured)]);
 if !crate::button_prompts::is_recomp(){
  // Capture at HID level even if focus changes the game's active menu path.
  let (pressed,blocked)=unsafe{(&mut *core::ptr::addr_of_mut!(NATIVE_MODAL)).step(raw,captured,menu)};
  if captured&&!capture&&!menu{STATE.with(|s|{
   if s.mode!=0{
    for(mask,event)in [(1,ACCEPT),(2,BACK),(4,DELETE),(8,SORT),((1<<12)|(1<<16),PREV),((1<<13)|(1<<17),UP),((1<<14)|(1<<18),NEXT),((1<<15)|(1<<19),DOWN)]{
     if pressed&mask!=0{s.events|=event;}
    }
    let (left,right,detail)=crate::menu_buttons::shoulders(pressed,false);
    if left{s.events|=TAB_PREV;}if right{s.events|=TAB_NEXT;}if detail{s.events|=DETAIL;}
   }
  });}
  if blocked!=0{unsafe{
   ptr::write_unaligned((state+8) as *mut u32,buttons&!blocked);
   if blocked==u32::MAX{ptr::write_unaligned((state+12) as *mut u32,0);}
   if blocked&0xf0000!=0{ptr::write_unaligned((state+16) as *mut u32,0);ptr::write_unaligned((state+20) as *mut u32,0);}
   if blocked&0xf00000!=0{ptr::write_unaligned((state+24) as *mut u32,0);ptr::write_unaligned((state+28) as *mut u32,0);}
  }}
 }
}
fn owns_input(overlay:bool,capture:bool)->bool{!overlay&&STATE.with(|s|s.mode!=0||capture||s.events&(PAD_REGISTER|PAD_LIST)!=0)}
/// The wishlist owns input while open, capturing a binding or about to open.
pub fn publish_ownership(){
 crate::katsmithy::set_owned(owns_input(forge::graphics::Graphics::is_menu_visible(),crate::bindings::capturing()));
}
// Observe final menu actions after the host's optional action remapping.
#[forge::hook(offset=0x3de4f8)]
fn menu_translate(object:u32,word:u32)->u32{
 let overlay=forge::graphics::Graphics::is_menu_visible();
 let capture=crate::bindings::capturing();
 let recomp=crate::button_prompts::is_recomp();
 // Decide ownership before calling inward: hooks below us act on what they receive.
 let owned=owns_input(overlay,capture);crate::katsmithy::set_owned(owned);
 let mut latch=NAV_RELEASE.load(Ordering::Acquire);
 let inner=crate::input_chain::inner_word(word,owned,&mut latch);
 NAV_RELEASE.store(latch,Ordering::Release);
 let mapped=original!(object,inner);
 let mapped=if owned{crate::input_chain::owned_actions(mapped,word,crate::katsmithy::input_wrapped())}else{mapped};
 let shoulder_word=mapped;
 let shoulder_pressed=shoulder_word&!SHOULDER_HELD.swap(shoulder_word,Ordering::AcqRel);
 STATE.with(|s|{
  let modal=s.mode!=0||capture||s.events&(PAD_REGISTER|PAD_LIST)!=0;
  let previous=INTERNAL_HELD.swap(mapped,Ordering::AcqRel);let pressed=mapped&!previous;
  let release=RELEASE.load(Ordering::Relaxed)&mapped;
  if overlay{RELEASE.store(0,Ordering::Relaxed);return mapped}
  if recomp&&s.mode!=0&&!capture{for(mask,event)in [(0x2000,ACCEPT),(0x4000,BACK),(0x1000,DELETE),(0x8000,SORT),(0x80,PREV),(0x10,UP),(0x20,NEXT),(0x40,DOWN)]{if pressed&mask!=0{s.events|=event}}}
  if recomp&&s.mode!=0&&!capture {
   let (left,right,detail)=crate::menu_buttons::shoulders(shoulder_pressed,recomp);
   if left{s.events|=TAB_PREV;}if right{s.events|=TAB_NEXT;}if detail{s.events|=DETAIL;}
  }
  RELEASE.store(if modal{mapped}else{release},Ordering::Release);
  if modal{0}else{mapped&!release}
 })
}
unsafe extern "C" fn full(s:u32,id:u32){unsafe{filter(0,s,id)}}
unsafe extern "C" fn dual(s:u32,id:u32){unsafe{filter(1,s,id)}}
unsafe extern "C" fn left(s:u32,id:u32){unsafe{filter(2,s,id)}}
unsafe extern "C" fn right(s:u32,id:u32){unsafe{filter(3,s,id)}}
unsafe extern "C" fn handheld(s:u32,id:u32){unsafe{filter(4,s,id)}}
// Keep the input hook and its imports independent: a controller style may
// not have been resolved yet at NRO startup. Never invoke PLT functions early.
static MENU_INSTALLED:AtomicBool=AtomicBool::new(false);
static REPORTED_MISMATCH:AtomicBool=AtomicBool::new(false);
const MENU_OFFSET:u32=0x3de4f8;
const MENU_EXPECTED:[u32;4]=[0xe3a02010,0xe3a00040,0xe0022821,0xe0000821];

fn install_menu_if_safe(base:u32){
 if MENU_INSTALLED.load(Ordering::Acquire){return}
 let site=base+MENU_OFFSET;
 let words=core::array::from_fn(|i|unsafe{rd(site+i as u32*4)});
 if !crate::hook_guard::matches(words,MENU_EXPECTED,site){
  if !REPORTED_MISMATCH.swap(true,Ordering::AcqRel){
   let got:[u32;5]=core::array::from_fn(|i|unsafe{rd(base+GOT[i])});
   forge::log::prelude::error!(
    "[wishlist-input] menu signature mismatch at 0x{:08X}; actual={:08X?}; expected={:08X?}",
    site,words,MENU_EXPECTED
   );
   forge::log::prelude::error!(
    "[wishlist-input] GOT entries at startup={:08X?}; unresolved=0x{:08X}",
    got,base+UNRESOLVED
   );
  }
  return;
 }
 forge::install_hook!(base,menu_translate);
 MENU_INSTALLED.store(true,Ordering::Release);
 forge::log::prelude::info!("[wishlist-input] menu hook installed at 0x{:08X}",site);
}

pub fn refresh_imports(base:u32){
 if !crate::input_chain::menu_ready(crate::button_prompts::is_recomp(),MENU_INSTALLED.load(Ordering::Acquire)){return}
 let detours=[full as *const (),dual as *const (),left as *const (),right as *const (),handheld as *const ()];
 let own=detours.map(|p|p as u32);
 for (i,offset) in GOT.iter().enumerate(){
  let slot=base+offset;
  let address=unsafe{rd(slot)};
  let replacement=detours[i] as u32;
  // A later wrapper can already call us. Never adopt it as our predecessor:
  // doing so would turn that chain into a cycle. Only unresolved styles retry.
  if !crate::input_chain::can_attach(ORIGINAL[i].load(Ordering::Acquire),address,base+UNRESOLVED,&own){
   continue;
  }
  ORIGINAL[i].store(address,Ordering::Release);
  unsafe{ptr::write_volatile(slot as *mut u32,replacement)};
  forge::log::prelude::info!(
   "[wishlist-input] pad import {} chained: original=0x{:08X}",i,address
  );
 }
}

pub fn ready(_base:u32)->bool{
 if !crate::input_chain::menu_ready(crate::button_prompts::is_recomp(),MENU_INSTALLED.load(Ordering::Acquire)){return false}
 // Readiness tracks a successful attachment, not ownership of the GOT head.
 // Another plugin can legitimately sit in front and forward calls to us.
 ORIGINAL.iter().any(|previous|previous.load(Ordering::Acquire)!=0)
}

// Called on startup and on later updates. Resolves only verified hooks and
// already-bound imports; returns false until input is actually available.
pub fn retry(base:u32)->bool{
 // Native input is owned at HID level; installing the engine menu hook is
 // unnecessary and would make native startup depend on an unrelated chain.
 if crate::button_prompts::is_recomp(){install_menu_if_safe(base);}
 refresh_imports(base);
 ready(base)
}
pub fn install(base:u32)->bool{retry(base)}
