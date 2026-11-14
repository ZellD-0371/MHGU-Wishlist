//! Production streaming validator, plus the actual font_valid file-open/close wiring.
extern crate alloc;
#[path="../src/diagnostics.rs"]mod diagnostics;
extern crate self as forge;
#[macro_export]macro_rules! error{($($t:tt)*)=>{{}}}
#[macro_export]macro_rules! info{($($t:tt)*)=>{{}}}
pub mod log{pub mod prelude{pub use crate::{error,info};}}
#[path="../src/text_blob.rs"]mod text_blob;
#[path="../src/language_pack.rs"]mod language_pack;
#[path="../src/font_validation.rs"]mod font_validation;
#[path="../src/language.rs"]mod language;
mod locales{pub const CODES:[&str;9]=["en","zh-Hans","zh-Hant","ja","ko","de","fr","it","es"];}
// Unused by the font path; demand-description SDK behavior has its own harness.
mod description_file{pub struct File;impl File{pub fn open(_: &core::ffi::CStr)->Result<Self,()>{Err(())}}impl crate::language_pack::Source for File{fn len(&self)->u64{0}fn read(&mut self,_:u64,_:&mut[u8])->Result<(),()>{Err(())}}}
struct Mock{data:Vec<u8>,at:usize,missing:bool,size:u64,short:bool,open:usize,close:usize,reads:usize,max_read:usize}
static MOCK:std::sync::Mutex<Option<Mock>>=std::sync::Mutex::new(None);
pub mod sys{pub mod imgui{
 use super::super::*;
 pub unsafe fn igImFileOpen(_: *const core::ffi::c_char,_:*const core::ffi::c_char)->*mut core::ffi::c_void{let mut state=MOCK.lock().unwrap();let m=state.as_mut().unwrap();if m.missing{return core::ptr::null_mut()}m.open+=1;1 as *mut _}
 pub unsafe fn igImFileGetSize(_: *mut core::ffi::c_void)->u64{MOCK.lock().unwrap().as_ref().unwrap().size}
 pub unsafe fn igImFileRead(out:*mut core::ffi::c_void,size:u64,count:u64,_:*mut core::ffi::c_void)->u64{assert_eq!(size,1);let mut state=MOCK.lock().unwrap();let m=state.as_mut().unwrap();let requested=count as usize;let n=if m.short{requested/2}else{requested.min(m.data.len().saturating_sub(m.at))};unsafe{core::ptr::copy_nonoverlapping(m.data.as_ptr().add(m.at),out.cast(),n)}m.at+=n;m.reads+=1;m.max_read=m.max_read.max(requested);n as u64}
 pub unsafe fn igImFileClose(_: *mut core::ffi::c_void)->bool{MOCK.lock().unwrap().as_mut().unwrap().close+=1;true}
}}
fn validate(data:&[u8],size:u64,wanted:(usize,u32))->(bool,usize,usize){let mut at=0;let mut calls=0;let mut largest=0;let valid=font_validation::validate(size,wanted,|part|{calls+=1;largest=largest.max(part.len());let n=part.len().min(data.len().saturating_sub(at));part[..n].copy_from_slice(&data[at..at+n]);at+=n;Ok(n)});(valid,calls,largest)}
#[test]fn chunked_crc_matches_single_read_across_boundaries_and_real_font(){
 for length in [4,7,65535,65536,65537,131072]{let mut data:Vec<u8>=(0..length).map(|i|(i%251) as u8).collect();data[..4].copy_from_slice(b"OTTO");let expected=(length,language_pack::crc32(&data));let(valid,calls,largest)=validate(&data,length as u64,expected);assert!(valid);assert_eq!(calls,length.div_ceil(font_validation::CHUNK));assert!(largest<=font_validation::CHUNK);}
 let font=std::fs::read("assets/languages/font.shared.otf").unwrap();let expected=language_pack::expected(0,2).unwrap();let(valid,calls,largest)=validate(&font,font.len() as u64,expected);assert!(valid);assert_eq!(calls,font.len().div_ceil(font_validation::CHUNK));assert_eq!(largest,65536);
}
#[test]fn malformed_size_signature_crc_truncation_and_read_errors_are_rejected(){
 let good=std::fs::read("assets/languages/font.shared.otf").unwrap();let expected=language_pack::expected(0,2).unwrap();
 for size in [0,3,good.len() as u64-1,good.len() as u64+1,u64::MAX]{let(valid,calls,_)=validate(&good,size,expected);assert!(!valid);assert_eq!(calls,0);}
 for at in [0,font_validation::CHUNK-1,font_validation::CHUNK,good.len()-1]{let mut damaged=good.clone();damaged[at]^=1;assert!(!validate(&damaged,damaged.len() as u64,expected).0);}
 let mut bad=good.clone();bad[0]=0;assert!(!validate(&bad,bad.len() as u64,(bad.len(),language_pack::crc32(&bad))).0,"signature must be checked even if checksum matches");
 assert!(!validate(&good[..good.len()-1],good.len() as u64,expected).0);
 assert!(!font_validation::validate(good.len() as u64,expected,|_|Err(())));
 assert!(!validate(&good,good.len() as u64,(good.len(),expected.1^1)).0);
}
#[test]fn actual_font_valid_closes_files_on_success_and_all_early_failures(){
 let good=std::fs::read("assets/languages/font.shared.otf").unwrap();
 for fault in 0..5{let mut data=good.clone();if fault==3{data[0]=0;}
  let size=good.len() as u64+if fault==2{1}else{0};*MOCK.lock().unwrap()=Some(Mock{data,at:0,missing:fault==1,size,short:fault==4,open:0,close:0,reads:0,max_read:0});
  assert_eq!(language::font_valid(),fault==0);let state=MOCK.lock().unwrap();let m=state.as_ref().unwrap();assert_eq!(m.open,m.close);assert!(m.max_read<=65536);if fault==0{assert_eq!(m.reads,good.len().div_ceil(65536));assert_eq!(m.at,good.len());}if fault==1||fault==2{assert_eq!(m.reads,0);}
 }
}
