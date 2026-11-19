//! Keyboard and mouse on MHGU Recomp only. Recomp's own Forge extensions give a
//! free pointer, its position and buttons, and the key pressed this frame. On
//! hardware and emulators these imports are stubs, so they are never called
//! there (button_prompts::is_recomp guards every call).
#[derive(Clone,Copy,Debug,Default)]
pub struct Mouse{pub x:f32,pub y:f32,pub left:bool,pub right:bool,pub wheel:i8,pub moved:bool}
/// Frames after a key press during which a pad press still counts as the
/// keyboard: Recomp also turns keys into pad buttons.
const KEY_GRACE:u32=15;

#[cfg(target_arch="arm")]
mod runtime{
 use core::sync::atomic::{AtomicBool,AtomicU32,Ordering};
 use super::{Mouse,KEY_GRACE};
 unsafe extern "C"{
  fn forge_input_getMouse(x:*mut f32,y:*mut f32,buttons:*mut u32)->bool;
  fn forge_input_setMouseFree(free:bool);
  fn forge_input_getKeyPressed(out:*mut u8,size:u32)->bool;
 }
 static FREE:AtomicBool=AtomicBool::new(false);
 static PREV:AtomicU32=AtomicU32::new(u32::MAX);
 static LAST:[AtomicU32;2]=[AtomicU32::new(0),AtomicU32::new(0)];
 static ACTIVE:AtomicBool=AtomicBool::new(false);
 static AGE:AtomicU32=AtomicU32::new(u32::MAX);
 static mut FRAME:Option<Mouse>=None;
 fn recomp()->bool{crate::button_prompts::is_recomp()}
 /// Free pointer while our own window is open; Recomp then shows it and its
 /// clicks reach only us. Never used for the dialog over Skill-Simulator, which
 /// already freed the pointer and would lose it if we released it.
 pub fn set_free(free:bool){
  if !recomp()||FREE.swap(free,Ordering::Relaxed)==free{return}
  unsafe{forge_input_setMouseFree(free)};
  PREV.store(u32::MAX,Ordering::Relaxed);
 }
 /// Once per rendered frame: read the key and the pointer, and track which
 /// device was used last (`pad_pressed`: a pad button was newly pressed).
 pub fn poll(pad_pressed:bool){
  unsafe{*core::ptr::addr_of_mut!(FRAME)=None;}
  if !recomp(){ACTIVE.store(false,Ordering::Relaxed);return}
  let mut name=[0u8;32];
  let pressed=unsafe{forge_input_getKeyPressed(name.as_mut_ptr(),name.len() as u32)};
  let n=name.iter().position(|&c|c==0).unwrap_or(name.len());
  let wheel:i8=if !pressed{0}else{match &name[..n]{b"Wheel Up"=>1,b"Wheel Down"=>-1,_=>0}};
  let (mut x,mut y,mut buttons)=(0f32,0f32,0u32);
  let mouse=if unsafe{forge_input_getMouse(&mut x,&mut y,&mut buttons)}{
   // Buttons held when the pointer became ours never count as a click.
   let new=buttons&!PREV.swap(buttons,Ordering::Relaxed);
   let (px,py)=(f32::from_bits(LAST[0].swap(x.to_bits(),Ordering::Relaxed)),f32::from_bits(LAST[1].swap(y.to_bits(),Ordering::Relaxed)));
   Some(Mouse{x,y,left:new&1!=0,right:new&4!=0,wheel,moved:(px-x).abs()+(py-y).abs()>3.})
  }else{PREV.store(u32::MAX,Ordering::Relaxed);None};
  let used=pressed||mouse.is_some_and(|m|m.left||m.right||m.moved);
  if used{AGE.store(0,Ordering::Relaxed);ACTIVE.store(true,Ordering::Relaxed);}
  else{
   let age=AGE.load(Ordering::Relaxed).saturating_add(1);AGE.store(age,Ordering::Relaxed);
   if pad_pressed&&age>KEY_GRACE{ACTIVE.store(false,Ordering::Relaxed);}
  }
  unsafe{*core::ptr::addr_of_mut!(FRAME)=mouse;}
 }
 /// The pointer this frame in console pixels (1280x720), when it is free.
 pub fn mouse()->Option<Mouse>{unsafe{*core::ptr::addr_of!(FRAME)}}
 /// The keyboard or mouse was used last.
 pub fn active()->bool{ACTIVE.load(Ordering::Relaxed)}
}
#[cfg(target_arch="arm")]
pub use runtime::{set_free,poll,mouse,active};
#[cfg(not(target_arch="arm"))]
pub fn set_free(_:bool){}
#[cfg(not(target_arch="arm"))]
pub fn poll(_:bool){}
#[cfg(not(target_arch="arm"))]
pub fn mouse()->Option<Mouse>{None}
#[cfg(not(target_arch="arm"))]
pub fn active()->bool{false}
