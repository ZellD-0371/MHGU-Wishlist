//! Same size, OTTO signature and CRC checks as before, with bounded heap scratch.
extern crate alloc;
use alloc::vec;
pub const CHUNK:usize=64*1024;
pub fn validate(actual:u64,expected:(usize,u32),mut read:impl FnMut(&mut[u8])->Result<usize,()>)->bool{
 let(size,wanted)=expected;if size<4||actual!=size as u64{return false}
 let mut buffer=vec![0;CHUNK.min(size)];let mut remaining=size;let mut crc=!0;let mut first=true;
 while remaining>0{
  let n=remaining.min(buffer.len());let part=&mut buffer[..n];if read(part)!=Ok(n){return false}
  if first{if part.get(..4)!=Some(b"OTTO"){return false}first=false;}
  crc=crate::language_pack::crc_step(crc,part);remaining-=n;
 }
 !crc==wanted
}
