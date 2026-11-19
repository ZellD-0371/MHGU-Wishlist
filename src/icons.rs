use forge::{imgui::DrawList,sys::imgui::{self as sys,ImVec2,ImTextureRef}};
use core::sync::atomic::{AtomicUsize,Ordering};
include!("icons_generated.rs");include!("prompt_icons_generated.rs");
pub const A:usize=107;pub const B:usize=108;pub const X:usize=109;pub const Y:usize=110;pub const L3:usize=111;pub const R3:usize=112;pub const ZL:usize=113;pub const ZR:usize=114;pub const DPAD:usize=115;pub const DPAD_LEFT:usize=118;pub const L:usize=116;pub const R:usize=117;
static ATLAS_UV_HEIGHT:AtomicUsize=AtomicUsize::new(ATLAS_HEIGHT as usize);
static LAYOUT:AtomicUsize=AtomicUsize::new(0);
pub fn set_layout(layout:usize){LAYOUT.store(layout.min(2),Ordering::Relaxed);}
static mut TEXTURE:Option<ImTextureRef>=None;
pub fn init(){unsafe{
 crate::diagnostics::init_status(64,false);
 let file=sys::igImFileOpen(c"app:/nativeNX/plugins/wishlist/wishlist-atlas.rgba".as_ptr(),c"rb".as_ptr());
 if file.is_null(){forge::log::prelude::error!("[wishlist] HD atlas file missing");return}
 let expected=sys::igImFileGetSize(file);
 let Some(atlas)=crate::atlas::layout(expected)else{sys::igImFileClose(file);forge::log::prelude::error!("[wishlist] invalid icon atlas size");return};
 let texture=sys::ImTextureData_ImTextureData();if texture.is_null(){sys::igImFileClose(file);return}
 sys::ImTextureData_Create(texture,sys::ImTextureFormat::RGBA32,atlas.width as i32,atlas.height as i32);
 let pixels=sys::ImTextureData_GetPixels(texture);if pixels.is_null(){sys::igImFileClose(file);sys::ImTextureData_destroy(texture);return}
 let read=sys::igImFileRead(pixels,1,expected,file);sys::igImFileClose(file);
 if read!=expected{sys::ImTextureData_destroy(texture);forge::log::prelude::error!("[wishlist] HD atlas read failed");return}
 crate::diagnostics::init_status(64,true);
 ATLAS_UV_HEIGHT.store(atlas.uv_height as usize,Ordering::Release);sys::igRegisterUserTexture(texture);TEXTURE=Some(sys::ImTextureData_GetTexRef(texture));
 if crate::diagnostics::enabled(){forge::log::prelude::info!("[wishlist] {} atlas loaded ({} x {}, {} bytes)",if atlas.low_memory{"LowMemory"}else{"HD"},atlas.width,atlas.height,expected);}
}}

pub fn draw(d:DrawList,tile:usize,x:f32,y:f32,w:f32,h:f32,tint:u32,flip:bool){
 // Controller prompts exist once per layout: twelve base slots and four extras.
 let layout=LAYOUT.load(Ordering::Relaxed);
 let tile=if (107..119).contains(&tile){tile+layout*12}else if (PAD_EXTRA..PAD_EXTRA+4).contains(&tile){tile+layout*4}else{tile};
 let Some(sprite)=ART.get(tile).or_else(||EXTRA.get(tile.wrapping_sub(EXTRA_START))) else{return};if d.raw().is_null(){return}
 let Some(texture)=(unsafe{TEXTURE}) else{return};
 let uv_height=ATLAS_UV_HEIGHT.load(Ordering::Acquire) as f32;
 let (left,right)=if flip{(sprite.x+sprite.w,sprite.x)}else{(sprite.x,sprite.x+sprite.w)};
 d.add_image(texture,ImVec2{x,y},ImVec2{x:x+w,y:y+h},ImVec2{x:left as f32/2048.,y:sprite.y as f32/uv_height},ImVec2{x:right as f32/2048.,y:(sprite.y+sprite.h) as f32/uv_height},tint);
}
pub fn equipment(kind:u8)->usize{if kind==25{14}else{*EQUIP.get(kind as usize).unwrap_or(&46)}}
pub fn material(id:u16)->(usize,u32){ITEMS.get(id as usize).map_or((46,0xffffffff),|r|(r.0 as usize,PALETTE[r.1 as usize]))}
/// Atlas tile of a keyboard/mouse prompt by its Kenney name (e.g. "keyboard_f5",
/// "mouse_left"), for keyboard and mouse prompts.
#[allow(dead_code)] // keyboard/mouse prompts are drawn once keyboard input lands
pub fn key_tile(name:&str)->Option<usize>{KEYS.iter().position(|k|*k==name).map(|i|KEY_START+i)}
