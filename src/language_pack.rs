//! Validated single-language resources; staging owns its allocations until commit.
extern crate alloc;
use alloc::{vec,vec::Vec,string::String,boxed::Box};
mod manifest{include!("language_assets.rs");}
/// Unfinalized CRC state so large files can use a bounded scratch buffer.
pub fn crc_step(mut value:u32,data:&[u8])->u32{
 const TABLE:[u32;16]=[0x00000000,0x1db71064,0x3b6e20c8,0x26d930ac,0x76dc4190,0x6b6b51f4,0x4db26158,0x5005713c,0xedb88320,0xf00f9344,0xd6d6a3e8,0xcb61b38c,0x9b64c2b0,0x86d3d2d4,0xa00ae278,0xbdbdf21c];
 for byte in data{value^=*byte as u32;value=(value>>4)^TABLE[(value&15) as usize];value=(value>>4)^TABLE[(value&15) as usize];}value
}
pub fn crc32(data:&[u8])->u32{!crc_step(!0,data)}
pub fn expected(lang:usize,kind:usize)->Option<(usize,u32)>{manifest::FILES.get(lang)?.get(kind).copied()}
pub fn matches(lang:usize,kind:usize,data:&[u8])->bool{expected(lang,kind).is_some_and(|(bytes,crc)|bytes==data.len()&&crc32(data)==crc)}
/// Read exactly the requested range; implementations own and close their file.
pub trait Source:Send{fn len(&self)->u64;fn read(&mut self,offset:u64,bytes:&mut[u8])->Result<(),()>;}
pub const CACHE_ENTRIES:usize=24;
pub const CACHE_BYTES:usize=24*1024;
const MAX_DESCRIPTION:usize=4096;
struct Cached{key:u32,text:String}
pub struct Pack{pub language:usize,names:Vec<u8>,index:Vec<u8>,source:Box<dyn Source>,cache:Vec<Cached>,pub failed_reads:usize}
fn word(bytes:&[u8],at:usize)->Option<u32>{Some(u32::from_le_bytes(bytes.get(at..at+4)?.try_into().ok()?))}
impl Pack{
 pub fn prepare(language:usize,names:Vec<u8>,mut source:Box<dyn Source>)->Result<Self,()>{
  if !matches(language,0,&names)||crate::text_blob::single_language(&names)!=Some(language){return Err(())}
  let(size,_)=expected(language,1).ok_or(())?;let(prefix,crc)=*manifest::DESCRIPTION_INDEX.get(language).ok_or(())?;
  if source.len()!=size as u64||prefix<16||prefix>size{return Err(())}
  let mut index=vec![0;prefix];source.read(0,&mut index)?;
  if crc32(&index)!=crc||index.get(..4)!=Some(b"WLD1")||word(&index,4)!=Some(1)||word(&index,8)!=Some(language as u32){return Err(())}
  let count=word(&index,12).ok_or(())? as usize;if count.checked_mul(8).and_then(|n|n.checked_add(16))!=Some(prefix){return Err(())}
  let mut previous=None;for row in index[16..].chunks_exact(8){let key=word(row,0).ok_or(())?;let at=word(row,4).ok_or(())? as usize;
   if previous.is_some_and(|p|p>=key)||at<prefix||at.checked_add(8).is_none_or(|end|end>size){return Err(())}previous=Some(key);
  }
  Ok(Self{language,names,index,source,cache:Vec::new(),failed_reads:0})
 }
 fn offset(&self,key:u32)->Option<u32>{let mut low=0;let mut high=(self.index.len()-16)/8;while low<high{let mid=(low+high)/2;let row=16+mid*8;match word(&self.index,row)?.cmp(&key){core::cmp::Ordering::Less=>low=mid+1,core::cmp::Ordering::Greater=>high=mid,core::cmp::Ordering::Equal=>return word(&self.index,row+4)}}None}
 fn description(&mut self,key:u32)->Result<String,()>{
  let Some(offset)=self.offset(key)else{return Ok(String::new())};let mut header=[0;8];self.source.read(offset as u64,&mut header)?;
  let length=word(&header,0).ok_or(())? as usize;let crc=word(&header,4).ok_or(())?;
  if length==0||length>MAX_DESCRIPTION||(offset as u64+8+length as u64)>self.source.len(){return Err(())}
  let mut raw=vec![0;length];self.source.read(offset as u64+8,&mut raw)?;
  if crc32(&raw)!=crc{return Err(())}String::from_utf8(raw).map_err(|_|())
 }
 pub fn text(&mut self,description:bool,table:u8,index:u16,lang:usize)->String{
  if lang!=self.language{return String::new()}
  if !description&&table!=31{return crate::text_blob::get(&self.names,table,index,lang).into()}
  let key=(table as u32)<<16|index as u32;
  if let Some(at)=self.cache.iter().position(|entry|entry.key==key){let entry=self.cache.remove(at);let text=entry.text.clone();self.cache.push(entry);return text}
  let text=match self.description(key){Ok(text)=>text,Err(())=>{self.failed_reads=self.failed_reads.saturating_add(1);String::new()}};
  while self.cache.len()>=CACHE_ENTRIES||self.cache.iter().map(|entry|entry.text.len()).sum::<usize>()+text.len()>CACHE_BYTES{self.cache.remove(0);}
  self.cache.push(Cached{key,text:text.clone()});text
 }
 pub fn bytes(&self)->usize{self.names.len()+self.index.len()+self.cache.iter().map(|entry|entry.text.capacity()).sum::<usize>()+self.cache.capacity()*core::mem::size_of::<Cached>()}
}
/// Commit is called only after both data and font preparation succeed.
#[derive(Default)]pub struct Store{pub active:Option<Pack>}
impl Store{
 pub fn commit(&mut self,next:Pack)->Option<Pack>{self.active.replace(next)}
 pub fn text(&mut self,description:bool,table:u8,index:u16,lang:usize)->String{self.active.as_mut().map_or_else(String::new,|p|p.text(description,table,index,lang))}
}
