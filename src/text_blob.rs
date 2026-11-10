//! Checked, sorted on-disk text index. No native addresses are stored in the file.
fn word(data:&[u8],at:usize)->Option<u32>{Some(u32::from_le_bytes(data.get(at..at+4)?.try_into().ok()?))}
pub fn valid(data:&[u8])->bool{
 if data.get(..4)!=Some(b"WLTX")||word(data,4)!=Some(1){return false}
 let Some(count)=word(data,8) else{return false};let Some(end)=(count as usize).checked_mul(40).and_then(|n|n.checked_add(12)) else{return false};
 if end>data.len(){return false}let mut previous=0;
 for i in 0..count as usize{let row=12+i*40;let key=word(data,row).unwrap();if key<=previous{return false}previous=key;
  for lang in 0..9{let at=word(data,row+4+lang*4).unwrap() as usize;if at<end||at>=data.len(){return false}}
 }
 core::str::from_utf8(&data[end..]).is_ok()&&data.last()==Some(&0)
}
pub fn get(data:&[u8],kind:u8,id:u16,lang:usize)->&str{
 if lang>=9{return ""}let key=(kind as u32)<<16|id as u32;let mut low=0;let mut high=word(data,8).unwrap_or(0) as usize;
 while low<high{let mid=(low+high)/2;let row=12+mid*40;match word(data,row){Some(n) if n<key=>low=mid+1,Some(n) if n>key=>high=mid,Some(_)=>{
  let Some(at)=word(data,row+4+lang*4) else{return ""};let Some(rest)=data.get(at as usize..) else{return ""};let end=rest.iter().position(|n|*n==0).unwrap_or(0);return core::str::from_utf8(&rest[..end]).unwrap_or("")
 },None=>return ""}}
 ""
}
