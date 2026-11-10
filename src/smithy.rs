//! The smithy list's kind 6 means decorations, unlike equipment-box kind 6 (charms).
//! +28DA is a category-material group for equipment recipes, not a decoration ID.
use crate::model::{Material,Recipe};
#[derive(Clone,Copy,Debug)]
pub struct SelectedRow{
 pub list_kind:u16,pub id:u16,pub level_index:u8,pub group:u16,pub points:u8,pub materials:[Material;4],
}
impl SelectedRow{
 pub fn pane(&self)->u32{if self.group!=0{11}else{10}}
 pub fn resolve(&self,recipes:&[Recipe],armor_upgrades:&[Recipe])->Option<Recipe>{
  let kind=match self.list_kind{6=>25,1..=5|7..=21=>self.list_kind as u8,_=>return None};
  if self.id==0{return None}
  let raw=Recipe{kind,id:self.id,materials:self.materials,..Recipe::default()};
  let materials=raw.totals();let level=u32::from(self.level_index)+1;
  let armor=(1..=5).contains(&kind);
  let matches=|r:&&Recipe|(r.kind==kind||(armor&&r.kind==0))&&r.id==self.id&&r.group==self.group&&r.points==u16::from(self.points)&&r.totals()==materials;
  recipes.iter().chain(armor_upgrades).filter(matches).find(|r|u32::from(r.level)==level)
   .or_else(||recipes.iter().filter(matches).find(|r|r.level==0)).map(|r|Recipe{kind,..*r})
 }
}
