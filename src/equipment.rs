use crate::model::Recipe;
include!("equipment_generated.rs");
include!("categories_generated.rs");
// Kind order differs from its numeric IDs, but each kind's rows are contiguous
// and ordered by (ID, level). Derive tiny ranges at compile time; no runtime index allocation.
const RANGES:[(usize,usize);26]={
 let mut ranges=[(0,0);26];let mut i=0;
 while i<DETAILS.len(){let d=&DETAILS[i];let k=d.kind as usize;assert!(k<26);let(start,end)=ranges[k];if end!=0{assert!(end==i);let previous=&DETAILS[i-1];assert!(previous.id<d.id||(previous.id==d.id&&previous.level<=d.level));ranges[k]=(start,i+1);}else{ranges[k]=(i,i+1);}i+=1;}ranges
};
pub fn get(r:Recipe)->Option<&'static Detail>{
 let &(start,end)=RANGES.get(r.kind as usize)?;let rows=&DETAILS[start..end];let at=rows.partition_point(|d|d.id<r.id);let lv=r.level.max(1);
 rows[at..].iter().take_while(|d|d.id==r.id).find(|d|d.level==0||d.level==lv)
}
pub fn cost(r:Recipe)->u32{get(r).map_or(0,|d|d.cost)}
pub fn icon_tint(r:Recipe)->u32{crate::presentation::rarity(get(r).map_or(1,|d|d.rarity))}
pub fn skill(id:u16,lang:usize)->&'static str{SKILLS.iter().find(|s|s.0==id).map_or("?",|s|s.1[lang])}
