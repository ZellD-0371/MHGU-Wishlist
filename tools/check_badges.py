"""Exercise the production badge code against deterministic guest-memory fixtures.
This does not emulate the game loop or execute ARM detours. The hook wiring audit
keeps Wishlist off the smithy entries KatSmithy guards; the replay tests protect the
open-window lookup and its state transitions.
"""
from pathlib import Path
import re, subprocess, os
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'target/host-check';OUT.mkdir(parents=True,exist_ok=True)
s=(ROOT/'src/native_badges.rs').read_text(encoding='utf-8')
# KatSmithy installs guarded detours on the smithy updater (+6F1AE8, checked on its
# first onUpdate) and the equipment-box handler (+6F7220). Wishlist must not own them.
for f in (ROOT/'src').glob('*.rs'):
 code=f.read_text(encoding='utf-8')
 for offset in ('0x6f1ae8','0x6f7220'):
  assert not re.search(r'#\[forge::hook\(offset='+offset+r'\)\]',code,re.I),f'{f.name} hooks KatSmithy-guarded +{offset[2:]}'
assert 'hunter_update' not in s,'Updater detour must stay removed'
assert 'rd(base+HUNTER_VTABLE+0x24)' in s and '!=base+0x6f1ae8' in s,'Missing vtable guard'
for sig in ('(0x55e50c,0xe594515c)','(0x55e80c,0xe584515c)','(0x55e840,0xe590015c)','(0x55e848,0x15d00038)','(0x6f0044,0xe3a02003)','(0x70e7d0,0xe5c04038)','rd(base+MENU_MANAGER_GOT)}!=base+MENU_MANAGER'):
 assert sig in s,f'Open-window signature {sig} missing; verify the supported game binary'
