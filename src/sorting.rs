//! Explicit sorting only. The display array and registration ranks travel together.
#[derive(Clone,Copy,Default)]
pub struct Dialog{pub secondary:bool,pub criterion:usize,pub category:usize,pub message:u8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Action{None,Cancel,Apply}
impl Dialog{
 pub fn input(&mut self,direction:i32,accept:bool,back:bool)->Action{
  if back{self.message=0;if self.secondary{self.secondary=false;self.category=0;return Action::None}return Action::Cancel}
  if direction!=0{self.message=0;if self.secondary{self.category=(self.category as i32+direction).rem_euclid(6) as usize}else{self.criterion=(self.criterion as i32+direction).rem_euclid(3) as usize}}
  if accept{if self.secondary||self.criterion==0{return Action::Apply}self.secondary=true;self.category=0;self.message=0;}
  Action::None
 }
}
pub fn preferred(kind:u8,category:usize)->bool{match category{1=>(7..=21).contains(&kind),2=>(1..=5).contains(&kind),3=>kind==25,4=>kind==22,5=>matches!(kind,23|24),_=>false}}
