//! All resources change together at NewFramePre, outside any UI/font stack.
use alloc::{vec,vec::Vec,string::String,format,ffi::CString};
use core::{cell::UnsafeCell,sync::atomic::{AtomicBool,Ordering}};
use crate::language_pack::{Pack,Store,expected};
struct Shared{lock:AtomicBool,data:UnsafeCell<Store>}
unsafe impl Sync for Shared{}
impl Shared{fn with<R>(&self,f:impl FnOnce(&mut Store)->R)->R{while self.lock.compare_exchange_weak(false,true,Ordering::Acquire,Ordering::Relaxed).is_err(){core::hint::spin_loop()}let result=f(unsafe{&mut*self.data.get()});self.lock.store(false,Ordering::Release);result}}
static DATA:Shared=Shared{lock:AtomicBool::new(false),data:UnsafeCell::new(Store{active:None})};
pub fn reset(){let old=DATA.with(|s|s.active.take());drop(old);}
pub fn current()->Option<usize>{DATA.with(|s|s.active.as_ref().map(|p|p.language))}
pub fn text(description:bool,table:u8,index:u16,lang:usize)->String{DATA.with(|s|s.text(description,table,index,lang))}
pub fn path(lang:usize,kind:usize)->Option<CString>{let code=crate::locales::CODES.get(lang)?;let (prefix,extension)=match kind{0=>("names","bin"),1=>("text","bin"),2=>("font","otf"),_=>return None};CString::new(if kind==2{String::from("app:/nativeNX/plugins/wishlist/languages/font.shared.otf")}else{format!("app:/nativeNX/plugins/wishlist/languages/{}.{}.{}",prefix,code,extension)}).ok()}
fn read(lang:usize,kind:usize)->Result<Vec<u8>,()>{unsafe{
 use forge::sys::imgui as sys;let path=path(lang,kind).ok_or(())?;let (size,_)=expected(lang,kind).ok_or(())?;
 let file=sys::igImFileOpen(path.as_ptr(),c"rb".as_ptr());if file.is_null(){return Err(())}
 if sys::igImFileGetSize(file)!=size as u64{sys::igImFileClose(file);return Err(())}
 let mut bytes=vec![0u8;size];let count=sys::igImFileRead(bytes.as_mut_ptr().cast(),1,size as u64,file);sys::igImFileClose(file);
 if count==size as u64{Ok(bytes)}else{Err(())}
}}
pub fn prepare(lang:usize)->Result<Pack,()>{Pack::prepare(lang,|kind|read(lang,kind))}
pub fn commit(pack:Pack){let lang=pack.language;let bytes=pack.bytes();let old=DATA.with(|s|s.commit(pack));drop(old);forge::log::prelude::info!("[wishlist] language {} text loaded ({} bytes)",lang,bytes);}

pub fn font_valid()->bool{read(0,2).is_ok_and(|font|crate::language_pack::matches(0,2,&font)&&font.get(..4)==Some(b"OTTO"))}
