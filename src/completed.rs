//! Observe only completed native production transactions; persist on the update
//! callback, outside the game's equipment/material mutation stack.
use core::sync::atomic::{AtomicU32,Ordering};
use crate::{STATE,native::{rd,valid},storage};
static BASE:AtomicU32=AtomicU32::new(0);
static PENDING:[AtomicU32;48]=[const{AtomicU32::new(0)};48];
unsafe fn half(p:u32)->u16{unsafe{core::ptr::read_unaligned(p as *const u16)}}
unsafe fn byte(p:u32)->u8{unsafe{core::ptr::read(p as *const u8)}}
fn key(kind:u8,id:u16,level:u8)->u32{((kind as u32)<<24)|((level as u32)<<16)|id as u32}
fn enqueue(value:u32){if value==0{return}for p in &PENDING{let old=p.load(Ordering::Relaxed);if old==value{return}if old==0&&p.compare_exchange(0,value,Ordering::Release,Ordering::Relaxed).is_ok(){return}}}
pub fn clear(){for p in &PENDING{p.store(0,Ordering::Release)}}
#[forge::hook(offset=0x6fb224)]
fn hunter_produce(owner:u32)->u32{
 let candidate=if valid(owner){let kind=unsafe{half(owner+0x5c)} as u8;let row=unsafe{byte(owner+0x44)} as u32;if matches!(kind,1..=5|7..=21)&&row<8{Some((kind,unsafe{half(owner+0x28d6+row*48)}))}else{None}}else{None};
 let result=original!(owner);
 if let Some((kind,id))=candidate{if id>0&&unsafe{half(owner+0x66)}!=u16::MAX{enqueue(key(kind,id,0));}}
 result
}
#[forge::hook(offset=0x740144)]
fn palico_produce(owner:u32)->u32{
 let candidate=if valid(owner){let row=unsafe{byte(owner+0x751)} as u32;if row<8{let p=owner+row*24;Some((unsafe{half(p+0x10d2)} as u8,unsafe{half(p+0x10d4)}))}else{None}}else{None};
 let result=original!(owner);
 if let Some((kind,id))=candidate{if (22..=24).contains(&kind)&&id>0&&unsafe{half(owner+0x46)}!=u16::MAX{enqueue(key(kind,id,0));}}
 result
}
fn count_item(base:u32,id:u16)->Option<u32>{let mut total=0;for (global,count) in [(0x1858438,2300),(0x18584b8,50)]{let p=unsafe{rd(base+global)};if !valid(p){return None}for i in 0..count{let item=unsafe{half(p+0x1a+i*4)};let count=unsafe{half(p+0x1c+i*4)};if item>=2992||count>99{return None}if item==id{total+=count as u32}}}Some(total)}
#[forge::hook(offset=0x6fdf14)]
fn decoration_produce(owner:u32)->u32{
 let base=BASE.load(Ordering::Relaxed);let id=if valid(owner)&&unsafe{half(owner+0x5c)}==6{let row=unsafe{byte(owner+0x44)} as u32;if row<8{unsafe{half(owner+0x28d6+row*48)}}else{0}}else{0};
 let before=if id>0{count_item(base,id)}else{None};let result=original!(owner);
 if before.and_then(|before|count_item(base,id).map(|after|(before,after))).is_some_and(|(before,after)|after>before){enqueue(key(25,id,0));}result
}
// This routine is reached after the native upgrade has committed its equipment
// replacement. Capture the target row before the game refreshes it.
#[forge::hook(offset=0x6fbcfc)]
fn upgrade_completed(owner:u32)->u32{
 let candidate=if valid(owner){let index=unsafe{byte(owner+0x44)} as u32;if index<8{let p=owner+index*48;
 crate::smithy::SelectedRow{list_kind:unsafe{half(owner+0x5c)},id:unsafe{half(p+0x28d6)},level_index:unsafe{byte(p+0x28dd)},group:unsafe{half(p+0x28da)},points:unsafe{byte(p+0x28de)},materials:core::array::from_fn(|i|crate::model::Material{id:unsafe{half(p+0x28e8+i as u32*4)},count:unsafe{byte(p+0x28ea+i as u32*4)} as u32})}.resolve(crate::generated::RECIPES,crate::generated::ARMOR_UPGRADES).filter(|r|r.level>0)
 }else{None}}else{None};let result=original!(owner);if let Some(r)=candidate{enqueue(key(r.kind,r.id,r.level));}result
}
pub fn install(base:u32)->bool{
 if [(0x6fbcfc,0xe92d48f0),(0x6fb224,0xe92d4070),(0x740144,0xe92d4070),(0x6fdf14,0xe92d4830)].iter().any(|&(at,signature)|unsafe{rd(base+at)}!=signature){return false}
 BASE.store(base,Ordering::Release);forge::install_hook!(base,upgrade_completed);forge::install_hook!(base,hunter_produce);forge::install_hook!(base,palico_produce);forge::install_hook!(base,decoration_produce);true
}
pub fn update(profile:usize){
 let values=core::array::from_fn::<_,48,_>(|i|PENDING[i].swap(0,Ordering::AcqRel));if values.iter().all(|v|*v==0){return}
 let (mut next,expected)=STATE.with(|s|(s.list.clone(),s.profile));if expected!=Some(profile){return}
 let mut changed=false;for value in values{if value>0{changed|=next.remove_completed((value>>24) as u8,value as u16,(value>>16) as u8);}}
 if !changed{return}
 match storage::character(profile).save(&next.encode()){
  Ok(())=>STATE.with(|s|{s.list=next;s.notices.retain_registered(&s.list);s.status=0;}),
  Err(_)=>{for value in values{enqueue(value)}STATE.with(|s|s.status=3);}
 }
}
