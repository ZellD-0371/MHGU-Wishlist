//! The native +/START menu, not Forge's Insert overlay.
//!
//! Hunter's Notes entry: the seventh row (index 6), shared by every mod that
//! publishes a mod menu block (mod_menu_model). The host adds it: one
//! participant shows its own name and opens directly, more show 'Mods' and a
//! list. A submenu has seven row panes only (an eighth row crashed reading past
//! them, 2026-11-20). Menu object
//! (Skill-Simulator research, 2026-11-18): +0x48 state (1 = in a submenu),
//! +0x4C submenu (5 = Hunter's Notes), +0xB0/+0xB4 name array (8-byte slots:
//! u8 selectable, u32 text) and its slot count, +0xC8/+0xCC description array
//! (12-byte slots) and its slot count, +0x109 cursor, +0x10A item count.
//! 0x670098(this, n, 1) sets the list widget's item count only; it never grows
//! the arrays (0x54EDEC, 2026-11-20). See docs/skill-simulator-menu-request.md.
pub const QUEST_VTABLE:u32=0x177aeb8;
pub const LOBBY_VTABLE:u32=0x177e424;
pub fn is_main_menu(vtable:u32,phase:u32,substate:u32,pane_visible:bool)->bool{
 matches!(vtable,QUEST_VTABLE|LOBBY_VTABLE)&&matches!(phase,2|3)&&substate==0&&pane_visible
}
/// Hunter's Notes submenu and its six game entries.
pub const NOTES:u32=5;
pub const GAME_ENTRIES:u8=6;
/// Our description message id (Skill-Simulator uses 0x7FF0), message file 9.
pub const DESC_ID:u32=0x7ff1;
pub const MSG_FILE:u32=9;
/// The shared entry's row.
pub const INDEX:u8=GAME_ENTRIES;
/// What to do with the Hunter's Notes item count this frame.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Grow{Wait,Grow,Ready}
pub fn grow(count:u8,index:u8)->Grow{
 if count<index||count>index+1{Grow::Wait}else if count==index{Grow::Grow}else{Grow::Ready}
}
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
 static REPORTED:AtomicBool=AtomicBool::new(false);
 /// The entry is the 'Mods' list (two or more participants).
 static HUB:AtomicBool=AtomicBool::new(false);
 const HUB_LABELS:[&core::ffi::CStr;9]=[c"Mods",c"模组",c"模組",c"MOD",c"모드",c"Mods",c"Mods",c"Mod",c"Mods"];
 const HUB_DESCS:[&core::ffi::CStr;9]=[
  c"Open the features of installed mods.",
  c"打开已安装模组的功能。",
  c"開啟已安裝模組的功能。",
  c"導入したMODの機能を開きます。",
  c"설치한 모드의 기능을 엽니다.",
  c"Funktionen installierter Mods öffnen.",
  c"Ouvrir les fonctions des mods installés.",
  c"Apri le funzioni delle mod installate.",
  c"Abre las funciones de los mods instalados.",
 ];
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
 /// Larger name/description arrays while Hunter's Notes is open, when the
 /// game's own are too small: (owner, names, name slots, descs, desc slots).
 const SLOTS:usize=8;
 #[repr(C,align(4))]struct Names([u8;SLOTS*8]);
 #[repr(C,align(4))]struct Descs([u8;SLOTS*12]);
 static mut NAMES:Names=Names([0;SLOTS*8]);
 static mut DESCS_BUF:Descs=Descs([0;SLOTS*12]);
 static mut SWAPPED:Option<(u32,u32,u32,u32,u32)>=None;
 unsafe fn rb(p:u32)->u8{unsafe{core::ptr::read(p as *const u8)}}
 unsafe fn wr(p:u32,v:u32){unsafe{core::ptr::write_unaligned(p as *mut u32,v)}}
 fn is_menu(owner:u32,base:u32)->bool{valid(owner)&&matches!(unsafe{rd(owner)}.wrapping_sub(base),QUEST_VTABLE|LOBBY_VTABLE)}
 fn in_notes(owner:u32)->bool{unsafe{rd(owner+0x48)==1&&rd(owner+0x4c)==NOTES}}
 /// Give the game its own arrays back, with whatever was written meanwhile.
 fn restore(force:bool){
  let Some((owner,names,name_slots,descs,desc_slots))=(unsafe{*core::ptr::addr_of!(SWAPPED)}) else{return};
  if !force&&valid(owner)&&in_notes(owner){return}
  unsafe{
   if valid(owner)&&rd(owner+0xb0)==core::ptr::addr_of!(NAMES) as u32{
    core::ptr::copy_nonoverlapping(core::ptr::addr_of!(NAMES) as *const u8,names as *mut u8,(name_slots as usize).min(SLOTS)*8);
    core::ptr::copy_nonoverlapping(core::ptr::addr_of!(DESCS_BUF) as *const u8,descs as *mut u8,(desc_slots as usize).min(SLOTS)*12);
    wr(owner+0xb0,names);wr(owner+0xb4,name_slots);wr(owner+0xc8,descs);wr(owner+0xcc,desc_slots);
   }
   *core::ptr::addr_of_mut!(SWAPPED)=None;
  }
 }
 /// Room for `need` slots in both arrays, swapping in ours if needed.
 fn ensure_slots(owner:u32,need:u32)->bool{
  unsafe{
   let (names,name_slots,descs,desc_slots)=(rd(owner+0xb0),rd(owner+0xb4),rd(owner+0xc8),rd(owner+0xcc));
   if !REPORTED.swap(true,Ordering::Relaxed){info!("[wishlist] Hunter's Notes arrays: {} name slots, {} description slots",name_slots,desc_slots);}
   if name_slots>=need&&desc_slots>=need{return valid(names)&&valid(descs)}
   if need as usize>SLOTS||!valid(names)||!valid(descs)||name_slots<u32::from(GAME_ENTRIES)||desc_slots<u32::from(GAME_ENTRIES){return false}
   restore(true);
   core::ptr::write_bytes(core::ptr::addr_of_mut!(NAMES) as *mut u8,0,SLOTS*8);
   core::ptr::write_bytes(core::ptr::addr_of_mut!(DESCS_BUF) as *mut u8,0,SLOTS*12);
   core::ptr::copy_nonoverlapping(names as *const u8,core::ptr::addr_of_mut!(NAMES) as *mut u8,(name_slots as usize).min(SLOTS)*8);
   core::ptr::copy_nonoverlapping(descs as *const u8,core::ptr::addr_of_mut!(DESCS_BUF) as *mut u8,(desc_slots as usize).min(SLOTS)*12);
   *core::ptr::addr_of_mut!(SWAPPED)=Some((owner,names,name_slots,descs,desc_slots));
   wr(owner+0xb0,core::ptr::addr_of!(NAMES) as u32);wr(owner+0xb4,SLOTS as u32);
   wr(owner+0xc8,core::ptr::addr_of!(DESCS_BUF) as u32);wr(owner+0xcc,SLOTS as u32);
  }
  true
 }
 fn text(p:u32)->alloc::string::String{
  if !valid(p){return alloc::string::String::new()}
  let mut v=alloc::vec::Vec::new();for i in 0..64{let b=unsafe{rb(p+i)};if b==0{break}v.push(b);}
  alloc::string::String::from_utf8_lossy(&v).into_owned()
 }
 /// Our name and description in slot `index`, following 'Manual' (slot 5).
 fn write_entry(owner:u32,index:u32){
  unsafe{
   let (names,descs)=(rd(owner+0xb0),rd(owner+0xc8));
   if !valid(names)||!valid(descs){return}
   let plugin=crate::STATE.with(|s|s.list.language);
   let lang=detect(&text(rd(names+5*8+4)),plugin);LANG.store(lang as u32,Ordering::Relaxed);
   let e=names+index*8;
   let label=if HUB.load(Ordering::Relaxed){HUB_LABELS[lang]}else{LABELS[lang]};
   core::ptr::write(e as *mut u8,rb(names+5*8));wr(e+4,label.as_ptr() as u32);
   let (src,dst)=(descs+5*12,descs+index*12);
   wr(dst,rd(src));core::ptr::write((dst+4) as *mut u8,0);core::ptr::write((dst+5) as *mut u8,rb(src+5));
   core::ptr::write_unaligned((dst+6) as *mut u16,MSG_FILE as u16);core::ptr::write_unaligned((dst+8) as *mut u16,DESC_ID as u16);core::ptr::write_unaligned((dst+10) as *mut u16,0xffff);
  }
 }
 /// After the game's menu update: grow Hunter's Notes and keep our entry.
 fn notes_entry(owner:u32,base:u32){
  restore(false);
  // A submenu shows at most seven rows (the layout has seven row panes; an
  // eighth row read past them and crashed, 2026-11-20). With Skill-Simulator
  // in the seventh row there is no room for us.
  if !entry()||!is_menu(owner,base)||!in_notes(owner){
   if is_menu(owner,base){HOT.store(false,Ordering::Release);}return
  }
  let index=INDEX;
  let count=unsafe{rb(owner+0x10a)};
  let ready=match grow(count,index){
   Grow::Wait=>false,
   Grow::Grow=>ensure_slots(owner,u32::from(index)+1)&&{
    let set_count:extern "C" fn(u32,u32,u32)=unsafe{core::mem::transmute(base+0x670098)};
    set_count(owner,u32::from(index)+1,1);true
   },
   Grow::Ready=>unsafe{rd(owner+0xb4)>u32::from(index)&&rd(owner+0xcc)>u32::from(index)},
  };
  if !ready{HOT.store(false,Ordering::Release);return}
  write_entry(owner,u32::from(index));
  let hot=unsafe{rb(owner+0x109)==index&&rb(rd(owner+0xb0)+u32::from(index)*8)==1};
  HOT.store(hot,Ordering::Release);
 }

 // The start menu's third panel ('sub2', layout Null_sub2 / mesS_07-08): state
 // +0x48 = 2, list widget +0x120 (cursor +0x12D, rows +0x12E, page +0x130),
 // name array +0xB8/+0xBC, description array +0xD0/+0xD4, panel component 3.
 // GU never opens it; the game still handles its cursor and B (back to the
 // submenu, closing component 3). We open it as the 'Mods' list, and take A.
 // Static analysis 2026-11-20: 0x66F0A8 state switch, 0x66FA04 sub2 input,
 // 0x6700A4 sub2 item count, 0x65CBA0 open component, 0x66F7CC sub2 help text.
 const SUB2_ROWS:usize=2;
 static OPEN_SUB2:AtomicBool=AtomicBool::new(false);
 static SUB2:AtomicBool=AtomicBool::new(false);
 static SUB2_ADDR:[AtomicU32;SUB2_ROWS]=[const{AtomicU32::new(0)};SUB2_ROWS];
 static SUB2_COUNT:AtomicU32=AtomicU32::new(0);
 static SUB2_INDEX:AtomicU32=AtomicU32::new(u32::MAX);
 fn in_sub2(owner:u32)->bool{unsafe{rd(owner+0x48)==2&&rd(owner+0x4c)==NOTES}}
 /// Open the 'Mods' panel with the participants (at most two rows for now).
 fn open_sub2(owner:u32,base:u32){
  let entries=crate::mod_menu::entries();let n=entries.len().min(SUB2_ROWS);
  if n==0{return}
  unsafe{
   let (names,name_slots,descs,desc_slots,sub_descs)=(rd(owner+0xb8),rd(owner+0xbc),rd(owner+0xd0),rd(owner+0xd4),rd(owner+0xc8));
   if !REPORTED_SUB2.swap(true,Ordering::Relaxed){info!("[wishlist] mods panel arrays: {} name slots, {} description slots",name_slots,desc_slots);}
   if !valid(names)||!valid(descs)||!valid(sub_descs)||(name_slots as usize)<n||(desc_slots as usize)<n{return}
   let lang=LANG.load(Ordering::Relaxed) as usize%9;
   for (i,e) in entries.iter().take(n).enumerate(){
    let label=if e.names[lang]!=0{e.names[lang]}else{e.names[0]};
    let slot=names+i as u32*8;core::ptr::write(slot as *mut u8,u8::from(e.ready));wr(slot+4,label);
    // Description: the 'Manual' row's message object, our message id per row.
    let (src,dst)=(sub_descs+5*12,descs+i as u32*12);
    wr(dst,rd(src));core::ptr::write((dst+4) as *mut u8,0);core::ptr::write((dst+5) as *mut u8,rb(src+5));
    core::ptr::write_unaligned((dst+6) as *mut u16,MSG_FILE as u16);core::ptr::write_unaligned((dst+8) as *mut u16,(DESC_ID+1+i as u32) as u16);core::ptr::write_unaligned((dst+10) as *mut u16,0xffff);
    SUB2_ADDR[i].store(e.address,Ordering::Release);
   }
   SUB2_COUNT.store(n as u32,Ordering::Release);
   let set_count:extern "C" fn(u32,u32,u32)=core::mem::transmute(base+0x6700a4);set_count(owner,n as u32,1);
   wr(owner+0x48,2);
   let open:extern "C" fn(u32,u32)->u32=core::mem::transmute(base+0x65cba0);open(owner,3);
   align_sub2(owner,base);
   // The decide sound the game plays when it opens a submenu (0x66F178).
   let sound:extern "C" fn(u32,u32)=core::mem::transmute(base+0x54f260);sound(0,1);
  }
  SUB2.store(true,Ordering::Release);
 }
 static REPORTED_SUB2:AtomicBool=AtomicBool::new(false);
 // The game is hard-float; this plugin is soft-float: bridges for the
 // float-in-s0 getter/setter of menu layout object positions.
 /// f(a0, a1), returning its s0 result as bits.
 #[unsafe(naked)]
 unsafe extern "C" fn call_ret_f32(_f:u32,_a0:u32,_a1:u32)->u32{
  core::arch::naked_asm!("push {{r4, lr}}","mov r4, r0","mov r0, r1","mov r1, r2","blx r4","vmov r0, s0","pop {{r4, pc}}");
 }
 /// f(a0, a1) with s0 = the float `bits`.
 #[unsafe(naked)]
 unsafe extern "C" fn call_with_f32_2(_f:u32,_a0:u32,_a1:u32,_bits:u32){
  core::arch::naked_asm!("mov ip, r0","mov r0, r1","mov r1, r2","vmov s0, r3","bx ip");
 }
 /// Line sub2's top up with the submenu's. Both panels share one layout of 43
 /// objects; Null_sub is object 15 (what 0x62CB68 sets) and Null_sub2 is 33
 /// (layout default y 94, the top of the screen). Object 32 is the submenu
 /// cursor animation the game moves every frame (Recomp log 2026-11-20).
 /// Each frame after the game's update: sub2's y = the submenu's y.
 const NULL_SUB:u32=15;
 const NULL_SUB2:u32=33;
 unsafe fn align_sub2(owner:u32,base:u32){unsafe{
  let panes=rd(owner+0x34);if !valid(panes){return}
  let sub2=rd(panes+12);if !valid(sub2){return}
  let layout=rd(sub2+0x24);if !valid(layout)||rd(layout+0x298)<=NULL_SUB2{return}
  let y=f32::from_bits(call_ret_f32(base+0x545af8,layout,NULL_SUB));
  if y.is_finite(){call_with_f32_2(base+0x545ac0,layout,NULL_SUB2,y.to_bits());}
 }}
 /// After the game's update: open the panel when asked, follow its cursor.
 fn sub2_update(owner:u32,base:u32){
  if !is_menu(owner,base){return}
  if OPEN_SUB2.swap(false,Ordering::AcqRel)&&in_notes(owner)&&unsafe{rb(owner+0x109)}==INDEX{open_sub2(owner,base);}
  if !SUB2.load(Ordering::Acquire){return}
  if !in_sub2(owner){SUB2.store(false,Ordering::Release);SUB2_INDEX.store(u32::MAX,Ordering::Release);return}
  unsafe{align_sub2(owner,base);}
  let index=unsafe{u32::from(core::ptr::read_unaligned((owner+0x130) as *const u16))*u32::from(rb(owner+0x12e))+u32::from(rb(owner+0x12d))};
  SUB2_INDEX.store(if index<SUB2_COUNT.load(Ordering::Acquire){index}else{u32::MAX},Ordering::Release);
 }
 // The common help bar at the bottom (0x65E5BC: flags, kind, file, id, and two
 // stack words; it draws into component 4 of the HUD object). While the
 // wishlist is open it shows our message instead, in the game's own panel and
 // font; the game's latest request is kept and restored when we close.
 const HELP_ID:u32=0x7ff8;
 static HELP_OURS:AtomicBool=AtomicBool::new(false);
 static HELP_INSTALLED:AtomicBool=AtomicBool::new(false);
 static HELP_LAST:[AtomicU32;6]=[const{AtomicU32::new(0)};6];
 static HELP_SEEN:AtomicBool=AtomicBool::new(false);
 static mut HELP_TEXT:alloc::vec::Vec<u8>=alloc::vec::Vec::new();
 static mut HELP_WANT:Option<alloc::string::String>=None;
 static mut HELP_SHOWN:Option<alloc::string::String>=None;
 #[forge::hook(offset=0x65e5bc)]
 fn help(flags:u32,kind:u32,file:u32,id:u32,a:u32,b:u32)->u32{
  if HELP_CALLING.load(Ordering::Acquire){return original!(flags,kind,file,id,a,b)}
  for (slot,v) in HELP_LAST.iter().zip([flags,kind,file,id,a,b]){slot.store(v,Ordering::Relaxed);}
  HELP_SEEN.store(true,Ordering::Release);
  // While ours is shown, the game's own updates wait for our window to close.
  if HELP_OURS.load(Ordering::Acquire){return 0}
  original!(flags,kind,file,id,a,b)
 }
 static HELP_CALLING:AtomicBool=AtomicBool::new(false);
 fn call_help(args:[u32;6]){
  let f:extern "C" fn(u32,u32,u32,u32,u32,u32)->u32=unsafe{core::mem::transmute(forge::mem::text_addr()+0x65e5bc)};
  HELP_CALLING.store(true,Ordering::Release);f(args[0],args[1],args[2],args[3],args[4],args[5]);HELP_CALLING.store(false,Ordering::Release);
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
    HELP_OURS.store(true,Ordering::Release);call_help([4,0,MSG_FILE,HELP_ID,0,0]);
   }
   None=>{HELP_OURS.store(false,Ordering::Release);call_help(core::array::from_fn(|i|HELP_LAST[i].load(Ordering::Relaxed)));}
  }
  *shown=want;
 }
 // While the wishlist is open over the start menu, the menu's panels (main,
 // its side component, submenu, sub2) are hidden with the game's own
 // hide-if-shown (0x56E04C) and shown again with show-if-hidden (0x56DF00):
 // panel visibility is component +0x18. Only the panels we hid come back.
 static HIDDEN:AtomicU32=AtomicU32::new(0);
 static HIDDEN_OWNER:AtomicU32=AtomicU32::new(0);
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
 static HIDE_READY:AtomicBool=AtomicBool::new(false);
 /// Ask for the 'Mods' panel (from the decision on our Hunter's Notes row).
 pub fn request_mods_panel(){OPEN_SUB2.store(true,Ordering::Release);}
 /// The participant under the 'Mods' panel cursor, while it is open.
 pub fn panel_choice()->Option<u32>{
  let i=SUB2_INDEX.load(Ordering::Acquire);
  (SUB2.load(Ordering::Acquire)&&(i as usize)<SUB2_ROWS).then(||SUB2_ADDR[i as usize].load(Ordering::Acquire))
 }
 #[forge::hook(offset=0x66ed24)]
 fn update_start_menu(owner:u32)->u32{
  let base=forge::mem::text_addr();
  if valid(owner){let vtable=unsafe{rd(owner)}.wrapping_sub(base);
   match vtable{QUEST_VTABLE=>QUEST.store(owner,Ordering::Relaxed),LOBBY_VTABLE=>LOBBY.store(owner,Ordering::Relaxed),_=>{}}
  }
  let r=original!(owner);
  notes_entry(owner,base);
  sub2_update(owner,base);
  r
 }
 // The help bar's description lookup (manager, kind, file, id); only our id.
 #[forge::hook(offset=0x55bac0)]
 fn message(manager:u32,kind:u32,file:u32,id:u32)->u32{
  if kind==0&&file==MSG_FILE&&id==DESC_ID{let lang=LANG.load(Ordering::Relaxed) as usize%9;return if HUB.load(Ordering::Relaxed){HUB_DESCS[lang]}else{DESCS[lang]}.as_ptr() as u32}
  if kind==0&&file==MSG_FILE&&id==HELP_ID{return unsafe{(*core::ptr::addr_of!(HELP_TEXT)).as_ptr()} as u32}
  // A 'Mods' panel row: that participant's own description.
  if kind==0&&file==MSG_FILE&&(DESC_ID+1..=DESC_ID+SUB2_ROWS as u32).contains(&id){
   let address=SUB2_ADDR[(id-DESC_ID-1) as usize].load(Ordering::Acquire);
   if let Some(text)=crate::mod_menu::description(address,LANG.load(Ordering::Relaxed) as usize%9){return text}
  }
  original!(manager,kind,file,id)
 }
 /// The game menu cursor is on our Hunter's Notes entry or in our panel: A is
 /// ours, unless another mod's window (opened from the panel) is in front.
 pub fn hot()->bool{(HOT.load(Ordering::Acquire)||panel_choice().is_some())&&!OTHER_OPEN.load(Ordering::Acquire)}
 static OTHER_OPEN:AtomicBool=AtomicBool::new(false);
 /// Render thread: another mod's window (Skill-Simulator) is open.
 pub fn set_other_open(open:bool){OTHER_OPEN.store(open,Ordering::Release);}
 /// Our Hunter's Notes entry is active (its shortcut bar is then not needed).
 pub fn entry()->bool{ENTRY.load(Ordering::Acquire)&&ACTIVE.load(Ordering::Acquire)}
 static ACTIVE:AtomicBool=AtomicBool::new(false);
 /// Once a frame: whether we host the entry, and as a list or alone. An older
 /// Skill-Simulator without a block keeps its own seventh row: stay out.
 pub fn refresh(){
  let entries=crate::mod_menu::entries();
  let host=crate::mod_menu::hosting(&entries)&&!(crate::sksim::present()&&entries.len()<2);
  HUB.store(entries.len()>1,Ordering::Relaxed);ACTIVE.store(host,Ordering::Release);
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
  if site(base,0x55bac0,[0xe92d4070,0xe1a04003,0xe1a06002,0xe1a05000])&&site(base,0x670098,[0xe28000fc,0xe3a03000,0xeafb7b51,0xe2800e12])&&site(base,0x6700a4,[0xe2800e12,0xe3a03000,0xeafb7b4e,0xe92d4800])&&site(base,0x65cba0,[0xe92d4010,0xe5900034,0xe7904101,0xe3540000])&&site(base,0x545af8,[0xe5902298,0xe1520001,0x9a000004,0xe59000f8])&&site(base,0x545ac0,[0xe5902298,0xe1520001,0x9a000003,0xe59000f8])&&unsafe{rd(base+0x54f260)}==0xe3500006{
   forge::install_hook!(base,message);ENTRY.store(true,Ordering::Release);
   if site(base,0x65e5bc,[0xe92d41f0,0xe24dd010,0xe1a04000,0xe59f00a4]){forge::install_hook!(base,help);HELP_INSTALLED.store(true,Ordering::Release);}
   if unsafe{rd(base+0x56e04c)==0xe92d4070&&rd(base+0x56e058)==0xe5d40018&&rd(base+0x56df00)==0xe92d4070&&rd(base+0x56df0c)==0xe5d40018}{HIDE_READY.store(true,Ordering::Release);}
  }else{info!("[wishlist] Hunter's Notes entry disabled: menu signatures differ");}
  true
 }
}
#[cfg(not(test))]pub use native::{install,visible,hot,entry,refresh,request_mods_panel,panel_choice,help_available,set_help,update_help,hide_menu,set_other_open};

#[cfg(test)]
mod tests{
 use super::*;
 #[test]
 fn the_shared_entry_takes_the_seventh_and_last_row(){
  assert_eq!(INDEX,6);
  assert_eq!(grow(6,INDEX),Grow::Grow);assert_eq!(grow(7,INDEX),Grow::Ready);
  // Another list (or a count we did not expect) is left alone: never an eighth row.
  assert_eq!(grow(3,INDEX),Grow::Wait);assert_eq!(grow(8,INDEX),Grow::Wait);
 }
 #[test]
 fn label_language_follows_the_game_text(){
  assert_eq!(detect("취급 설명서",0),4);assert_eq!(detect("取扱説明書",0),1);assert_eq!(detect("取り扱い説明書",0),3);
  assert_eq!(detect("取扱說明書",2),2);assert_eq!(detect("Manual",6),6);assert_eq!(detect("Manual",4),0);
 }
}
