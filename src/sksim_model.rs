//! Skill-Simulator guest link ("MHGU-SKSIM-EXT01" version 1): block layout,
//! what to register, and the confirmation dialog's input. No platform calls.
//!
//! Skill-Simulator publishes the cursor's set (or one armor piece), the raw pad
//! buttons it withholds from the game, and a slot on its hint line. We detect
//! our own shortcut from that raw word, draw our hint in the slot, and show our
//! own dialog while asking it to ignore its input (guest_modal).
extern crate alloc;
use alloc::vec::Vec;
use crate::model::{List,Recipe};

pub const MAGIC:&[u8;16]=b"MHGU-SKSIM-EXT01";
pub const VERSION:u32=1;
pub const SIZE:usize=128;
/// Guest fields written by us.
pub const GUEST_FRAME:usize=92;
pub const HINT_WIDTH:usize=96;
pub const GUEST_MODAL:usize=100;
/// Our frames without a new Skill-Simulator frame before its window counts as closed.
const STALE:u32=10;
pub const OWNED:u8=1;
pub const REWARD:u8=2;
/// Raw HID bits used by the dialog: A, B, D-pad and left stick directions.
pub const A:u32=1;
pub const B:u32=2;
const LEFT:u32=(1<<12)|(1<<16);
const UP:u32=(1<<13)|(1<<17);
const RIGHT:u32=(1<<14)|(1<<18);
const DOWN:u32=(1<<15)|(1<<19);

#[derive(Clone,Copy,Default,Debug,PartialEq,Eq)]
pub struct Piece{pub series:u16,pub part:u8,pub flags:u8}

