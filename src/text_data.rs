use alloc::{boxed::Box,vec};
use forge::sys::imgui as sys;
static mut TEXT:&[u8]=&[];
pub fn init(){unsafe{
 let file=sys::igImFileOpen(c"app:/nativeNX/plugins/wishlist/ui-text.bin".as_ptr(),c"rb".as_ptr());
 if file.is_null(){forge::log::prelude::error!("[wishlist] description file missing");return}
 let size=sys::igImFileGetSize(file);if !(12..=16*1024*1024).contains(&size){sys::igImFileClose(file);return}
 let mut data=vec![0u8;size as usize];let read=sys::igImFileRead(data.as_mut_ptr().cast(),1,size,file);sys::igImFileClose(file);
 if read!=size||!crate::text_blob::valid(&data){forge::log::prelude::error!("[wishlist] invalid description file");return}
 TEXT=Box::leak(data.into_boxed_slice());forge::log::prelude::info!("[wishlist] descriptions loaded ({} bytes)",size);
}}
pub fn description(kind:u8,id:u16,lang:usize)->&'static str{crate::text_blob::get(unsafe{TEXT},kind,id,lang)}
