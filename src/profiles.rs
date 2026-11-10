//! Sidecar migration policy, independent of game memory and filesystem APIs.
use crate::model::List;
pub struct Plan{pub list:List,pub claim:Option<usize>,pub write:bool}
pub fn load(slot:usize,initialized:bool,mut list:List,legacy:&List,owner:Option<usize>)->Plan{
 let assigned=owner.unwrap_or(slot);
 if !initialized&&assigned==slot{list=legacy.clone();}
 list.copy_preferences(legacy);
 Plan{list,claim:if !initialized&&owner.is_none(){Some(slot)}else{None},write:!initialized}
}