#[derive(Clone,Copy,Default,Debug,PartialEq)]
pub struct View{
 pub open:bool,pub frame:u32,pub screen:u32,pub overlay:bool,pub kind:u32,pub part:u32,
 pub pieces:[Piece;5],pub raw:u32,pub serial:u32,pub hint:[f32;4],pub scale:f32,
}
fn word(b:&[u8],at:usize)->u32{u32::from_le_bytes([b[at],b[at+1],b[at+2],b[at+3]])}
fn float(b:&[u8],at:usize)->f32{f32::from_bits(word(b,at))}
/// The block's contents, or None when the magic or version does not match.
pub fn parse(b:&[u8;SIZE])->Option<View>{
 if &b[..16]!=MAGIC||word(b,16)!=VERSION{return None}
 let pieces=core::array::from_fn(|i|{let at=44+i*4;Piece{series:u16::from_le_bytes([b[at],b[at+1]]),part:b[at+2],flags:b[at+3]}});
 Some(View{open:word(b,20)==1,frame:word(b,24),screen:word(b,28),overlay:word(b,32)!=0,kind:word(b,36),part:word(b,40),pieces,
  raw:word(b,64),serial:word(b,68),hint:core::array::from_fn(|i|float(b,72+i*4)),scale:float(b,88)})
}
/// Whether the window is open: `open` set and its frame counter still moving.
#[derive(Clone,Copy,Debug)]
pub struct Alive{frame:u32,idle:u32}
impl Alive{
 pub const fn new()->Self{Self{frame:0,idle:u32::MAX}}
 pub fn step(&mut self,v:&View)->bool{
  if v.frame!=self.frame{self.frame=v.frame;self.idle=0;}else{self.idle=self.idle.saturating_add(1);}
  v.open&&self.idle<=STALE
 }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Plan{None,Register(Vec<Recipe>),Remove(usize)}
/// What our shortcut would do for the cursor. A set registers every piece that
/// is not owned, not a quest reward and not registered yet; one piece toggles.
pub fn plan(v:&View,list:&List,recipe:impl Fn(u8,u16)->Option<Recipe>)->Plan{
 if !v.open||v.overlay||!matches!(v.screen,3|4){return Plan::None}
 let lookup=|p:&Piece|if p.series==0||!(1..=5).contains(&p.part){None}else{recipe(p.part,p.series)};
 match v.kind{
  1=>{
   let mut out=Vec::new();
   for p in &v.pieces{
    if p.flags&(OWNED|REWARD)!=0{continue}
    if let Some(r)=lookup(p)&&list.registered(r).is_none()&&!out.contains(&r){out.push(r);}
   }
   if out.is_empty(){Plan::None}else{Plan::Register(out)}
  }
  2=>{
   let Some(p)=(v.part as usize).checked_sub(1).and_then(|i|v.pieces.get(i)) else{return Plan::None};
   let Some(r)=lookup(p) else{return Plan::None};
   match list.registered(r){Some(slot)=>Plan::Remove(slot),None=>Plan::Register(alloc::vec![r])}
  }
  _=>Plan::None,
 }
}
/// Why a piece of the set is not registered (shown in the dialog).
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Reason{Owned,Reward,Registered,NoRecipe}
/// The set's pieces that stay out, with the reason, in part order. Only for a
/// set (kind 1); one piece is registered or removed as it is.
pub fn left_out(v:&View,list:&List,recipe:impl Fn(u8,u16)->Option<Recipe>)->Vec<(u8,u16,Reason)>{
 if v.kind!=1{return Vec::new()}
 let mut out=Vec::new();
 for p in &v.pieces{
  if p.series==0||!(1..=5).contains(&p.part){continue}
  let reason=if p.flags&OWNED!=0{Reason::Owned}else if p.flags&REWARD!=0{Reason::Reward}else{
   match recipe(p.part,p.series){None=>Reason::NoRecipe,Some(r) if list.registered(r).is_some()=>Reason::Registered,Some(_)=>continue}
  };
  out.push((p.part,p.series,reason));
 }
 out
}
/// Free slots missing for a registration (0: it fits).
pub fn shortage(plan:&Plan,list:&List)->usize{
 let Plan::Register(r)=plan else{return 0};
 r.len().saturating_sub(list.slots.iter().filter(|e|e.is_none()).count())
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Outcome{Registered(usize),Removed,Short(usize),Failed}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Dialog{pub plan:Plan,pub yes:bool,pub outcome:Option<Outcome>,pub left_out:Vec<(u8,u16,Reason)>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Answer{Yes,Close}
impl Dialog{
 /// A question, or straight to its message when the set does not fit.
 pub fn new(plan:Plan,list:&List)->Self{
  let short=shortage(&plan,list);
  let yes=matches!(plan,Plan::Register(_));
  Self{plan,yes,outcome:(short>0).then_some(Outcome::Short(short)),left_out:Vec::new()}
 }
 /// Newly pressed raw buttons.
 pub fn input(&mut self,pressed:u32)->Option<Answer>{
  if self.outcome.is_some(){return (pressed&(A|B)!=0).then_some(Answer::Close)}
  if pressed&B!=0{return Some(Answer::Close)}
  if pressed&(LEFT|RIGHT|UP|DOWN)!=0{self.yes=!self.yes;}
  if pressed&A!=0{return Some(if self.yes{Answer::Yes}else{Answer::Close})}
  None
 }
}
/// Newly pressed bits of a held word.
#[derive(Clone,Copy,Default,Debug)]
pub struct Edges{held:u32}
impl Edges{
 pub const fn new()->Self{Self{held:0}}
 pub fn step(&mut self,raw:u32)->u32{let pressed=raw&!self.held;self.held=raw;pressed}
}
/// guest_modal stays set until the buttons held when our dialog closed are
/// released, so the closing press never reaches Skill-Simulator.
#[derive(Clone,Copy,Default,Debug)]
pub struct Gate{latch:u32}
impl Gate{
 pub const fn new()->Self{Self{latch:0}}
 pub fn modal(&mut self,dialog:bool,raw:u32)->bool{
  if dialog{self.latch=raw;return true}
  self.latch&=raw;self.latch!=0
 }
}

#[cfg(test)]
mod tests{
 use super::*;
 use crate::model::Material;
 fn recipe(part:u8,series:u16)->Option<Recipe>{
  (series!=99).then_some(Recipe{kind:part,id:series,level:0,materials:[Material{id:1,count:1},Material::default(),Material::default(),Material::default()],group:0,points:0})
 }
 fn view(kind:u32,part:u32,pieces:[(u16,u8);5])->View{
  View{open:true,screen:3,kind,part,pieces:core::array::from_fn(|i|Piece{series:pieces[i].0,part:i as u8+1,flags:pieces[i].1}),..View::default()}
 }
 #[test]
 fn parses_the_published_layout(){
  let mut b=[0u8;SIZE];b[..16].copy_from_slice(MAGIC);b[16]=1;b[20]=1;b[24]=7;b[28]=4;b[36]=2;b[40]=3;
  b[44+8..44+12].copy_from_slice(&[33,0,3,1]);b[64]=0x20;b[72..76].copy_from_slice(&10f32.to_le_bytes());b[88..92].copy_from_slice(&0.8f32.to_le_bytes());
  let v=parse(&b).unwrap();
  assert_eq!((v.open,v.frame,v.screen,v.kind,v.part,v.raw),(true,7,4,2,3,0x20));
  assert_eq!(v.pieces[2],Piece{series:33,part:3,flags:OWNED});
  assert_eq!((v.hint[0],v.scale),(10.,0.8));
  b[16]=2;assert!(parse(&b).is_none());
 }
 #[test]
 fn a_set_skips_owned_reward_registered_and_missing_pieces(){
  let mut list=List::default();
  list.register(0,recipe(5,40).unwrap(),0).unwrap();
  let v=view(1,0,[(10,0),(20,OWNED),(30,REWARD),(99,0),(40,0)]);
  assert_eq!(plan(&v,&list,recipe),Plan::Register(alloc::vec![recipe(1,10).unwrap()]));
  let none=view(1,0,[(0,0),(20,OWNED),(30,REWARD),(99,0),(40,0)]);
  assert_eq!(plan(&none,&list,recipe),Plan::None);
 }
 #[test]
 fn one_piece_toggles_and_overlays_or_other_screens_do_nothing(){
  let mut list=List::default();
  let v=view(2,2,[(10,0),(20,OWNED),(0,0),(0,0),(0,0)]);
  assert_eq!(plan(&v,&list,recipe),Plan::Register(alloc::vec![recipe(2,20).unwrap()]));
  list.register(3,recipe(2,20).unwrap(),0).unwrap();
  assert_eq!(plan(&v,&list,recipe),Plan::Remove(3));
  assert_eq!(plan(&view(2,3,[(10,0);5]).clone(),&list,|_,_|None),Plan::None);
  for v in [View{overlay:true,..v},View{screen:1,..v},View{open:false,..v},View{part:0,..v},View{kind:0,..v}]{assert_eq!(plan(&v,&list,recipe),Plan::None);}
 }
 #[test]
 fn left_out_pieces_carry_their_reason(){
  let mut list=List::default();
  list.register(0,recipe(5,40).unwrap(),0).unwrap();
  let v=view(1,0,[(10,0),(20,OWNED),(30,REWARD),(99,0),(40,0)]);
  assert_eq!(left_out(&v,&list,recipe),alloc::vec![(2,20,Reason::Owned),(3,30,Reason::Reward),(4,99,Reason::NoRecipe),(5,40,Reason::Registered)]);
  assert!(left_out(&view(2,1,[(10,OWNED);5]),&list,recipe).is_empty());
 }
 #[test]
 fn a_set_that_does_not_fit_is_refused_whole(){
  let mut list=List::default();
  for i in 0..crate::model::CAPACITY-1{list.register(i,recipe(1,100+i as u16).unwrap(),0).unwrap();}
  let plan=Plan::Register(alloc::vec![recipe(1,10).unwrap(),recipe(2,10).unwrap(),recipe(3,10).unwrap()]);
  assert_eq!(shortage(&plan,&list),2);
  let d=Dialog::new(plan,&list);
  assert_eq!(d.outcome,Some(Outcome::Short(2)));
  assert_eq!(shortage(&Plan::Register(alloc::vec![recipe(1,10).unwrap()]),&list),0);
 }
 #[test]
 fn dialog_answers_from_raw_presses(){
  let list=List::default();
  let mut d=Dialog::new(Plan::Register(alloc::vec![recipe(1,10).unwrap()]),&list);
  assert!(d.yes);assert_eq!(d.input(0),None);
  assert_eq!(d.input(1<<14),None);assert!(!d.yes);
  assert_eq!(d.input(A),Some(Answer::Close));
  d.yes=true;assert_eq!(d.input(A),Some(Answer::Yes));
  assert_eq!(d.input(B),Some(Answer::Close));
  let mut remove=Dialog::new(Plan::Remove(0),&list);assert!(!remove.yes);
  remove.outcome=Some(Outcome::Removed);assert_eq!(remove.input(1<<12),None);assert_eq!(remove.input(B),Some(Answer::Close));
 }
 #[test]
 fn modal_holds_until_the_closing_press_is_released(){
  let mut g=Gate::new();
  assert!(g.modal(true,0));assert!(g.modal(true,A));
  assert!(g.modal(false,A));assert!(g.modal(false,A|0x40));
  assert!(!g.modal(false,0x40));assert!(!g.modal(false,A));
 }
 #[test]
 fn window_counts_as_closed_when_its_frame_stops(){
  let mut a=Alive::new();let mut v=View{open:true,frame:1,..View::default()};
  assert!(a.step(&v));for _ in 0..STALE{assert!(a.step(&v));}assert!(!a.step(&v));
  v.frame=2;assert!(a.step(&v));v.open=false;v.frame=3;assert!(!a.step(&v));
 }
}
