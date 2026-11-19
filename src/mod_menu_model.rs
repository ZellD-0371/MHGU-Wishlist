//! Shared "Mods" entry of the game's Hunter's Notes menu ("MHGU-MODMENU-V01"
//! version 1). A submenu shows at most seven rows, so mods share the seventh:
//! every participating mod publishes one block in its data segment, the host
//! (lowest `order` among host-capable blocks) adds the entry and its list, and
//! asks a participant to open by bumping that block's `request` counter.
//! Spec: docs/mod-menu-protocol.md. No platform calls here.
//!
//! Layout (repr(C), little endian, 128 bytes, 4-aligned, in .data):
//!   +0   [u8;16] magic "MHGU-MODMENU-V01"
//!   +16  u32 version 1
//!   +20  u32 order      list position and host priority (lower first)
//!   +24  u32 flags      bit0 host-capable (can draw the list)
//!   +28  u32 ready      1 = the mod can open its window now (else dimmed)
//!   +32  u32 request    bumped by the host: open your window
//!   +36  [u32;9] names  UTF-8 NUL-terminated list labels (en, zh-Hans, zh-Hant, ja, ko, de, fr, it, es)
//!   +72  [u32;9] descs  UTF-8 one-line descriptions, 0 = none
//!   +108 [u32;5] reserved 0
extern crate alloc;
use alloc::vec::Vec;

pub const MAGIC:&[u8;16]=b"MHGU-MODMENU-V01";
pub const VERSION:u32=1;
pub const SIZE:usize=128;
pub const ORDER:usize=20;
pub const FLAGS:usize=24;
pub const READY:usize=28;
pub const REQUEST:usize=32;
pub const NAMES:usize=36;
pub const DESCS:usize=72;
pub const HOST:u32=1;
/// Wishlist's order: it hosts the list whenever it is loaded.
pub const WISHLIST_ORDER:u32=10;

#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct Entry{pub address:u32,pub order:u32,pub flags:u32,pub ready:bool,pub request:u32,pub names:[u32;9],pub descs:[u32;9]}
fn word(b:&[u8],at:usize)->u32{u32::from_le_bytes([b[at],b[at+1],b[at+2],b[at+3]])}
pub fn parse(address:u32,b:&[u8;SIZE])->Option<Entry>{
 if &b[..16]!=MAGIC||word(b,16)!=VERSION{return None}
 Some(Entry{address,order:word(b,ORDER),flags:word(b,FLAGS),ready:word(b,READY)==1,request:word(b,REQUEST),
  names:core::array::from_fn(|i|word(b,NAMES+i*4)),descs:core::array::from_fn(|i|word(b,DESCS+i*4))})
}
/// List order: by `order`, then by address so every mod agrees.
pub fn sorted(mut entries:Vec<Entry>)->Vec<Entry>{entries.sort_by_key(|e|(e.order,e.address));entries}
/// The host: the first host-capable entry in list order.
pub fn host(entries:&[Entry])->Option<u32>{entries.iter().filter(|e|e.flags&HOST!=0).min_by_key(|e|(e.order,e.address)).map(|e|e.address)}
/// What the Hunter's Notes entry does when chosen.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Pick{Open(u32),List}
/// One participant opens directly (no extra press); two or more get the list.
pub fn pick(entries:&[Entry])->Option<Pick>{
 match entries.len(){0=>None,1=>Some(Pick::Open(entries[0].address)),_=>Some(Pick::List)}
}

#[cfg(test)]
mod tests{
 use super::*;
 fn block(order:u32,flags:u32,ready:u32,request:u32)->[u8;SIZE]{
  let mut b=[0u8;SIZE];b[..16].copy_from_slice(MAGIC);b[16]=1;
  b[ORDER..ORDER+4].copy_from_slice(&order.to_le_bytes());b[FLAGS..FLAGS+4].copy_from_slice(&flags.to_le_bytes());
  b[READY..READY+4].copy_from_slice(&ready.to_le_bytes());b[REQUEST..REQUEST+4].copy_from_slice(&request.to_le_bytes());
  b[NAMES+16..NAMES+20].copy_from_slice(&0x1234u32.to_le_bytes());b
 }
 #[test]
 fn parses_and_rejects_other_versions(){
  let e=parse(0x100,&block(20,0,1,7)).unwrap();
  assert_eq!((e.address,e.order,e.flags,e.ready,e.request,e.names[4]),(0x100,20,0,true,7,0x1234));
  let mut b=block(20,0,1,7);b[16]=2;assert!(parse(0x100,&b).is_none());
  b=block(20,0,1,7);b[0]=b'X';assert!(parse(0x100,&b).is_none());
 }
 #[test]
 fn host_is_the_first_capable_entry_and_order_is_stable(){
  let wishlist=parse(0x900,&block(WISHLIST_ORDER,HOST,1,0)).unwrap();
  let sim=parse(0x500,&block(20,0,1,0)).unwrap();
  let other=parse(0x400,&block(20,HOST,1,0)).unwrap();
  let list=sorted(alloc::vec![sim,other,wishlist]);
  assert_eq!(list.iter().map(|e|e.address).collect::<Vec<_>>(),[0x900,0x400,0x500]);
  assert_eq!(host(&list),Some(0x900));
  assert_eq!(host(&[sim,other]),Some(0x400));
  assert_eq!(host(&[sim]),None);
 }
 #[test]
 fn a_single_mod_opens_directly(){
  let sim=parse(0x500,&block(20,0,1,0)).unwrap();let w=parse(0x900,&block(10,HOST,1,0)).unwrap();
  assert_eq!(pick(&[w]),Some(Pick::Open(0x900)));assert_eq!(pick(&[w,sim]),Some(Pick::List));assert_eq!(pick(&[]),None);
 }
}
