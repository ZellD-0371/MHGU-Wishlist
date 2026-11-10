#![no_std]
extern crate alloc;
mod name_data;
mod hook_guard;
mod model;mod generated;mod locales;mod native;mod controller;mod storage;mod ui;mod smithy;mod equipment;mod navigation;mod sound;mod icons;mod interaction;mod session;mod profiles;mod text_blob;mod text_data;mod presentation;mod game_menu;mod sound_cues;mod notifications;mod badge_model;mod native_badges;mod completed;
use core::{cell::UnsafeCell,sync::atomic::{AtomicBool,Ordering},ffi::c_void};
use model::{List,Recipe};
use forge::log::prelude::{info,error};
#[derive(Clone,Copy)]
struct Target{owner:u32,recipe:Recipe,cost:u32}
struct State{
 settings_loaded:bool,profile:Option<usize>,game_ready:bool,base:u32,installed:bool,input_ready:bool,list:List,loaded:bool,mode:u8,selected:usize,tab:usize,aggregate:interaction::AggregateFocus,preview_clock:f32,
 target:Option<Target>,frozen:Option<Recipe>,stock:[u32;4096],stock_ready:bool,
 notices:notifications::Queue,notice_demo:Option<notifications::Demo>,notice_test_case:usize,cooldown:u32,timer:f32,events:u32,status:u8,preview:bool,preview_panel:usize,detail_page:usize,frozen_cost:u32,focus:interaction::Focus,
}
impl State{
 fn new()->Self{Self{settings_loaded:false,profile:None,game_ready:false,base:0,installed:false,input_ready:false,list:List::default(),loaded:false,mode:0,selected:0,tab:0,aggregate:interaction::AggregateFocus::default(),preview_clock:0.,target:None,frozen:None,stock:[0;4096],stock_ready:false,notices:notifications::Queue::default(),notice_demo:None,notice_test_case:2,cooldown:180,timer:0.,events:0,status:0,preview:false,preview_panel:0,detail_page:0,frozen_cost:0,focus:interaction::Focus::default()}}
 fn capture(&mut self,owner:u32,recipe:Recipe,cost:u32){if self.mode==0&&recipe.valid(){self.target=Some(Target{owner,recipe,cost});}}
 fn active(&self)->Option<Recipe>{self.target.filter(|t|native::visible(t.owner,self.base)).map(|t|t.recipe).or_else(||native_badges::selected_registered(&self.list,self.base).map(|(_,r)|r))}
 fn at_smithy(&self)->bool{self.active().is_some()||native_badges::smithy_visible(self.base)}
 fn open(&mut self,register:bool){
  if !self.game_ready||self.profile.is_none()||!self.loaded||!self.input_ready{return}
  self.focus=interaction::Focus::default();self.preview=false;self.notice_demo=None;self.tab=0;self.aggregate=interaction::AggregateFocus::default();self.preview_clock=0.;self.status=0;self.detail_page=0;sound::request_open(self.list.open_sound);
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
 if !native::install(base){error!("[wishlist] MHGU global 1.4.0 signatures rejected; disabled");return}
 if !native_badges::install(base){error!("[wishlist] native icon signatures rejected; badges disabled");}
 if !completed::install(base){error!("[wishlist] production completion signatures rejected; auto removal disabled");}
 sound::install(base);
 if !game_menu::install(base){error!("[wishlist] native + menu check failed; controller list shortcut disabled");}
 if !session::install(base){error!("[wishlist] character session signatures rejected");return}
 let input_ready=controller::install(base);
 STATE.with(|s|{s.installed=true;s.input_ready=input_ready;});
 if !input_ready{error!("[wishlist] input capture failed; opening disabled");}
 info!("[wishlist] v1.0.0; steel workshop UI; 9 languages; 48 slots; R3 register / L3+R3 list");
}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(dt:f32){
 let (base,ready)=STATE.with(|s|(s.base,s.installed));if !ready{return}
 sound::update(base);
 if !STATE.with(|s|s.settings_loaded){let mut bytes=[0;storage::MAX_BYTES];if let Ok(text)=storage::WISHLIST.read(&mut bytes){if let Ok(list)=List::decode(text){STATE.with(|s|{s.list.copy_preferences(&list);s.settings_loaded=true;});}}}
 let (slot,playing)=session::update();
 STATE.with(|s|{if playing&&!s.game_ready{info!("[wishlist] character session ready (slot {})",slot.unwrap_or(0)+1);}
 s.game_ready=playing;if s.profile!=slot{completed::clear();native_badges::clear();s.profile=slot;s.loaded=false;s.mode=0;s.target=None;s.stock_ready=false;s.notices.clear();s.notice_demo=None;s.events=0;s.cooldown=180;}if !playing{s.mode=0;s.preview=false;s.events=0;s.notices.clear();s.notice_demo=None;}});
 let Some(slot)=slot else{return};
 if !STATE.with(|s|s.loaded){match storage::load_character(slot){
  Ok(list)=>STATE.with(|s|{s.list=list;s.loaded=true}),
  Err(_)=>return,
 }}
 if !playing{return}
 completed::update(slot);
 let poll=STATE.with(|s|{
  // Notification time advances only while its panel is actually rendered.
  if s.cooldown>0{s.cooldown-=1;return None}
  s.timer+=dt.clamp(0.,0.1);if s.timer<0.5{return None}s.timer=0.;Some(s.base)
 });
 if let Some(base)=poll{let mut stock=[0;4096];let valid=native::inventory(base,&mut stock);STATE.with(|s|{
  let material=if s.mode==2&&s.tab==1{s.list.ordered_totals(&s.stock,s.stock_ready).get(s.aggregate.material).map(|m|m.id)}else{None};
  s.stock_ready=valid;if valid{s.stock=stock;if let Some(id)=material{if let Some(index)=s.list.ordered_totals(&s.stock,true).iter().position(|m|m.id==id){s.aggregate.material=index;}}let events=s.list.poll(&stock,generated::PROVISIONS);
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
