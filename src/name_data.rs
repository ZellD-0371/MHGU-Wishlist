//! Names live in the resource heap instead of the shared Forge plugin region.
#[cfg(target_arch="arm")]
static mut DATA:&[u8]=&[];
#[cfg(not(target_arch="arm"))]
static DATA:&[u8]=include_bytes!("../assets/ui-names.bin");

#[cfg(target_arch="arm")]
pub fn init(){unsafe{
 use alloc::{boxed::Box,vec};
 use forge::sys::imgui as sys;
 let current=DATA;if !current.is_empty(){return}
 let file=sys::igImFileOpen(c"app:/nativeNX/plugins/wishlist/ui-names.bin".as_ptr(),c"rb".as_ptr());
 if file.is_null(){forge::log::prelude::error!("[wishlist] name file missing");return}
 let size=sys::igImFileGetSize(file);
 if !(12..=16*1024*1024).contains(&size){sys::igImFileClose(file);forge::log::prelude::error!("[wishlist] invalid name file size");return}
 let mut data=vec![0u8;size as usize];
 let read=sys::igImFileRead(data.as_mut_ptr().cast(),1,size,file);sys::igImFileClose(file);
 if read!=size||!crate::text_blob::valid(&data){forge::log::prelude::error!("[wishlist] invalid name file");return}
 DATA=Box::leak(data.into_boxed_slice());
 forge::log::prelude::info!("[wishlist] names loaded ({} bytes)",size);
}}
#[cfg(not(target_arch="arm"))]
pub fn init(){}
pub fn get(table:u8,index:usize,lang:usize)->&'static str{
 if index>u16::MAX as usize{return ""}
 #[cfg(target_arch="arm")]
 let data=unsafe{DATA};
 #[cfg(not(target_arch="arm"))]
 let data=DATA;
 crate::text_blob::get(data,table,index as u16,lang)
}
pub fn material_description(id:u16,lang:usize)->&'static str{get(31,id as usize,lang)}
