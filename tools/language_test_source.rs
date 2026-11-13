//! Host adapter with the same exact-range contract as the SDK reader.
use std::{io::{Read,Seek,SeekFrom},fs::File};
pub struct Disk{file:File,len:u64}
impl Disk{pub fn open(language:usize)->Result<Self,()>{let code=["en","zh-Hans","zh-Hant","ja","ko","de","fr","it","es"].get(language).ok_or(())?;let file=File::open(format!("assets/languages/text.{code}.bin")).map_err(|_|())?;let len=file.metadata().map_err(|_|())?.len();Ok(Self{file,len})}}
impl crate::language_pack::Source for Disk{
 fn len(&self)->u64{self.len}
 fn read(&mut self,offset:u64,bytes:&mut[u8])->Result<(),()>{self.file.seek(SeekFrom::Start(offset)).map_err(|_|())?;self.file.read_exact(bytes).map_err(|_|())}
}
pub fn prepare(language:usize)->Result<crate::language_pack::Pack,()>{let code=["en","zh-Hans","zh-Hant","ja","ko","de","fr","it","es"].get(language).ok_or(())?;let names=std::fs::read(format!("assets/languages/names.{code}.bin")).map_err(|_|())?;crate::language_pack::Pack::prepare(language,names,Box::new(Disk::open(language)?))}
pub struct Memory(pub Vec<u8>);
impl crate::language_pack::Source for Memory{
 fn len(&self)->u64{self.0.len() as u64}
 fn read(&mut self,offset:u64,bytes:&mut[u8])->Result<(),()>{bytes.copy_from_slice(self.0.get(offset as usize..offset as usize+bytes.len()).ok_or(())?);Ok(())}
}
