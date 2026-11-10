use crate::model::Recipe;
include!("equipment_generated.rs");
include!("categories_generated.rs");
pub fn get(r:Recipe)->Option<&'static Detail>{let lv=if r.level==0{1}else{r.level};DETAILS.iter().find(|d|d.kind==r.kind&&d.id==r.id&&(d.level==0||d.level==lv))}
pub fn cost(r:Recipe)->u32{get(r).map_or(0,|d|d.cost)}
pub fn skill(id:u16,lang:usize)->&'static str{SKILLS.iter().find(|s|s.0==id).map_or("?",|s|s.1[lang])}
