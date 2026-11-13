//! Validated single-language resources; staging owns its allocations until commit.
extern crate alloc;
use alloc::{vec::Vec,string::String};
mod manifest{include!("language_assets.rs");}
pub fn crc32(data:&[u8])->u32{
 const TABLE:[u32;16]=[0x00000000,0x1db71064,0x3b6e20c8,0x26d930ac,0x76dc4190,0x6b6b51f4,0x4db26158,0x5005713c,0xedb88320,0xf00f9344,0xd6d6a3e8,0xcb61b38c,0x9b64c2b0,0x86d3d2d4,0xa00ae278,0xbdbdf21c];
 let mut value=!0u32;for byte in data{value^=*byte as u32;value=(value>>4)^TABLE[(value&15) as usize];value=(value>>4)^TABLE[(value&15) as usize];}!value
}
pub fn expected(lang:usize,kind:usize)->Option<(usize,u32)>{manifest::FILES.get(lang)?.get(kind).copied()}
pub fn matches(lang:usize,kind:usize,data:&[u8])->bool{expected(lang,kind).is_some_and(|(bytes,crc)|bytes==data.len()&&crc32(data)==crc)}
pub struct Pack{pub language:usize,names:Vec<u8>,descriptions:Vec<u8>}
impl Pack{
 pub fn prepare(language:usize,mut read:impl FnMut(usize)->Result<Vec<u8>,()>)->Result<Self,()>{
  let names=read(0)?;if !matches(language,0,&names)||crate::text_blob::single_language(&names)!=Some(language){return Err(())}
  let descriptions=read(1)?;if !matches(language,1,&descriptions)||crate::text_blob::single_language(&descriptions)!=Some(language){return Err(())}
  Ok(Self{language,names,descriptions})
 }
 pub fn text(&self,description:bool,table:u8,index:u16,lang:usize)->String{crate::text_blob::get(if description{&self.descriptions}else{&self.names},table,index,lang).into()}
 pub fn bytes(&self)->usize{self.names.len()+self.descriptions.len()}
}
/// Commit is called only after both data and font preparation succeed.
#[derive(Default)]pub struct Store{pub active:Option<Pack>}
impl Store{
 pub fn commit(&mut self,next:Pack)->Option<Pack>{self.active.replace(next)}
 pub fn text(&self,description:bool,table:u8,index:u16,lang:usize)->String{self.active.as_ref().map_or_else(String::new,|p|p.text(description,table,index,lang))}
}
