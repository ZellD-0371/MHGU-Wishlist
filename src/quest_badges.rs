//! Read-only quest counter adapter for MHGU global 1.4.0. No input/UI hooks.
use crate::native::{rd,valid,visible};
#[derive(Clone,Copy,Default)]pub struct Row{pub id:u32,pub x:f32,pub y:f32}
fn status(stage:u32,a:u32,b:u32,c:u32){crate::diagnostics::event(17,"quest counter stage (1 signature, 2 absent, 3 class, 4 phase, 5 children, 6 panel, 7 ready, 8 counter substate)",[stage,a,b,c]);}
pub fn rows(base:u32)->Option<[Row;8]>{
 // Quest counters occupy the manager's quest slot (+164), not the smithy's +15C.
 // Validate the unmodified updater tail and vtable before touching any children.
 if unsafe{rd(base+0x177e2d4+0x24)}!=base+0x781e34||unsafe{rd(base+0x781e48)}!=0xe2400001{status(1,0,0,0);return None}
 let manager=unsafe{rd(base+0x1899458)};if !valid(manager){status(2,0,0,0);return None}
 let window=unsafe{rd(manager+0x164)};if !valid(window){status(2,0,0,0);return None}
 let vt=unsafe{rd(window)};if vt!=base+0x177e2d4{status(3,vt.wrapping_sub(base),0,0);return None}
 let phase=unsafe{rd(window+0xc)}&7;if phase!=2{status(4,phase,0,0);return None}
 // Substate 2 is the difficulty list. It retains the previous IDs and matrices.
 let substate=unsafe{core::ptr::read((window+0x3c) as *const u8)};
 if !crate::quest_badge_model::quest_list(substate){status(8,substate as u32,0,0);return None}
 let children=unsafe{rd(window+0x34)};if !valid(children){status(5,0,0,0);return None}
 let panel=unsafe{rd(children+2*4)};if !visible(panel,base)||unsafe{rd(panel)}!=base+0x1774b98{status(6,0,0,0);return None}
 let layout=unsafe{rd(panel+0x24)};
 let mut rows=[Row::default();8];
 for (i,row) in rows.iter_mut().enumerate(){
  let id=unsafe{rd(window+0x48+i as u32*4)};
  if id==0||id>1_000_000{continue}
  if let Some(anchor)=crate::native_badges::quest_anchor(layout,16+i as u32){*row=Row{id,x:anchor.x+12.,y:anchor.y+8.5};}
 }
 let count=rows.iter().filter(|r|r.id!=0).count() as u32;
 status(7,count,0,0);
 crate::diagnostics::event(18,"quest badge v2 anchor: first quest / x bits / y bits / valid rows (panel-left, scale 165%)",[rows[0].id,rows[0].x.to_bits(),rows[0].y.to_bits(),count]);
 Some(rows)
}

#[derive(Clone,Copy)]pub struct Detail{pub quest:u32,pub x:f32,pub y:f32}
pub fn detail(base:u32,rows:[Row;8])->Option<Detail>{
 let manager=unsafe{rd(base+0x1899458)};if !valid(manager){return None}
 let window=unsafe{rd(manager+0x164)};if !valid(window)||unsafe{rd(window)}!=base+0x177e2d4{return None}
 // The eight-row quest selector at +13DC stores its cursor at +11.
 let cursor=unsafe{core::ptr::read((window+0x13ed) as *const u8)} as usize;
 let quest=rows.get(cursor)?.id;if quest==0{return None}
 let children=unsafe{rd(window+0x34)};if !valid(children){return None}
 let panel=unsafe{rd(children)};if !visible(panel,base)||unsafe{rd(panel)}!=base+0x1774a24{return None}
 // Match the loaded detail's ID too: while it changes, no preceding quest's gear.
 let data=unsafe{rd(panel+0x50)};if !valid(data)||unsafe{rd(data)}!=base+0x1753068||unsafe{rd(data+8)}!=quest{return None}
 let anchor=crate::native_badges::quest_detail_anchor(unsafe{rd(panel+0x24)})?;
 Some(Detail{quest,x:anchor.x,y:anchor.y})
}

