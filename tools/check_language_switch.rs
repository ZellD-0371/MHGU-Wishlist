//! Run the production pre-frame language coordinator against a host adapter.
extern crate alloc;
extern crate self as forge;
use std::{ffi::CString,sync::{Mutex,atomic::{AtomicUsize,Ordering}}};
#[macro_export]macro_rules! error{($($t:tt)*)=>{{}}}
pub mod log{pub mod prelude{pub use crate::error;}}
#[path="../src/model.rs"]mod model;
#[path="../src/binding_model.rs"]mod binding_model;
#[path="../src/sorting.rs"]mod sorting;
#[path="../src/text_blob.rs"]mod text_blob;
#[path="../src/language_pack.rs"]mod language_pack;
#[path="language_test_source.rs"]mod language_test_source;
#[path="../src/language_ui.rs"]mod language_ui;
const CODES:[&str;9]=["en","zh-Hans","zh-Hant","ja","ko","de","fr","it","es"];
static FAULT:AtomicUsize=AtomicUsize::new(99);
static FONT_LOADS:AtomicUsize=AtomicUsize::new(0);
static LOCKED:AtomicUsize=AtomicUsize::new(0);
struct State{list:model::List,pending_preferences:Option<model::List>,settings_loaded:bool,settings_notice:u8}
struct Shared(Mutex<Option<State>>);
static STATE:Shared=Shared(Mutex::new(None));
impl Shared{fn with<R>(&self,f:impl FnOnce(&mut State)->R)->R{f(self.0.lock().unwrap().as_mut().unwrap())}}
pub mod imgui{
 #[derive(Clone,Copy)]pub struct ImFont;
 impl ImFont{pub fn load(_: &std::ffi::CStr)->Option<Self>{assert_eq!(super::LOCKED.load(super::Ordering::SeqCst),0,"font loaded inside a frame");super::FONT_LOADS.fetch_add(1,super::Ordering::SeqCst);Some(Self)}}
}
mod icons{pub fn init(){}}
mod ui{pub(crate)static mut FONT:Option<crate::imgui::ImFont>=None;}
pub mod sys{pub mod imgui{
 pub type ImGuiID=u32;pub enum ImGuiContext{}pub enum ImGuiContextHookType{NewFramePre}
 pub struct ImGuiContextHook{pub HookId:u32,pub Type:ImGuiContextHookType,pub Owner:u32,pub Callback:Option<unsafe extern "C" fn(*mut ImGuiContext,*mut ImGuiContextHook)>,pub UserData:*mut std::ffi::c_void}
 static CALLBACK:std::sync::Mutex<Option<unsafe extern "C" fn(*mut ImGuiContext,*mut ImGuiContextHook)>>=std::sync::Mutex::new(None);
 pub unsafe fn igGetCurrentContext()->*mut ImGuiContext{1 as *mut ImGuiContext}
 pub unsafe fn igAddContextHook(_: *mut ImGuiContext,h:*const ImGuiContextHook)->u32{*CALLBACK.lock().unwrap()=unsafe{(*h).Callback};1}
 pub fn frame(){super::super::LOCKED.store(0,super::super::Ordering::SeqCst);let callback=*CALLBACK.lock().unwrap();unsafe{callback.unwrap()(igGetCurrentContext(),std::ptr::null_mut());}super::super::LOCKED.store(1,super::super::Ordering::SeqCst);}
}}
mod language{
 use super::*;static STORE:Mutex<language_pack::Store>=Mutex::new(language_pack::Store{active:None});
 pub fn reset(){STORE.lock().unwrap().active=None;}
 pub fn current()->Option<usize>{STORE.lock().unwrap().active.as_ref().map(|p|p.language)}
 pub fn path(_:usize,_:usize)->Option<CString>{CString::new("assets/languages/font.shared.otf").ok()}
 pub fn font_valid()->bool{true}
 pub fn prepare(lang:usize)->Result<language_pack::Pack,()>{let mut read=|kind|{
  if FAULT.load(Ordering::SeqCst)==kind{return Err(())}let code=CODES.get(lang).ok_or(())?;
  let mut data=std::fs::read(format!("assets/languages/{}.{}.bin",if kind==0{"names"}else{"text"},code)).map_err(|_|())?;
  if FAULT.load(Ordering::SeqCst)==10+kind{data[0]^=1;}Ok(data)
 };let names=read(0)?;let text=read(1)?;language_pack::Pack::prepare(lang,names,Box::new(language_test_source::Memory(text)))}
 pub fn commit(pack:language_pack::Pack){drop(STORE.lock().unwrap().commit(pack));}
 pub fn text(lang:usize)->String{STORE.lock().unwrap().text(false,1,2,lang)}
}
#[test]fn production_language_switch_is_deferred_atomic_failure_safe_and_font_stable(){
 let mut list=model::List::default();list.language=4;list.insert(7,model::Recipe{kind:9,id:51,materials:[model::Material{id:1,count:1};4],..Default::default()}).unwrap();
 *STATE.0.lock().unwrap()=Some(State{list,pending_preferences:None,settings_loaded:true,settings_notice:0});
 language_ui::init();assert_eq!(language::current(),Some(4));assert_eq!(FONT_LOADS.load(Ordering::SeqCst),1);
 let old_owned=language::text(4);assert!(!old_owned.is_empty());
 for fault in [0,1,10,11]{
  FAULT.store(fault,Ordering::SeqCst);let mut next=STATE.with(|s|s.list.clone());next.language=3;
  LOCKED.store(1,Ordering::SeqCst);language_ui::request(&next);assert_eq!(STATE.with(|s|s.list.language),4);assert_eq!(language::current(),Some(4));
  sys::imgui::frame();assert_eq!(STATE.with(|s|s.list.language),4);assert_eq!(STATE.with(|s|s.settings_notice),4);assert_eq!(language::text(4),old_owned);
 }
 FAULT.store(99,Ordering::SeqCst);
 for lang in (0..9).cycle().take(90){let mut next=STATE.with(|s|s.list.clone());next.language=lang;language_ui::request(&next);sys::imgui::frame();assert_eq!(STATE.with(|s|s.list.language),lang);assert_eq!(language::current(),Some(lang));assert!(STATE.with(|s|s.list.slots[7].is_some()));}
 assert_eq!(FONT_LOADS.load(Ordering::SeqCst),1,"language changes must not append fonts");let mut korean=STATE.with(|s|s.list.clone());korean.language=4;language_ui::request(&korean);sys::imgui::frame();assert_eq!(language::text(4),old_owned);
 // Cancelling a queued language change by restoring current preferences must discard it.
 let current=STATE.with(|s|s.list.clone());let mut other=current.clone();other.language=(current.language+1)%9;language_ui::request(&other);language_ui::request(&current);sys::imgui::frame();assert_eq!(language::current(),Some(current.language));
}
