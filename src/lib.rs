#![no_std]
extern crate alloc;
mod name_data;mod language;mod language_pack;mod language_ui;mod description_file;
mod diagnostics;mod inventory;mod sorting;mod atlas;mod text_cache;mod material_cache;mod font_validation;
mod binding_model;mod bindings;
mod hook_guard;
mod hud_layout;
mod model;mod generated;mod locales;mod native;mod controller;mod storage;mod ui;mod smithy;mod equipment;mod navigation;mod sound;mod icons;mod interaction;mod session;mod profiles;mod text_blob;mod text_data;mod presentation;mod game_menu;mod sound_cues;mod notifications;mod badge_model;mod native_badges;mod completed;
use core::{cell::UnsafeCell,sync::atomic::{AtomicBool,Ordering},ffi::c_void};
use model::{List,Recipe};
use forge::log::prelude::{info,error};
#[derive(Clone,Copy)]
struct Target{owner:u32,recipe:Recipe,cost:u32}
struct State{
 pending_preferences:Option<List>,sort:Option<sorting::Dialog>,settings_loaded:bool,settings_notice:u8,profile:Option<usize>,game_ready:bool,base:u32,installed:bool,input_ready:bool,list:List,loaded:bool,mode:u8,selected:usize,tab:usize,aggregate:interaction::AggregateFocus,preview_clock:f32,
 target:Option<Target>,frozen:Option<Recipe>,stock:[u32;4096],stock_ready:bool,material_cache:material_cache::Cache,
 notices:notifications::Queue,notice_demo:Option<notifications::Demo>,notice_test_case:usize,cooldown:u32,timer:f32,events:u32,status:u8,preview:bool,preview_panel:usize,detail_page:usize,frozen_cost:u32,focus:interaction::Focus,
}
impl State{
 fn new()->Self{Self{pending_preferences:None,sort:None,settings_loaded:false,settings_notice:0,profile:None,game_ready:false,base:0,installed:false,input_ready:false,list:List::default(),loaded:false,mode:0,selected:0,tab:0,aggregate:interaction::AggregateFocus::default(),preview_clock:0.,target:None,frozen:None,stock:[0;4096],stock_ready:false,material_cache:material_cache::Cache::default(),notices:notifications::Queue::default(),notice_demo:None,notice_test_case:2,cooldown:180,timer:0.,events:0,status:0,preview:false,preview_panel:0,detail_page:0,frozen_cost:0,focus:interaction::Focus::default()}}
 fn capture(&mut self,owner:u32,recipe:Recipe,cost:u32){if self.mode==0&&recipe.valid(){self.target=Some(Target{owner,recipe,cost});}}
 fn active(&self)->Option<Recipe>{self.target.filter(|t|native::visible(t.owner,self.base)).map(|t|t.recipe).or_else(||native_badges::selected_registered(&self.list,self.base).map(|(_,r)|r))}
 fn at_smithy(&self)->bool{self.active().is_some()||native_badges::smithy_visible(self.base)}
 fn open(&mut self,register:bool){
  if !self.game_ready||self.profile.is_none()||!self.loaded||!self.input_ready||crate::language::current().is_none(){return}
  self.sort=None;self.focus=interaction::Focus::default();self.preview=false;self.notice_demo=None;self.tab=0;self.aggregate=interaction::AggregateFocus::default();self.preview_clock=0.;self.status=0;self.detail_page=0;sound::request_open(self.list.open_sound);
  if register{if let Some(r)=self.active(){self.frozen=Some(r);self.frozen_cost=self.target.map_or(0,|t|t.cost);if let Some(slot)=self.list.registered(r){self.mode=8;self.selected=slot;self.focus.yes=false;}else{self.mode=1;self.selected=self.list.slots.iter().position(Option::is_none).unwrap_or(0)}}}else{self.frozen=None;self.mode=2;self.selected=0;}
 }
}
struct Shared{lock:AtomicBool,data:UnsafeCell<Option<State>>}
unsafe impl Sync for Shared{}
impl Shared{fn with<R>(&self,f:impl FnOnce(&mut State)->R)->R{while self.lock.compare_exchange_weak(false,true,Ordering::Acquire,Ordering::Relaxed).is_err(){core::hint::spin_loop()}let s=unsafe{&mut*self.data.get()};let r=f(s.get_or_insert_with(State::new));self.lock.store(false,Ordering::Release);r}}
static STATE:Shared=Shared{lock:AtomicBool::new(false),data:UnsafeCell::new(None)};
#[forge::entry]
fn main(){
 let base=forge::mem::text_addr();STATE.with(|s|s.base=base);
 if !{let ok=native::install(base);diagnostics::init_status(1,ok);ok}{error!("[wishlist] MHGU global 1.4.0 signatures rejected; disabled");return}
 if !{let ok=native_badges::install(base);diagnostics::init_status(2,ok);ok}{error!("[wishlist] native icon signatures rejected; badges disabled");}
 if !{let ok=completed::install(base);diagnostics::init_status(4,ok);ok}{error!("[wishlist] production completion signatures rejected; auto removal disabled");}
 #[cfg(feature="debug-ui")]
 sound::install(base);
 if !{let ok=game_menu::install(base);diagnostics::init_status(8,ok);ok}{error!("[wishlist] native + menu check failed; controller list shortcut disabled");}
 if !{let ok=session::install(base);diagnostics::init_status(16,ok);ok}{error!("[wishlist] character session signatures rejected");return}
 let input_ready=controller::install(base);diagnostics::init_status(32,input_ready);
 STATE.with(|s|{s.installed=true;s.input_ready=input_ready;});
 if !input_ready{error!("[wishlist] input capture failed; opening disabled");}
 if crate::diagnostics::enabled(){info!("[wishlist] v{}; steel workshop UI; 9 languages; 48 slots; R3 register / L3+R3 list",env!("CARGO_PKG_VERSION"));}
}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(dt:f32){
 let (base,ready)=STATE.with(|s|(s.base,s.installed));if !ready{return}
 sound::update(base);
 if !STATE.with(|s|s.settings_loaded){let mut bytes=[0;storage::MAX_BYTES];match storage::WISHLIST.read(&mut bytes){Ok(text)=>match List::decode(text){Ok(list)=>STATE.with(|s|{s.list.copy_preferences(&list);s.settings_loaded=true;}),Err(())=>diagnostics::event(3,"settings INI parse failed",[0,0,0,0])},Err(e)=>diagnostics::event(3,"settings INI read failed (error enum)",[1,e as u32,0,0])}}
 let logging=STATE.with(|s|s.list.diagnostics);diagnostics::set_enabled(logging);diagnostics::tick(dt);
 let (slot,playing)=session::update();
 if logging{let data=STATE.with(|s|[u32::from(s.installed)|(u32::from(s.input_ready)<<1)|(u32::from(s.loaded)<<2)|(u32::from(playing)<<3),slot.map_or(0,|i|i as u32+1),s.cooldown,s.list.slots.iter().flatten().count() as u32]);diagnostics::event(0,"state: flags (hooks/input/save/playing), character slot, cooldown frames, entries",data);let prefs=STATE.with(|s|[s.list.language as u32,s.list.layout as u32,s.list.ui_scale as u32,crate::language::current().map_or(0,|l|l as u32+1)]);diagnostics::event(12,"preferences: language / layout / UI scale / active language+1 (0 unavailable)",prefs);}
 STATE.with(|s|{if playing&&!s.game_ready{if crate::diagnostics::enabled(){info!("[wishlist] character session ready (slot {})",slot.unwrap_or(0)+1);}}
 s.game_ready=playing;if s.profile!=slot{completed::clear();native_badges::clear();s.profile=slot;s.loaded=false;s.sort=None;s.mode=0;s.target=None;s.stock_ready=false;s.material_cache=material_cache::Cache::default();s.notices.clear();s.notice_demo=None;s.events=0;s.cooldown=180;}if !playing{s.sort=None;s.mode=0;s.preview=false;s.events=0;s.notices.clear();s.notice_demo=None;}});
 let Some(slot)=slot else{return};
 if !STATE.with(|s|s.loaded){match storage::load_character(slot){
  Ok(mut list)=>STATE.with(|s|{if s.settings_loaded{list.copy_preferences(&s.list);}s.list=list;s.loaded=true}),
  Err(e)=>{diagnostics::event(2,"character list load failed: slot / error",[slot as u32,e as u32,0,0]);return},
 }}
 if !playing{return}
 completed::update(slot);
 let poll=STATE.with(|s|{
  // Notification time advances only while its panel is actually rendered.
  if s.cooldown>0{s.cooldown-=1;return None}
  s.timer+=dt.clamp(0.,0.1);if s.timer<0.5{return None}s.timer=0.;Some(s.base)
 });
 if let Some(base)=poll{let mut stock=[0;4096];let valid=native::inventory(base,&mut stock);STATE.with(|s|{
  let material=if s.mode==2&&s.tab==1{s.material_cache.refresh(&s.list,&s.stock,s.stock_ready,generated::PROVISIONS);s.material_cache.totals().get(s.aggregate.material).map(|m|m.id)}else{None};
  if valid!=s.stock_ready||(valid&&s.stock!=stock){s.material_cache.inventory_changed();}
  s.stock_ready=valid;if valid{s.stock=stock;if let Some(id)=material{s.material_cache.refresh(&s.list,&s.stock,true,generated::PROVISIONS);if let Some(index)=s.material_cache.totals().iter().position(|m|m.id==id){s.aggregate.material=index;}}let events=s.list.poll(&stock,generated::PROVISIONS);
   for i in events{if let Some(e)=s.list.slots[i]{s.notices.push(e.recipe);}}
  }else{for e in s.list.slots.iter_mut().flatten(){e.ready=None}}
 });}
}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiInit(_ctx:*mut c_void,_alloc:*mut c_void,_free:*mut c_void){ui::init();}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiFreeRender(){ui::render();}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiRender(){ui::menu();}