// The detail's page selector at window +0x1400 keeps the page at +0x11, the page
// count shown in "n / m" at +0x12 and the count paging wraps at at +0x18 (3 for every
// quest seen). The L/R handler (+0x780668) handles page slots 0..4, so up to two
// pages are added for the wishlist; the game leaves whatever it drew last on those,
// and the wishlist draws over the lower half. The quest is read from the detail's
// data (+0x50, ID at +8), which stays valid on every page.
const PAGE:u32=0x1411;const SHOWN:u32=0x1412;const WRAP:u32=0x1418;
const DETAIL_PANEL:u32=0x1774a24;const DETAIL_DATA:u32=0x1753068;
static NATIVE:AtomicU32=AtomicU32::new(0);
static WRITTEN:AtomicU32=AtomicU32::new(0);
// Page + 1 to return to after the game loaded the next quest (0: none).
static PENDING:AtomicU32=AtomicU32::new(0);
use core::sync::atomic::{AtomicU32,Ordering::Relaxed};
/// `loading`: the list moved while on a wishlist page; the game is loading the next
/// quest on its own last page and the wishlist page comes back once it is in.
#[derive(Clone,Copy,Debug,PartialEq)]pub struct Pages{pub quest:u32,pub page:u32,pub native:u32,pub loading:bool}
fn detail_window(base:u32)->Option<(u32,u32)>{
 let manager=unsafe{rd(base+0x1899458)};if !valid(manager){return None}
 let window=unsafe{rd(manager+0x164)};if !valid(window)||unsafe{rd(window)}!=base+0x177e2d4||unsafe{rd(window+0xc)}&7!=2{return None}
 let children=unsafe{rd(window+0x34)};if !valid(children){return None}
 // The detail panel hides itself on pages 2 and 3 (its later pages are other
 // panes), so only its class is checked here, not its visibility.
 let panel=unsafe{rd(children)};if !valid(panel)||unsafe{rd(panel)}!=base+DETAIL_PANEL{return None}
 let data=unsafe{rd(panel+0x50)};if !valid(data)||unsafe{rd(data)}!=base+DETAIL_DATA{return None}
 let quest=unsafe{rd(data+8)};(quest!=0&&quest<=1_000_000).then_some((window,quest))
}
/// The quest under the list cursor (the eight-row selector at +13DC keeps its cursor
/// at +11; row IDs at +48).
fn selected(window:u32)->u32{let cursor=rb(window,0x13ed);if cursor<8{unsafe{rd(window+0x48+cursor*4)}}else{0}}
fn rb(window:u32,o:u32)->u32{u32::from(unsafe{core::ptr::read((window+o) as *const u8)})}
fn wb(window:u32,o:u32,v:u32){unsafe{core::ptr::write((window+o) as *mut u8,v as u8)}}
/// Game thread, once a frame: keep `extra(quest)` (0..=2) wishlist pages after the
/// game's own. A count we did not write is the game's; ours is put back to it when
/// no page is wanted any more.
pub fn update_pages(base:u32,extra:impl FnOnce(u32)->u32){
 let Some((window,quest))=detail_window(base) else{WRITTEN.store(0,Relaxed);PENDING.store(0,Relaxed);return};
 // On the added pages the game does not load the next quest when the list cursor
 // moves. Send it to its own last page, which it loads as usual, and once the next
 // quest is in, return to the same page or the last one that quest has.
 let native_now=NATIVE.load(Relaxed);
 if native_now!=0&&rb(window,PAGE)>=native_now{let next=selected(window);if next!=0&&next!=quest{PENDING.store(rb(window,PAGE)+1,Relaxed);wb(window,PAGE,native_now-1);return}}
 let shown=rb(window,SHOWN);let written=WRITTEN.load(Relaxed);
 if shown!=written||written==0{if !(1..=5).contains(&shown){return}NATIVE.store(shown,Relaxed);WRITTEN.store(0,Relaxed);}
 let native=NATIVE.load(Relaxed);let want=native+extra(quest).min(5u32.saturating_sub(native));
 if want!=shown{
  wb(window,SHOWN,want);wb(window,WRAP,want);WRITTEN.store(if want==native{0}else{want},Relaxed);
  if rb(window,PAGE)>=want{wb(window,PAGE,want-1);}
 }
 let pending=PENDING.swap(0,Relaxed);
 if pending!=0&&selected(window)==quest{let target=(pending-1).min(want-1);if target>=native{wb(window,PAGE,target);}}
 else if pending!=0{PENDING.store(pending,Relaxed);}
}
/// The page in view (0-based) and the game's own page count, while a detail is open.
pub fn pages(base:u32)->Option<Pages>{
 let (window,quest)=detail_window(base)?;let native=NATIVE.load(Relaxed);
 if native==0{return None}
 // Between quests the wishlist page is still the one in view: keep its place, so the
 // game's own last page does not flash through.
 let pending=PENDING.load(Relaxed);
 if pending!=0{return Some(Pages{quest,page:(pending-1).max(native),native,loading:true})}
 let page=rb(window,PAGE);
 // The list just moved and the game thread has not seen it yet: the same.
 if selected(window)!=quest{return (page>=native).then_some(Pages{quest,page,native,loading:true})}
 Some(Pages{quest,page,native,loading:false})
}
