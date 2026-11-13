//! Language text changes run before NewFrame, outside every widget scope.
use crate::{STATE,model::List,ui::FONT};
use forge::{imgui,sys::imgui as sys};
static mut CONTEXT:*mut sys::ImGuiContext=core::ptr::null_mut();
static mut HOOK:sys::ImGuiID=0;
fn fail(){STATE.with(|s|s.settings_notice=4);forge::log::prelude::error!("[wishlist] language resources could not be loaded; previous language retained");}
fn switch(next:&List,edited:bool)->bool{
 let lang=next.language;
 if crate::language::current()==Some(lang){STATE.with(|s|{s.list.copy_preferences(next);if edited{s.settings_loaded=true;s.settings_notice=1;}});return true}
 let Ok(pack)=crate::language::prepare(lang)else{fail();return false};
 // Fonts are shared and never replaced. Commit text and its language together.
 STATE.with(|s|{crate::language::commit(pack);s.list.copy_preferences(next);if edited{s.settings_loaded=true;s.settings_notice=1;}});
 true
}
unsafe extern "C" fn before_frame(_ctx:*mut sys::ImGuiContext,_hook:*mut sys::ImGuiContextHook){
 let request=STATE.with(|s|{if let Some(next)=s.pending_preferences.take(){return Some((next,true))}let active=crate::language::current();if active.is_some()&&active!=Some(s.list.language){Some((s.list.clone(),false))}else{None}});
 if let Some((next,edited))=request{if !switch(&next,edited){if let Some(active)=crate::language::current(){STATE.with(|s|s.list.language=active);}}}
}
pub fn init(){unsafe{
 let ctx=sys::igGetCurrentContext();if ctx==CONTEXT&&HOOK!=0{return}CONTEXT=ctx;HOOK=0;FONT=None;crate::language::reset();crate::icons::init();
 if crate::language::font_valid(){if let Some(path)=crate::language::path(0,2){FONT=imgui::ImFont::load(&path);}}
 if {let font=FONT;font.is_none()}{fail();STATE.with(|s|s.list.language=0);return}
 let next=STATE.with(|s|s.list.clone());
 if !switch(&next,false){let mut fallback=next;fallback.language=0;if !switch(&fallback,false){STATE.with(|s|s.list.language=0);}}
 let hook=sys::ImGuiContextHook{HookId:0,Type:sys::ImGuiContextHookType::NewFramePre,Owner:0x574c4c31,Callback:Some(before_frame),UserData:core::ptr::null_mut()};
 HOOK=sys::igAddContextHook(ctx,&hook);if HOOK==0{fail();}
}}
pub fn request(next:&List){
 if crate::language::current()!=Some(next.language){if unsafe{HOOK}==0{fail();return}STATE.with(|s|s.pending_preferences=Some(next.clone()));}
 else{STATE.with(|s|{s.pending_preferences=None;s.list.copy_preferences(next);s.settings_loaded=true;s.settings_notice=1;});}
}
