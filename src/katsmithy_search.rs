//! Content searches share a bounded candidate budget per frame, not per caller.
pub const POSITIONS_PER_FRAME:usize=16*1024; // at most 64 KiB of starting offsets
pub const READY_FRAMES:u32=6;
pub struct Budget{frame:u64,left:usize,limit:usize}
impl Budget{
 pub const fn new(limit:usize)->Self{Self{frame:u64::MAX,left:0,limit}}
 fn take(&mut self,frame:u64)->usize{if self.frame!=frame{self.frame=frame;self.left=self.limit}let n=self.left;self.left=0;n}
}
#[derive(Debug,PartialEq,Eq)]
pub enum Result{Pending,Found(usize),Miss(u32)}
pub struct Search{last:u64,stable:u32,next:usize,retry:u64,misses:u32}
impl Search{
 pub const fn new()->Self{Self{last:u64::MAX,stable:0,next:0,retry:0,misses:0}}
 pub fn step(&mut self,frame:u64,ready:bool,len:usize,budget:&mut Budget,mut candidate:impl FnMut(usize)->Option<usize>)->Result{
  if self.last==frame{return Result::Pending}
  if self.last.wrapping_add(1)!=frame{self.stable=0}
  self.last=frame;
  if !ready{self.stable=0;return Result::Pending}
  self.stable=self.stable.saturating_add(1);
  if self.stable<READY_FRAMES||frame<self.retry{return Result::Pending}
  let count=budget.take(frame);if count==0{return Result::Pending}
  let end=self.next.saturating_add(count*4).min(len);
  for at in (self.next..end).step_by(4){if let Some(found)=candidate(at){self.next=0;self.misses=0;return Result::Found(found)}}
  self.next=end;
  if end<len{return Result::Pending}
  self.next=0;self.misses=self.misses.saturating_add(1);
  self.retry=frame.saturating_add(120u64<<self.misses.saturating_sub(1).min(7));
  Result::Miss(self.misses)
 }
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn search_resumes_without_gaps_and_repeated_call_cannot_multiply_budget(){
  let mut s=Search::new();let mut budget=Budget::new(2);let mut checked=alloc::vec::Vec::new();
  for f in 0..READY_FRAMES as u64-1{assert_eq!(s.step(f,true,28,&mut budget,|_|panic!("not ready")),Result::Pending)}
  let f=READY_FRAMES as u64-1;
  assert_eq!(s.step(f,true,28,&mut budget,|a|{checked.push(a);None}),Result::Pending);
  assert_eq!(checked,alloc::vec![0,4]);
  assert_eq!(s.step(f,true,28,&mut budget,|_|panic!("duplicate frame")),Result::Pending);
  for f in f+1..f+4{s.step(f,true,28,&mut budget,|a|{checked.push(a);(a==24).then_some(a|1)});}
  assert_eq!(checked,alloc::vec![0,4,8,12,16,20,24]);
 }
 #[test]fn transition_frames_do_not_search_or_count_as_failure(){
  let mut s=Search::new();let mut b=Budget::new(1);
  for f in 0..100{s.step(f,false,32,&mut b,|_|panic!("transition"));}
  for f in 100..105{s.step(f,true,32,&mut b,|_|panic!("settling"));}
  assert_eq!(s.step(105,true,32,&mut b,|a|Some(a)),Result::Found(0));
 }
 #[test]fn only_completed_sweeps_back_off_and_a_later_record_is_discovered(){
  let mut s=Search::new();let mut b=Budget::new(2);
  for f in 0..6{s.step(f,true,12,&mut b,|_|None);}
  assert_eq!(s.step(6,true,12,&mut b,|_|None),Result::Miss(1));
  for f in 7..126{s.step(f,true,12,&mut b,|_|panic!("cooldown"));}
  assert_eq!(s.step(126,true,12,&mut b,|a|(a==4).then_some(a)),Result::Found(4));
 }
 #[test]fn real_tree_validation_finds_records_across_chunk_boundaries_in_both_layouts(){
  use crate::katsmithy_model as model;
  for (layout,l) in model::TREE_LAYOUTS.iter().enumerate(){
   let at=0x10000-4;let mut memory=alloc::vec![0u8;at+l.end+64];
   for (offset,value) in [(l.columns,5u32),(l.rows,1)]{memory[at+offset..at+offset+4].copy_from_slice(&value.to_le_bytes());}
   for (offset,value) in [(l.row_height,1u16),(l.row_line,1)]{memory[at+offset..at+offset+2].copy_from_slice(&value.to_le_bytes());}
   let mut search=Search::new();let mut budget=Budget::new(1024);let mut found=None;
   for frame in 0..100{
    let mut count=0;
    let result=search.step(frame,true,memory.len(),&mut budget,|offset|{
     count+=1;
     (0..model::TREE_LAYOUTS.len() as u8).find(|&i|model::tree_at(&memory,offset,i).is_some()).map(|i|offset|i as usize)
    });
    assert!(count<=1024);
    if let Result::Found(value)=result{found=Some(value);break}
   }
   assert_eq!(found,Some(at|layout));
   assert!(model::tree_live(&memory,found.unwrap()).is_some());
  }
 }
 #[test]fn two_searches_share_the_frame_budget(){
  let mut a=Search::new();let mut b=Search::new();let mut budget=Budget::new(2);
  for f in 0..5{a.step(f,true,32,&mut budget,|_|None);b.step(f,true,32,&mut budget,|_|None);}
  a.step(5,true,32,&mut budget,|_|None);
  assert_eq!(b.step(5,true,32,&mut budget,|_|panic!("shared budget exhausted")),Result::Pending);
 }
}
