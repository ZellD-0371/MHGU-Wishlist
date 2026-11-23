//! The native +/START menu, not Forge's Insert overlay.
//!
//! 'Wishlist' entry at the end of the village / gathering hall menu's top list
//! (the "Menu 1/2" window): that list has 16 entries on 8-row pages, so ours
//! opens a page 3 (after any entry another mod appended first, e.g.
//! Skill-Simulator). The field (quest) menu is left alone: its last row is
//! 'Abandon Quest', reached by habit, so nothing may follow it. Until
//! 2026-11-21 the entry was the 7th row of Hunter's Notes, which WishFinder
//! (another mod) also uses. Same rules as Skill-Simulator's game_menu.rs:
//! our index = the entry count first seen, arrays grown with the game's own
//! allocator, an old names array freed only when it is the game's (16), the
//! list total only ever grown, only our own slot written.
//! Research (1.4.0, Skill-Simulator 2026-11-21): menu object +0x48 state (-1
//! closed, 0 top list, 1 submenu); top names +0xA8/+0xAC (8 bytes: u8
//! selectable, u32 UTF-8 text); top descriptions +0xC0/+0xC4 (12 bytes);
//! top widget +0xD8 (+0xD cursor, +0xE rows per page, +0x10 page, +0x14
//! total). Town init 0x78F784 (vtable 18) resets the total and cursor; town
//! top fill 0x791408 (vtable 37) writes names and descriptions on each open;
//! town 'selectable' query 0x7918CC (vtable 36); the town action ignores
//! indices above 15. Game allocator: GOT 0x1838B5C -> 0x7AA47C -> vtable
//! +0x1C alloc / +0x34 free. Description resize 0x625138 (default element
//! 0x54F9C0); list total 0x54EE34.
pub const QUEST_VTABLE:u32=0x177aeb8;
pub const LOBBY_VTABLE:u32=0x177e424;
pub fn is_main_menu(vtable:u32,phase:u32,substate:u32,pane_visible:bool)->bool{
 matches!(vtable,QUEST_VTABLE|LOBBY_VTABLE)&&matches!(phase,2|3)&&substate==0&&pane_visible
}
/// Top index of Hunter's Notes: our entry follows its selectable flag, and its
/// text tells which language the game shows.
pub const NOTES:u32=5;
/// Our description message id (Skill-Simulator uses 0x7FF0), message file 9.
pub const DESC_ID:u32=0x7ff1;
pub const MSG_FILE:u32=9;
/// Entries the game's village / hall constructor allocates: only an array of
/// exactly this size is known to be the game's own, and only that is freed.
pub const GAME_TOWN_COUNT:u32=16;
/// Our index: the count first seen, kept while later menus still have it.
pub fn town_index(previous:Option<u32>,count:u32)->u32{
 match previous{Some(i) if count>i=>i,_=>count}
}
/// The selected top index: page * rows + cursor (0x66F8F8).
pub fn selected(page:u16,rows:u8,cursor:u8)->u32{u32::from(page)*u32::from(rows)+u32::from(cursor)}
/// Script of the game's menu text, to label our entry in a font the game has:
/// Korean, Japanese and Chinese by their characters; Latin follows the plugin
/// language when that is a Latin one. Languages: 0 en, 1 zh-Hans, 2 zh-Hant,
/// 3 ja, 4 ko, 5 de, 6 fr, 7 it, 8 es.
pub fn detect(text:&str,plugin:usize)->usize{
 if text.chars().any(|c|('\u{ac00}'..='\u{d7a3}').contains(&c)){return 4}
 if text.chars().any(|c|('\u{3040}'..='\u{30ff}').contains(&c)){return 3}
 if text.chars().any(|c|('\u{4e00}'..='\u{9fff}').contains(&c)){return if plugin==2{2}else{1}}
 if matches!(plugin,0|5|6|7|8){plugin}else{0}
}
#[cfg(not(test))]
mod native{
 use super::*;
 use core::sync::atomic::{AtomicBool,AtomicU32,Ordering};
 use crate::native::{rd,valid};
 use forge::log::prelude::info;
 static INSTALLED:AtomicBool=AtomicBool::new(false);
 static ENTRY:AtomicBool=AtomicBool::new(false);
 static QUEST:AtomicU32=AtomicU32::new(0);
 static LOBBY:AtomicU32=AtomicU32::new(0);
 static HOT:AtomicBool=AtomicBool::new(false);
 static LANG:AtomicU32=AtomicU32::new(0);
 /// Our top index in the village / hall menu (u32::MAX until set up).
 static TOWN_INDEX:AtomicU32=AtomicU32::new(u32::MAX);
 const ALLOC_GOT:u32=0x1838b5c;
 const LABELS:[&core::ffi::CStr;9]=[c"Wishlist",c"愿望清单",c"願望清單",c"ウィッシュリスト",c"찜 목록",c"Wunschliste",c"Liste de souhaits",c"Lista desideri",c"Lista de deseos"];
 const DESCS:[&core::ffi::CStr;9]=[
  c"Check registered equipment\nand the materials it needs.",
  c"查看已登记的装备\n及其所需素材。",
  c"查看已登錄的裝備\n及其所需素材。",
  c"登録した装備と\n必要素材を確認します。",
  c"등록한 장비와\n필요 소재를 확인합니다.",
  c"Eingetragene Ausrüstung und\nbenötigte Materialien ansehen.",
  c"Voir l'équipement enregistré\net les matériaux requis.",
  c"Visualizza l'equipaggiamento\nregistrato e i materiali.",
  c"Consulta el equipo registrado\ny los materiales necesarios.",
 ];
 unsafe fn rb(p:u32)->u8{unsafe{core::ptr::read(p as *const u8)}}
 unsafe fn rh(p:u32)->u16{unsafe{core::ptr::read_unaligned(p as *const u16)}}
 unsafe fn wr(p:u32,v:u32){unsafe{core::ptr::write_unaligned(p as *mut u32,v)}}
 fn is_menu(owner:u32,base:u32)->bool{valid(owner)&&matches!(unsafe{rd(owner)}.wrapping_sub(base),QUEST_VTABLE|LOBBY_VTABLE)}
 fn is_town(owner:u32,base:u32)->bool{valid(owner)&&unsafe{rd(owner)}.wrapping_sub(base)==LOBBY_VTABLE}
 /// Our index in this menu, once a village / hall menu has been set up.
 fn our_index(owner:u32,base:u32)->Option<u32>{
  if !is_town(owner,base){return None}
  let i=TOWN_INDEX.load(Ordering::Relaxed);
  (i!=u32::MAX&&unsafe{rd(owner+0xac)}>i).then_some(i)
 }
 fn text(p:u32)->alloc::string::String{
  if !valid(p){return alloc::string::String::new()}
  let mut v=alloc::vec::Vec::new();for i in 0..64{let b=unsafe{rb(p+i)};if b==0{break}v.push(b);}
  alloc::string::String::from_utf8_lossy(&v).into_owned()
 }
 /// The game allocator the menu arrays come from: (allocator, alloc, free).
 unsafe fn allocator(base:u32)->Option<(u32,extern "C" fn(u32,u32,u32)->u32,extern "C" fn(u32,u32))>{unsafe{
  let descriptor=rd(base+ALLOC_GOT);if !valid(descriptor){return None}
  let get:extern "C" fn(u32)->u32=core::mem::transmute(base+0x7aa47c);
  let a=get(descriptor);if !valid(a)||!valid(rd(a)){return None}
  let vt=rd(a);Some((a,core::mem::transmute(rd(vt+0x1c)),core::mem::transmute(rd(vt+0x34))))
 }}
 /// After the town menu's init: one more top entry (names, descriptions,
 /// total). Init runs again on later openings and resets the total, so the
 /// arrays grow once per menu object and the total is restored every time.
 fn after_init(this:u32,base:u32){
  if !is_town(this,base){return}
  unsafe{
   let (count,names)=(rd(this+0xac),rd(this+0xa8));
   if !valid(names)||count==0||count>64{return}
   let previous=TOWN_INDEX.load(Ordering::Relaxed);
   let index=town_index((previous!=u32::MAX).then_some(previous),count);TOWN_INDEX.store(index,Ordering::Relaxed);
   let n=index+1;
   // Names: a new array from the game allocator, old slots copied, ours
   // pointing at a valid text at once.
   if count<n{
    let Some((a,alloc,free))=allocator(base) else{return};
    let fresh=alloc(a,n*8,0x10);if !valid(fresh){return}
    core::ptr::copy_nonoverlapping(names as *const u8,fresh as *mut u8,(count*8) as usize);
    for i in count..n{core::ptr::write((fresh+i*8) as *mut u8,0);wr(fresh+i*8+4,LABELS[0].as_ptr() as u32);}
    wr(this+0xa8,fresh);wr(this+0xac,n);
    // Free only the game's own array (16); another size was replaced by
    // another mod, possibly from its own heap: leave it (about 130 bytes).
    if count==GAME_TOWN_COUNT{free(a,names);}
    else{info!("[wishlist] start menu: names array already replaced by another mod ({} entries); old one kept",count);}
   }
   // Descriptions: the game's own resize.
   if rd(this+0xc4)<n{
    let mut template=[0u32;3];
    let make:extern "C" fn(u32)=core::mem::transmute(base+0x54f9c0);make(template.as_mut_ptr() as u32);
    let resize:extern "C" fn(u32,u32,u32)=core::mem::transmute(base+0x625138);resize(this+0xc0,n,template.as_ptr() as u32);
   }
   if rd(this+0xc4)<n{return}
   // List total (pages recounted): only ever grown.
   let list=this+0xd8;
   if u32::from(rh(list+0x14))<n{
    let set_total:extern "C" fn(u32,u32,u32)=core::mem::transmute(base+0x54ee34);set_total(list,n,rd(list+0x1c));
   }
   info!("[wishlist] start menu (village/hub): our entry at index {}, {} entries, {} rows per page, {} pages",index,rh(list+0x14),rb(list+0xe),rh(list+0x12));
  }
 }
 /// After the town top list was filled (each open): our name and description.
 fn after_fill(this:u32,base:u32){
  let Some(index)=our_index(this,base) else{return};
  unsafe{
   let (names,descs)=(rd(this+0xa8),rd(this+0xc0));
   if !valid(names)||!valid(descs)||rd(this+0xc4)<=index||rd(this+0xac)<=NOTES{return}
   let plugin=crate::STATE.with(|s|s.list.language);
   let lang=detect(&text(rd(names+NOTES*8+4)),plugin);LANG.store(lang as u32,Ordering::Relaxed);
   let e=names+index*8;
   // Selectable like Hunter's Notes (dimmed where the game dims it).
   core::ptr::write(e as *mut u8,rb(names+NOTES*8));wr(e+4,LABELS[lang].as_ptr() as u32);
   let d=descs+index*12;
   core::ptr::write((d+4) as *mut u8,0);
   core::ptr::write_unaligned((d+6) as *mut u16,MSG_FILE as u16);core::ptr::write_unaligned((d+8) as *mut u16,DESC_ID as u16);core::ptr::write_unaligned((d+10) as *mut u16,0xffff);
  }
 }
 #[forge::hook(offset=0x78f784)]
 fn init_town(this:u32)->u32{let r=original!(this);after_init(this,forge::mem::text_addr());r}
 #[forge::hook(offset=0x791408)]
 fn fill_town(this:u32)->u32{let r=original!(this);after_fill(this,forge::mem::text_addr());r}
 // Backstop: the game is never told our entry is selectable, so it cannot act
 // on it even if our A interception were missed.
 #[forge::hook(offset=0x7918cc)]
 fn pick_town(this:u32,index:u32)->u32{
  if our_index(this,forge::mem::text_addr())==Some(index){return 0}
  original!(this,index)
 }
 // The common help bar at the bottom (0x65E5BC: flags, kind, file, id, and two
 // stack words; it draws into component 4 of the HUD object). While the
 // wishlist is open it shows our message instead, in the game's own panel and
 // font; the game's latest request is kept and restored when we close.
 const HELP_ID:u32=0x7ff8;
 static HELP_OURS:AtomicBool=AtomicBool::new(false);
 static HELP_INSTALLED:AtomicBool=AtomicBool::new(false);
 // The help function is variadic: when its 6th argument is 1 it reads format values
 // from the caller's stack right after it (sp+0x30 inside, the 7th word on) and hands
 // them to +0x6429A4. The Smithy (+0x708204) passes one such value. The hook therefore
 // takes the next stack words too and passes them on in the same places, so the
 // original sees exactly the caller's arguments (otherwise it read our frame and built
 // garbage text every call, which piled up over KatSmithy's Smithy).
 const HELP_WORDS:usize=6+HELP_EXTRA;const HELP_EXTRA:usize=8;
 static HELP_LAST:[AtomicU32;HELP_WORDS]=[const{AtomicU32::new(0)};HELP_WORDS];
 static HELP_SEEN:AtomicBool=AtomicBool::new(false);
 static mut HELP_TEXT:alloc::vec::Vec<u8>=alloc::vec::Vec::new();
 static mut HELP_WANT:Option<alloc::string::String>=None;
 static mut HELP_SHOWN:Option<alloc::string::String>=None;
 #[forge::hook(offset=0x65e5bc)]
 fn help(flags:u32,kind:u32,file:u32,id:u32,a:u32,b:u32,x0:u32,x1:u32,x2:u32,x3:u32,x4:u32,x5:u32,x6:u32,x7:u32)->u32{
  if HELP_CALLING.load(Ordering::Acquire){return original!(flags,kind,file,id,a,b,x0,x1,x2,x3,x4,x5,x6,x7)}
  for (slot,v) in HELP_LAST.iter().zip([flags,kind,file,id,a,b,x0,x1,x2,x3,x4,x5,x6,x7]){slot.store(v,Ordering::Relaxed);}
  HELP_SEEN.store(true,Ordering::Release);
  // While ours is shown, the game's own updates wait for our window to close.
  if HELP_OURS.load(Ordering::Acquire){return 0}
  original!(flags,kind,file,id,a,b,x0,x1,x2,x3,x4,x5,x6,x7)
 }
 static HELP_CALLING:AtomicBool=AtomicBool::new(false);
 fn call_help(a:[u32;HELP_WORDS]){
  type Help=extern "C" fn(u32,u32,u32,u32,u32,u32,u32,u32,u32,u32,u32,u32,u32,u32)->u32;
  let f:Help=unsafe{core::mem::transmute(forge::mem::text_addr()+0x65e5bc)};
  HELP_CALLING.store(true,Ordering::Release);f(a[0],a[1],a[2],a[3],a[4],a[5],a[6],a[7],a[8],a[9],a[10],a[11],a[12],a[13]);HELP_CALLING.store(false,Ordering::Release);
 }
 /// The game's help bar can carry our message.
 pub fn help_available()->bool{HELP_INSTALLED.load(Ordering::Acquire)&&HELP_SEEN.load(Ordering::Acquire)}
 /// Render thread: the message we want in the help bar (None = the game's).
 pub fn set_help(text:Option<&str>){unsafe{*core::ptr::addr_of_mut!(HELP_WANT)=text.map(alloc::string::String::from);}}
 /// Game thread, once a frame: show our message or give the bar back.
 pub fn update_help(){
  if !help_available(){return}
  let want=unsafe{(*core::ptr::addr_of!(HELP_WANT)).clone()};
  let shown=unsafe{&mut *core::ptr::addr_of_mut!(HELP_SHOWN)};
  if want==*shown{return}
  match want.as_deref(){
   Some(text)=>{
    unsafe{let t=&mut *core::ptr::addr_of_mut!(HELP_TEXT);t.clear();t.extend_from_slice(text.as_bytes());t.push(0);}
    HELP_OURS.store(true,Ordering::Release);call_help([4,0,MSG_FILE,HELP_ID,0,0,0,0,0,0,0,0,0,0]);
   }
   None=>{HELP_OURS.store(false,Ordering::Release);call_help(core::array::from_fn(|i|HELP_LAST[i].load(Ordering::Relaxed)));}
  }
  *shown=want;
 }
 // While the wishlist is open over the start menu, the menu's panels are
 // hidden with the game's own hide-if-shown (0x56E04C) and shown again with
 // show-if-hidden (0x56DF00): panel visibility is component +0x18. Only the
 // panels we hid come back.
 static HIDDEN:AtomicU32=AtomicU32::new(0);
 static HIDDEN_OWNER:AtomicU32=AtomicU32::new(0);
 static HIDE_READY:AtomicBool=AtomicBool::new(false);
 fn open_menu()->Option<u32>{
  let base=forge::mem::text_addr();
  [&QUEST,&LOBBY].iter().map(|p|p.load(Ordering::Relaxed)).find(|&o|is_menu(o,base)&&unsafe{rd(o+0x48)}!=u32::MAX&&matches!(unsafe{rd(o+0x38)},2|3))
 }
 /// Game thread, once a frame: hide the start menu under our window.
 pub fn hide_menu(hide:bool){
  if !HIDE_READY.load(Ordering::Acquire){return}
  let base=forge::mem::text_addr();
  let component=|owner:u32,i:u32|->u32{let panes=unsafe{rd(owner+0x34)};if valid(panes){unsafe{rd(panes+i*4)}}else{0}};
  let hidden=HIDDEN.load(Ordering::Acquire);
  if hide&&hidden==0{
   let Some(owner)=open_menu() else{return};
   let mut mask=0;
   for i in 0..4u32{
    let c=component(owner,i);if !valid(c)||unsafe{rb(c+0x18)}==0{continue}
    let f:extern "C" fn(u32)=unsafe{core::mem::transmute(base+0x56e04c)};f(c);mask|=1<<i;
   }
   HIDDEN_OWNER.store(owner,Ordering::Release);HIDDEN.store(mask,Ordering::Release);
  }else if !hide&&hidden!=0{
   let owner=HIDDEN_OWNER.load(Ordering::Acquire);HIDDEN.store(0,Ordering::Release);
   if open_menu()!=Some(owner){return}
   for i in 0..4u32{
    if hidden&(1<<i)==0{continue}
    let c=component(owner,i);if !valid(c){continue}
    let f:extern "C" fn(u32)=unsafe{core::mem::transmute(base+0x56df00)};f(c);
   }
  }
 }
 // The button prompts at the bottom right (e.g. the Smithy's sort prompt) are the
 // game's cUIOUnderKeyAssign (vtable +1777C60). The common UI owner at menu manager
 // +0x148 keeps its parts in the slot table at +0x34: slot 4 is the help bar the help
 // function (0x65E5BC) fills, slot 5 the prompt column. While our window is open the
 // column is hidden with the same hide-if-shown, re-hidden if the game shows it again,
 // and shown on close only if we hid it.
 const MENU_MANAGER:u32=0x1899458;const KEY_ASSIGN_VTABLE:u32=0x1777c60;
 static KEYS_HIDDEN:AtomicU32=AtomicU32::new(0);
 fn key_assign(base:u32)->Option<u32>{
  let manager=unsafe{rd(base+MENU_MANAGER)};if !valid(manager){return None}
  let owner=unsafe{rd(manager+0x148)};if !valid(owner){return None}
  let slots=unsafe{rd(owner+0x34)};if !valid(slots){return None}
  let keys=unsafe{rd(slots+5*4)};
  (valid(keys)&&unsafe{rd(keys)}==base+KEY_ASSIGN_VTABLE).then_some(keys)
 }
 /// Game thread, once a frame: hide the game's button prompt column under our window.
 pub fn hide_keys(hide:bool){
  if !HIDE_READY.load(Ordering::Acquire){return}
  let base=forge::mem::text_addr();
  let hidden=KEYS_HIDDEN.load(Ordering::Acquire);
  if hide{
   let Some(keys)=key_assign(base) else{return};
   if unsafe{rb(keys+0x18)}!=0{
    let f:extern "C" fn(u32)=unsafe{core::mem::transmute(base+0x56e04c)};f(keys);
    if hidden!=keys{KEYS_HIDDEN.store(keys,Ordering::Release);if crate::diagnostics::enabled(){info!("[wishlist] game button prompts hidden ({:08X})",keys);}}
   }
  }else if hidden!=0{
   KEYS_HIDDEN.store(0,Ordering::Release);
   if key_assign(base)!=Some(hidden){return}
   let f:extern "C" fn(u32)=unsafe{core::mem::transmute(base+0x56df00)};f(hidden);
  }
 }
 /// The game's button prompt column is hidden under our window (our hints may use its place).
 pub fn keys_hidden()->bool{KEYS_HIDDEN.load(Ordering::Acquire)!=0}
 #[forge::hook(offset=0x66ed24)]
 fn update_start_menu(owner:u32)->u32{
  let base=forge::mem::text_addr();
  if valid(owner){let vtable=unsafe{rd(owner)}.wrapping_sub(base);
   match vtable{QUEST_VTABLE=>QUEST.store(owner,Ordering::Relaxed),LOBBY_VTABLE=>LOBBY.store(owner,Ordering::Relaxed),_=>{}}
  }
  let r=original!(owner);
  // The top-list cursor on our entry: A is then ours (controller).
  if is_menu(owner,base){
   let hot=our_index(owner,base).is_some_and(|index|unsafe{
    let list=owner+0xd8;
    rd(owner+0x48)==0&&selected(rh(list+0x10),rb(list+0xe),rb(list+0xd))==index&&valid(rd(owner+0xa8))
     // Dimmed (not selectable): the game handles A as 'cannot select'.
     &&rb(rd(owner+0xa8)+index*8)==1
   });
   HOT.store(hot,Ordering::Release);
  }
  r
 }
 // The help bar's description lookup (manager, kind, file, id); only our ids.
 #[forge::hook(offset=0x55bac0)]
 fn message(manager:u32,kind:u32,file:u32,id:u32)->u32{
  if kind==0&&file==MSG_FILE&&id==DESC_ID{return DESCS[LANG.load(Ordering::Relaxed) as usize%9].as_ptr() as u32}
  if kind==0&&file==MSG_FILE&&id==HELP_ID{return unsafe{(*core::ptr::addr_of!(HELP_TEXT)).as_ptr()} as u32}
  original!(manager,kind,file,id)
 }
 /// The menu cursor is on our entry: A is ours, unless another mod's window
 /// (Skill-Simulator) is in front.
 pub fn hot()->bool{HOT.load(Ordering::Acquire)&&!OTHER_OPEN.load(Ordering::Acquire)}
 static OTHER_OPEN:AtomicBool=AtomicBool::new(false);
 /// Render thread: another mod's window (Skill-Simulator) is open.
 pub fn set_other_open(open:bool){OTHER_OPEN.store(open,Ordering::Release);}
 /// The open village / hall menu shows our entry (its shortcut bar is then
 /// not needed there; the field menu keeps the bar).
 pub fn entry()->bool{
  let base=forge::mem::text_addr();let owner=LOBBY.load(Ordering::Relaxed);
  ENTRY.load(Ordering::Acquire)&&our_index(owner,base).is_some()&&unsafe{rd(owner+0x48)}!=u32::MAX
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
 fn site(base:u32,offset:u32,expected:[u32;4])->bool{
  let words=core::array::from_fn(|i|unsafe{rd(base+offset+i as u32*4)});
  crate::hook_guard::matches(words,expected,base+offset)
 }
 pub fn install(base:u32)->bool{
  // Both native vtables use this common updater; lobby's override calls it.
  // Another plugin (Skill-Simulator) may already hook it: accept that chain.
  if !site(base,0x66ed24,[0xe92d4830,0xe1a04000,0xe5940038,0xe3500003]){return false}
  if unsafe{rd(base+QUEST_VTABLE+0x24)}!=base+0x66ed24||unsafe{rd(base+LOBBY_VTABLE+0x24)}!=base+0x78f38c{return false}
  forge::install_hook!(base,update_start_menu);INSTALLED.store(true,Ordering::Release);
  if site(base,0x55bac0,[0xe92d4070,0xe1a04003,0xe1a06002,0xe1a05000]){
   forge::install_hook!(base,message);
   if site(base,0x78f784,[0xe92d4ff0,0xe24ddfb9,0xe1a04000,0xebfb7bd1])&&site(base,0x791408,[0xe92d4ff0,0xe24dd004,0xe1a04000,0xe59400a8])&&site(base,0x7918cc,[0xe59000a8,0xe7d00181,0xe12fff1e,0xe92d4bf0])
    &&unsafe{rd(base+0x625138)==0xe92d4bf0&&rd(base+0x54ee34)==0xe1c011b4&&rd(base+0x54f9c0)==0xe59f1024&&rd(base+0x7aa47c)==0xe59f1010}{
    forge::install_hook!(base,init_town);forge::install_hook!(base,fill_town);forge::install_hook!(base,pick_town);ENTRY.store(true,Ordering::Release);
   }else{info!("[wishlist] start menu entry disabled: menu signatures differ");}
   if site(base,0x65e5bc,[0xe92d41f0,0xe24dd010,0xe1a04000,0xe59f00a4]){forge::install_hook!(base,help);HELP_INSTALLED.store(true,Ordering::Release);}
   if unsafe{rd(base+0x56e04c)==0xe92d4070&&rd(base+0x56e058)==0xe5d40018&&rd(base+0x56df00)==0xe92d4070&&rd(base+0x56df0c)==0xe5d40018}{HIDE_READY.store(true,Ordering::Release);}
  }
  true
 }
}
#[cfg(not(test))]pub use native::{install,visible,hot,entry,help_available,set_help,update_help,hide_menu,hide_keys,keys_hidden,set_other_open};

#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn our_index_follows_the_count_first_seen(){
  // Alone: index 16 (page 3, first row). After another mod: the next one.
  assert_eq!(town_index(None,16),16);assert_eq!(town_index(None,17),17);
  // A later init of the same menu (count already grown) keeps the index.
  assert_eq!(town_index(Some(16),17),16);assert_eq!(town_index(Some(17),18),17);
  // A new menu object back at the game's 16: take the slot again.
  assert_eq!(town_index(Some(16),16),16);assert_eq!(town_index(Some(17),16),16);
 }
 #[test]
 fn selected_index_is_page_times_rows_plus_cursor(){
  assert_eq!(selected(2,8,0),16);assert_eq!(selected(0,8,5),5);assert_eq!(selected(2,8,1),17);
 }
 #[test]
 fn label_language_follows_the_game_text(){
  assert_eq!(detect("취급 설명서",0),4);assert_eq!(detect("取扱説明書",0),1);assert_eq!(detect("取り扱い説明書",0),3);
  assert_eq!(detect("取扱說明書",2),2);assert_eq!(detect("Manual",6),6);assert_eq!(detect("Manual",4),0);
 }
}
