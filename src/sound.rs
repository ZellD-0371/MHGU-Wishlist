//! Native UI effects; requests are played outside the shared state lock.
use core::{ptr,sync::atomic::{AtomicBool,AtomicU32,Ordering}};
use alloc::vec::Vec;
static CAPTURE:AtomicBool=AtomicBool::new(false);
static CAPTURED:[AtomicU32;32]=[const{AtomicU32::new(u32::MAX)};32];
static PENDING:AtomicU32=AtomicU32::new(u32::MAX);
pub use crate::sound_cues::Cue;
pub fn cue(cue:Cue){PENDING.store(cue.id(),Ordering::Release);}
pub fn capturing()->bool{CAPTURE.load(Ordering::Relaxed)}
pub fn capture(on:bool){if on{for c in &CAPTURED{c.store(u32::MAX,Ordering::Relaxed)}}CAPTURE.store(on,Ordering::Release);}
pub fn captured()->Vec<u32>{CAPTURED.iter().map(|c|c.load(Ordering::Relaxed)).filter(|c|*c!=u32::MAX).collect()}
pub fn request_open(_id:u32){cue(Cue::Open);}
#[cfg(feature="debug-ui")]
#[forge::hook(offset=0x4eeb78)]
fn play_ui(owner:u32,id:u32)->u32{
 if capturing(){for c in &CAPTURED{let current=c.load(Ordering::Relaxed);if current==id{break}if current==u32::MAX{c.store(id,Ordering::Relaxed);forge::log::prelude::info!("[wishlist-sound] captured SE {}",id);break}}}
 original!(owner,id)
}
#[cfg(feature="debug-ui")]
pub fn install(base:u32){if unsafe{ptr::read_unaligned((base+0x4eeb78) as *const u32)}==0xe1a02000{forge::install_hook!(base,play_ui);}}
pub fn update(base:u32){
 let id=PENDING.swap(u32::MAX,Ordering::AcqRel);if capturing()||id==u32::MAX{return}
 unsafe{
  let manager=ptr::read_unaligned((base+0x1896f5c) as *const u32);if manager<0x10000{return}
  let play:unsafe extern "C" fn(u32,u32)=core::mem::transmute(base+0x4eeb78);play(manager,id);
 }
}
