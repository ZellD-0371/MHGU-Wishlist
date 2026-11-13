//! Read-only SDK file ranges. No ImGui file seek or Recomp extension is needed.
use core::ffi::{CStr,c_char,c_void};
#[repr(C)]#[derive(Clone,Copy)]struct Handle(u32);
const _:()=assert!(core::mem::size_of::<Handle>()==4);
unsafe extern "C"{
 #[link_name="_ZN2nn2fs8OpenFileEPNS0_10FileHandleEPKci"]fn open(handle:*mut Handle,path:*const c_char,mode:i32)->u32;
 #[link_name="_ZN2nn2fs11GetFileSizeEPxNS0_10FileHandleE"]fn size(bytes:*mut i64,handle:Handle)->u32;
 #[link_name="_ZN2nn2fs8ReadFileENS0_10FileHandleExPvj"]fn read(handle:Handle,offset:i64,bytes:*mut c_void,length:usize)->u32;
 #[link_name="_ZN2nn2fs9CloseFileENS0_10FileHandleE"]fn close(handle:Handle);
}
pub struct File{handle:Handle,bytes:u64}
impl File{
 pub fn open(path:&CStr)->Result<Self,()>{unsafe{
  let mut handle=Handle(0);if open(&mut handle,path.as_ptr(),1)!=0{return Err(())}
  let mut bytes=0;let result=size(&mut bytes,handle);if result!=0||bytes<0{close(handle);return Err(())}
  Ok(Self{handle,bytes:bytes as u64})
 }}
}
impl crate::language_pack::Source for File{
 fn len(&self)->u64{self.bytes}
 fn read(&mut self,offset:u64,bytes:&mut[u8])->Result<(),()>{
  if offset.checked_add(bytes.len() as u64).is_none_or(|end|end>self.bytes){return Err(())}
  if unsafe{read(self.handle,offset as i64,bytes.as_mut_ptr().cast(),bytes.len())}==0{Ok(())}else{Err(())}
 }
}
impl Drop for File{fn drop(&mut self){unsafe{close(self.handle)}}}
