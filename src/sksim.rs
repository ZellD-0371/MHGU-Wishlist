//! Runtime side of the Skill-Simulator guest link (see sksim_model): finds the
//! shared block in Skill-Simulator's data segment once and reads/writes it.
use core::sync::atomic::{AtomicBool,AtomicU32,Ordering};
use crate::katsmithy::{Map,query,readable};
use crate::sksim_model::{self as model,View,SIZE};
use forge::log::prelude::info;
static SCANNED:AtomicBool=AtomicBool::new(false);
static BLOCK:AtomicU32=AtomicU32::new(0);
static GUEST_FRAME:AtomicU32=AtomicU32::new(0);
static mut ALIVE:model::Alive=model::Alive::new();
const MAX_DATA:u32=16*1024*1024;

unsafe fn word(address:u32)->u32{unsafe{core::ptr::read_unaligned(address as *const u32)}}
fn writable(address:u32)->bool{let mut m=Map::default();let r=unsafe{query(&mut m,address)};r==0&&m.perm&3==3}
/// The block inside another NRO's data segment (never our own module).
fn find(own:u32)->Option<u32>{find_block(own,model::MAGIC,|v|v==model::VERSION,model::GUEST_FRAME as u32)}
/// Skill-Simulator's window block ("MHGU-SKSIM-WIN01", any version: the
/// first 44 bytes never change): open flag +20, frame rect +24..+36.
static WINDOW:AtomicU32=AtomicU32::new(0);
const WINDOW_MAGIC:&[u8;16]=b"MHGU-SKSIM-WIN01";
fn find_block(own:u32,magic:&[u8;16],version:impl Fn(u32)->bool,writable_at:u32)->Option<u32>{
 let mut at=0u32;
 for _ in 0..16384{
  let mut m=Map::default();if unsafe{query(&mut m,at)}!=0{break}
  let next=m.base.saturating_add(m.size);if next<=at as u64{break}
  if m.perm&5==5&&m.base>=0x10000&&m.size>=0x80&&m.base<=u32::MAX as u64{
   let base=m.base as u32;
   if readable(base,0x80)&&unsafe{word(base+0x10)}==0x304f524e{
    let (image,data,len)=unsafe{(word(base+0x18),word(base+0x30),word(base+0x34))};
    let fits=data.checked_add(len).is_some_and(|end|end<=image)&&(SIZE as u32..=MAX_DATA).contains(&len);
    if fits&&!(base..base.saturating_add(image)).contains(&own)&&readable(base+data,len){
     let bytes=unsafe{core::slice::from_raw_parts((base+data) as *const u8,len as usize)};
     let mut i=0;
     while i+SIZE<=bytes.len(){
      if &bytes[i..i+16]==magic&&version(u32::from_le_bytes([bytes[i+16],bytes[i+17],bytes[i+18],bytes[i+19]])){
       let block=base+data+i as u32;
       if writable(block+writable_at){return Some(block)}
      }
      i+=4;
     }
    }
   }
  }
  if next>=0x1_0000_0000{break}at=next as u32;
 }
 None
}
/// Once, after every plugin was loaded.
pub fn discover_once(){
 if SCANNED.swap(true,Ordering::AcqRel){return}
 match find(discover_once as *const () as u32){
  Some(block)=>{BLOCK.store(block,Ordering::Release);info!("[wishlist] Skill-Simulator link found at {:08X}",block);}
  None=>info!("[wishlist] Skill-Simulator link not found"),
 }
 if let Some(block)=find_block(discover_once as *const () as u32,WINDOW_MAGIC,|v|v>=1,0){WINDOW.store(block,Ordering::Release);}
}
/// Skill-Simulator's window frame (left, top, right, bottom) in screen pixels.
pub fn window_rect()->Option<[f32;4]>{
 let block=WINDOW.load(Ordering::Acquire);if block==0{return None}
 let r:[f32;4]=core::array::from_fn(|i|f32::from_bits(unsafe{core::ptr::read_volatile((block+24+i as u32*4) as *const u32)}));
 (r.iter().all(|v|v.is_finite())&&r[2]>r[0]&&r[3]>r[1]).then_some(r)
}
/// Skill-Simulator is loaded (its block was found).
pub fn present()->bool{BLOCK.load(Ordering::Acquire)!=0}
fn read()->Option<View>{
 let block=BLOCK.load(Ordering::Acquire);if block==0{return None}
 let mut bytes=[0u8;SIZE];
 for (i,b) in bytes.iter_mut().enumerate(){*b=unsafe{core::ptr::read_volatile((block+i as u32) as *const u8)};}
 model::parse(&bytes)
}
/// Once per rendered frame: the view while Skill-Simulator's window is open.
pub fn frame()->Option<View>{
 let v=read()?;
 let alive=unsafe{(*core::ptr::addr_of_mut!(ALIVE)).step(&v)};
 alive.then_some(v)
}
/// Our requests for this frame; the heartbeat keeps them honoured.
pub fn request(hint_width:f32,modal:bool){
 let block=BLOCK.load(Ordering::Acquire);if block==0{return}
 let frame=GUEST_FRAME.fetch_add(1,Ordering::Relaxed).wrapping_add(1);
 unsafe{
  core::ptr::write_volatile((block+model::HINT_WIDTH as u32) as *mut u32,hint_width.max(0.).to_bits());
  core::ptr::write_volatile((block+model::GUEST_MODAL as u32) as *mut u32,u32::from(modal));
  core::ptr::write_volatile((block+model::GUEST_FRAME as u32) as *mut u32,frame);
 }
}
