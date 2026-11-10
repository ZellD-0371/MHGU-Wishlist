//! Pure identity rules shared by native icon badges and direct smithy removal.
use crate::model::{List};
#[derive(Clone,Copy,Default,Debug,PartialEq,Eq)]
pub struct Gear{pub kind:u8,pub id:u16,pub level_index:u8}
impl Gear{
 pub fn from_native(packed:u16,id:u16)->Self{Self{kind:(packed&31) as u8,id,level_index:((packed>>5)&31) as u8}}
 pub fn registered(&self,list:&List)->Option<usize>{
  // A box icon represents the owned equipment before its next level-up. Production
  // registrations refer to the equipment itself; upgrade registrations name its target level.
  list.slots.iter().position(|e|e.is_some_and(|e|e.recipe.kind==self.kind&&e.recipe.id==self.id&&(e.recipe.level==0||e.recipe.level==self.level_index+2)))
 }
}
