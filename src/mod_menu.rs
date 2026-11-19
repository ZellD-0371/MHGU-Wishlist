//! Runtime side of the shared "Mods" entry (see mod_menu_model): publishes our
//! block, finds every participating mod's block once, and asks one to open.
use core::sync::atomic::{AtomicBool,AtomicU32,Ordering};
use alloc::vec::Vec;
use crate::katsmithy::{Map,query,readable};
use crate::mod_menu_model::{self as model,Entry,SIZE};
use forge::log::prelude::info;

#[repr(C,align(4))]
struct Block{magic:[u8;16],version:u32,order:u32,flags:u32,ready:u32,request:u32,names:[u32;9],descs:[u32;9],reserved:[u32;5]}
const _:()=assert!(core::mem::size_of::<Block>()==SIZE);
// Non-zero contents keep it in .data, where other mods look for it.
#[used]
static mut BLOCK:Block=Block{magic:*model::MAGIC,version:model::VERSION,order:model::WISHLIST_ORDER,flags:model::HOST,ready:0,request:0,names:[0;9],descs:[0;9],reserved:[0;5]};
const NAMES:[&core::ffi::CStr;9]=[c"Wishlist",c"愿望清单",c"願望清單",c"ウィッシュリスト",c"찜 목록",c"Wunschliste",c"Liste de souhaits",c"Lista desideri",c"Lista de deseos"];
const DESCS:[&core::ffi::CStr;9]=[
 c"Check registered equipment and the materials it needs.",
 c"查看已登记的装备及其所需素材。",
 c"查看已登錄的裝備及其所需素材。",
 c"登録した装備と必要素材を確認します。",
 c"등록한 장비와 필요 소재를 확인합니다.",
 c"Eingetragene Ausrüstung und benötigte Materialien ansehen.",
 c"Voir l'équipement enregistré et les matériaux requis.",
 c"Visualizza l'equipaggiamento registrato e i materiali.",
 c"Consulta el equipo registrado y los materiales necesarios.",
];
static SCANNED:AtomicBool=AtomicBool::new(false);
static mut FOUND:Vec<u32>=Vec::new();
static SEEN_REQUEST:AtomicU32=AtomicU32::new(0);
const MAX_DATA:u32=16*1024*1024;

fn own()->u32{core::ptr::addr_of!(BLOCK) as u32}
unsafe fn word(address:u32)->u32{unsafe{core::ptr::read_volatile(address as *const u32)}}
/// Every block in every loaded NRO's data segment, ours included.
fn scan()->Vec<u32>{
 let mut out=Vec::new();let mut at=0u32;
 for _ in 0..16384{
  let mut m=Map::default();if unsafe{query(&mut m,at)}!=0{break}
  let next=m.base.saturating_add(m.size);if next<=at as u64{break}
  if m.perm&5==5&&m.base>=0x10000&&m.size>=0x80&&m.base<=u32::MAX as u64{
   let base=m.base as u32;
   if readable(base,0x80)&&unsafe{word(base+0x10)}==0x304f524e{
    let (image,data,len)=unsafe{(word(base+0x18),word(base+0x30),word(base+0x34))};
    if data.checked_add(len).is_some_and(|end|end<=image)&&(SIZE as u32..=MAX_DATA).contains(&len)&&readable(base+data,len){
     let bytes=unsafe{core::slice::from_raw_parts((base+data) as *const u8,len as usize)};
     let mut i=0;
     while i+SIZE<=bytes.len(){
      if &bytes[i..i+16]==model::MAGIC&&u32::from_le_bytes([bytes[i+16],bytes[i+17],bytes[i+18],bytes[i+19]])==model::VERSION{out.push(base+data+i as u32);}
      i+=4;
     }
    }
   }
  }
  if next>=0x1_0000_0000{break}at=next as u32;
 }
 out
}
/// Fill our labels, then find every block once all plugins are loaded.
pub fn discover_once(){
 if SCANNED.swap(true,Ordering::AcqRel){return}
 unsafe{
  let b=&mut *core::ptr::addr_of_mut!(BLOCK);
  for i in 0..9{b.names[i]=NAMES[i].as_ptr() as u32;b.descs[i]=DESCS[i].as_ptr() as u32;}
  let mut found=scan();if !found.contains(&own()){found.push(own());}
  info!("[wishlist] mod menu: {} participating mod(s)",found.len());
  *core::ptr::addr_of_mut!(FOUND)=found;
 }
}
fn read(address:u32)->Option<Entry>{
 let mut bytes=[0u8;SIZE];
 for (i,b) in bytes.iter_mut().enumerate(){*b=unsafe{core::ptr::read_volatile((address+i as u32) as *const u8)};}
 model::parse(address,&bytes)
}
/// Participants in list order.
pub fn entries()->Vec<Entry>{
 if !SCANNED.load(Ordering::Acquire){return Vec::new()}
 model::sorted(unsafe{(*core::ptr::addr_of!(FOUND)).iter().filter_map(|a|read(*a)).collect()})
}
pub fn is_ours(address:u32)->bool{address==own()}
/// We add the Hunter's Notes entry and its list.
pub fn hosting(entries:&[Entry])->bool{model::host(entries)==Some(own())}
pub fn set_ready(ready:bool){unsafe{core::ptr::write_volatile(core::ptr::addr_of_mut!(BLOCK.ready),u32::from(ready));}}
/// Ask a participant to open its window.
pub fn request(address:u32){
 if is_ours(address){return}
 unsafe{let p=(address+model::REQUEST as u32) as *mut u32;core::ptr::write_volatile(p,core::ptr::read_volatile(p).wrapping_add(1));}
}
/// Another host asked us to open (each bump counts once).
pub fn take_request()->bool{
 let now=unsafe{core::ptr::read_volatile(core::ptr::addr_of!(BLOCK.request))};
 SEEN_REQUEST.swap(now,Ordering::AcqRel)!=now
}
/// A participant's description in `lang` (else English), as a text pointer.
pub fn description(address:u32,lang:usize)->Option<u32>{
 let e=read(address)?;let p=if e.descs[lang%9]!=0{e.descs[lang%9]}else{e.descs[0]};(p>=0x10000).then_some(p)
}