assert re.search(r'fn grid_owner\(base:u32\)->Option<u32>\{\s*let parent=open_hunter\(base\)',s),'Equipment-box parent must come from the open-window slot'
s=re.sub(r'#\[forge::hook\(offset=0x[0-9a-f]+\)\]\n','',s)
a=s.index('pub fn install(');b=s.index('fn object(',a);s=s[:a]+s[b:]
s=s.replace('core::ptr::read_unaligned(p as *const u16)','crate::native::half(p)').replace('core::ptr::read(p as *const u8)','crate::native::byte(p)')
header=r"""
extern crate alloc;
#[path="__SRC__/binding_model.rs"]mod binding_model;
#[path="__SRC__/sorting.rs"]mod sorting;
#[path="__SRC__/model.rs"]mod model;
#[path="__SRC__/badge_model.rs"]mod badge_model;
#[path="__SRC__/quest_badge_model.rs"]mod quest_badge_model;
macro_rules! original {($($args:expr),*)=>{0u32};}
mod native{
 use std::{collections::BTreeMap,sync::{Mutex,OnceLock}};
 fn memory()-> &'static Mutex<BTreeMap<u32,u8>>{static M:OnceLock<Mutex<BTreeMap<u32,u8>>>=OnceLock::new();M.get_or_init(||Mutex::new(BTreeMap::new()))}
 pub fn reset(){memory().lock().unwrap().clear();}
 pub fn put(p:u32,v:u32){let mut m=memory().lock().unwrap();for (i,b) in v.to_le_bytes().iter().enumerate(){m.insert(p+i as u32,*b);}}
 pub fn valid(p:u32)->bool{(0x10000..0xeffff000).contains(&p)}
 pub unsafe fn byte(p:u32)->u8{*memory().lock().unwrap().get(&p).unwrap_or(&0)}
 pub unsafe fn half(p:u32)->u16{unsafe{u16::from_le_bytes([byte(p),byte(p+1)])}}
 pub unsafe fn rd(p:u32)->u32{unsafe{u32::from_le_bytes([byte(p),byte(p+1),byte(p+2),byte(p+3)])}}
 pub fn visible(p:u32,_:u32)->bool{valid(p)&&unsafe{byte(p+0x18)}==1}
}
mod production{
""".replace('__SRC__',(ROOT/'src').as_posix())
tests=r"""
#[cfg(test)]mod replay{
 use super::*;
 const BASE:u32=0x200000;
 const MANAGER:u32=0x180000;
 fn open(window:u32){crate::native::put(BASE+MENU_MANAGER,MANAGER);crate::native::put(MANAGER+OPEN_WINDOW,window);}
 fn fixture(parent:u32,pane:u32){
  use crate::native::put;
  put(parent,BASE+HUNTER_VTABLE);put(parent+4,7);put(parent+0x18,1);put(parent+0x38,HUNTER_WINDOW_KIND as u32);put(parent+0x34,parent+0x400);put(parent+0x400+8*4,pane);
  put(pane,BASE+0x1760000);put(pane+0x18,1);put(pane+0x24,pane+0x400);put(pane+0x58,pane+0x100);
  put(pane+0x114,0x080a0000); // cursor x=0,y=0,columns=10,rows=8
  let layout=pane+0x400;let table=pane+0x800;let node=pane+0x900;let objects=pane+0xa00;let icon=pane+0xb00;
  put(layout+0x298,13);put(layout+0xf8,table);put(table+12*4,node);put(node+0x54,1);put(node+0xd8,objects);put(objects+2*4,icon);put(icon+0x54,1);
  put(icon+0x40,1112_f32.to_bits());put(icon+0x44,96_f32.to_bits());
  let gear=pane+0xc00;put(gear+4,(9|(3<<5))|(51<<16));grid_icon(pane,12,2,gear,0,0);
 }
 fn list()->List{let mut l=List::default();l.insert(0,Recipe{kind:9,id:51,level:5,materials:[Material{id:1,count:1},Material::default(),Material::default(),Material::default()],..Recipe::default()}).unwrap();l}
 fn start(){clear();crate::native::reset();}
 #[test]fn quest_anchor_disappears_when_retained_row_or_list_group_is_hidden(){
  start();use crate::native::put;
  let (layout,table,node,objects,texture,root)=(0x100000,0x110000,0x120000,0x130000,0x140000,0x150000);
  put(layout+0x298,24);put(layout+0xf8,table);put(table+16*4,node);
  put(node+0x54,0x0100001b);put(node+0xd8,objects);put(node+0x68,root);
  put(objects+4,texture);put(texture+0x54,0x40001);
  // Live GUI has image parents separate from the row-group chain. They do not
  // carry group visibility bit 3 even while the row is fully visible.
  put(texture+0x68,0x160000);put(0x160000+0x54,0x02000001);put(0x160000+0x68,0x170000);put(0x170000+0x54,0x02000001);
  put(texture+0x40,734_f32.to_bits());put(texture+0x44,202_f32.to_bits());put(root+0x54,0x0100000b);
  assert!(quest_anchor(layout,16).is_some());
  // Both changes leave allocation/ID/coordinates intact, as on the level list.
  put(node+0x54,0x01000013);assert!(quest_anchor(layout,16).is_none());
  put(node+0x54,0x0100001b);put(root+0x54,0x01000003);assert!(quest_anchor(layout,16).is_none());
  put(root+0x54,0x0100000b);assert!(quest_anchor(layout,16).is_some());
 }
 #[test]fn first_grid_visit_does_not_require_opening_recipe_details(){
  start();fixture(0x100000,0x110000);assert!(markers(&list(),BASE).is_empty());
  open(0x100000);assert_eq!(markers(&list(),BASE).len(),1);assert!(selected_registered(&list(),BASE).is_some());
 }
 #[test]fn conversation_exit_and_new_owner_reentry_rebuild_the_badge(){
  start();fixture(0x100000,0x110000);open(0x100000);assert_eq!(markers(&list(),BASE).len(),1);
  destroy_ui(0x110000);destroy_ui(0x100000);open(0);assert!(grid_owner(BASE).is_none());assert!(markers(&list(),BASE).is_empty());
  fixture(0x120000,0x130000);open(0x120000);assert_eq!(markers(&list(),BASE).len(),1);
  destroy_ui(0x100000);assert_eq!(markers(&list(),BASE).len(),1);
 }
 #[test]fn delete_and_successful_upgrade_remove_pin_and_direct_removal_target(){
  start();fixture(0x100000,0x110000);open(0x100000);let mut l=list();assert_eq!(markers(&l,BASE).len(),1);
  l.slots[0]=None;assert!(markers(&l,BASE).is_empty());assert!(selected_registered(&l,BASE).is_none());
  l=list();assert!(l.remove_completed(9,51,5));assert!(markers(&l,BASE).is_empty());
 }
 #[test]fn hidden_box_other_smithy_page_and_character_change_hide_pin(){
  start();fixture(0x100000,0x110000);open(0x100000);crate::native::put(0x110018,0);assert!(markers(&list(),BASE).is_empty());
  crate::native::put(0x110018,1);crate::native::put(0x100004,3);assert!(markers(&list(),BASE).is_empty());
  crate::native::put(0x100004,7);assert_eq!(markers(&list(),BASE).len(),1);clear();assert!(markers(&list(),BASE).is_empty());
 }
 #[test]fn open_window_slot_accepts_only_the_hunter_smithy(){
  start();fixture(0x100000,0x110000);assert!(grid_owner(BASE).is_none());
  crate::native::put(BASE+MENU_MANAGER,0);crate::native::put(MANAGER+OPEN_WINDOW,0x100000);assert!(grid_owner(BASE).is_none());
  open(0x100000);crate::native::put(0x100038,4);assert!(grid_owner(BASE).is_none());
  crate::native::put(0x100038,3);crate::native::put(0x100000,BASE+PALICO_VTABLE);assert!(grid_owner(BASE).is_none());
  crate::native::put(0x100000,BASE+HUNTER_VTABLE);assert_eq!(grid_owner(BASE),Some(0x110000));
 }
 #[test]fn recipe_flow_observation_still_works_without_the_slot(){
  start();fixture(0x100000,0x110000);observe_hunter(0x100000);assert_eq!(markers(&list(),BASE).len(),1);
  open(0x120000);assert_eq!(markers(&list(),BASE).len(),1); // slot rejects a non-window, recipe parent remains
 }
 #[test]fn reused_grid_cell_cannot_keep_preceding_items_badge(){
  start();fixture(0x100000,0x110000);open(0x100000);assert_eq!(markers(&list(),BASE).len(),1);
  crate::native::put(0x110c04,(9|(3<<5))|(52<<16));grid_icon(0x110000,12,2,0x110c00,0,0);assert!(markers(&list(),BASE).is_empty());
 }
}
}
"""
p=OUT/'badge_replay.rs';p.write_text(header+s+tests,encoding='utf-8')
subprocess.run(['rustc','+nightly','--edition','2024','--test',str(p),'-Awarnings','-o',str(OUT/'badge_replay.exe')],check=True)
subprocess.run([str(OUT/'badge_replay.exe'),'--test-threads=1'],check=True)
print('Badge hook wiring audit and production-code lifecycle replay passed (mock guest memory; no live game).')
