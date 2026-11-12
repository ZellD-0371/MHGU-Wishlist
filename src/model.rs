extern crate alloc;
use alloc::{vec::Vec,string::String,format};
pub const CAPACITY:usize=48;
#[derive(Clone,Copy,Default,Debug,PartialEq,Eq)]
pub struct Material{pub id:u16,pub count:u32}
#[derive(Clone,Copy,Default,Debug,PartialEq,Eq)]
pub struct Recipe{pub kind:u8,pub id:u16,pub level:u8,pub materials:[Material;4],pub group:u16,pub points:u16}
impl Recipe{
 pub fn valid(&self)->bool{matches!(self.kind,1..=5|7..=25)&&self.id>0&&self.level<=32&&self.materials.iter().all(|m|m.id<2992&&m.count<=999&&(m.id!=0||m.count==0))&&(self.materials.iter().any(|m|m.id>0&&m.count>0)||self.group>0&&self.points>0)}
 pub fn ready(&self,stock:&[u32;4096],provisions:&[(u16,u16,[u16;3])])->bool{
  if !self.valid(){return false}
  let required=self.totals();
  if required.iter().any(|m|stock[m.id as usize]<(m.count as u32)){return false}
  if self.group==0{return true}
  // Explicit recipe items are consumed first; those same units cannot also pay points.
  let value:u32=provisions.iter().filter(|p|p.2.contains(&self.group)).map(|p|{
   let used=required.iter().find(|m|m.id==p.0).map_or(0,|m|m.count as u32);
   stock[p.0 as usize].saturating_sub(used)*p.1 as u32
  }).sum();value>=self.points as u32
 }
 pub fn totals(&self)->Vec<Material>{let mut out:Vec<Material>=Vec::new();for m in self.materials.iter().filter(|m|m.id>0&&m.count>0){if let Some(v)=out.iter_mut().find(|v|v.id==m.id){v.count+=m.count}else{out.push(*m)}}out}
}
#[derive(Clone,Copy,Default)]
pub struct Entry{pub recipe:Recipe,pub ready:Option<bool>}
#[derive(Clone)]
pub struct List{pub slots:[Option<Entry>;CAPACITY],pub language:usize,pub costs:[u32;CAPACITY],pub open_sound:u32,pub layout:usize,pub ui_scale:u16,pub pad_bindings:[u32;crate::binding_model::COUNT],pub key_bindings:[u32;crate::binding_model::COUNT]}
impl Default for List{fn default()->Self{Self{slots:[None;CAPACITY],language:0,costs:[0;CAPACITY],open_sound:1,layout:0,ui_scale:100,pad_bindings:crate::binding_model::DEFAULT_PAD,key_bindings:crate::binding_model::DEFAULT_KEYS}}}
impl List{
 pub fn copy_preferences(&mut self,settings:&Self){self.open_sound=settings.open_sound;self.pad_bindings=settings.pad_bindings;self.key_bindings=settings.key_bindings;self.language=settings.language;self.layout=settings.layout;self.ui_scale=settings.ui_scale;}
 pub fn reset_preferences(&mut self){self.copy_preferences(&Self::default());}
 pub fn insert(&mut self,slot:usize,recipe:Recipe)->Result<(),&'static str>{
  if slot>=CAPACITY||!recipe.valid(){return Err("invalid")}
  if self.slots.iter().enumerate().any(|(i,e)|i!=slot&&e.is_some_and(|e|e.recipe==recipe)){return Err("duplicate")}
  self.slots[slot]=Some(Entry{recipe,ready:None});Ok(())
 }
 pub fn poll(&mut self,stock:&[u32;4096],provisions:&[(u16,u16,[u16;3])])->Vec<usize>{
  let mut events=Vec::new();for (i,e) in self.slots.iter_mut().enumerate(){if let Some(e)=e{
   let ready=e.recipe.ready(stock,provisions);
   if e.ready==Some(false)&&ready{events.push(i)}e.ready=Some(ready);
  }}events
 }
 pub fn totals(&self)->Vec<Material>{let mut out:Vec<Material>=Vec::new();for e in self.slots.iter().flatten(){for m in e.recipe.totals(){if let Some(v)=out.iter_mut().find(|v|v.id==m.id){v.count+=m.count}else{out.push(m)}}}out.sort_by_key(|m|m.id);out}
 pub fn ordered_totals(&self,stock:&[u32;4096],known:bool)->Vec<Material>{let mut out=self.totals();if known{out.sort_by_key(|m|(stock[m.id as usize]>=m.count,m.id));}out}
 pub fn remove_completed(&mut self,kind:u8,id:u16,level:u8)->bool{let mut changed=false;for i in 0..CAPACITY{if self.slots[i].is_some_and(|e|e.recipe.kind==kind&&e.recipe.id==id&&e.recipe.level==level){self.slots[i]=None;self.costs[i]=0;changed=true;}}changed}
 pub fn registered(&self,recipe:Recipe)->Option<usize>{self.slots.iter().position(|e|e.is_some_and(|e|e.recipe==recipe))}
 pub fn requiring_material(&self,id:u16)->Vec<usize>{
  self.slots.iter().enumerate().filter_map(|(slot,entry)|entry.filter(|e|e.recipe.materials.iter().any(|m|m.id==id&&m.count>0)).map(|_|slot)).collect()
 }
 pub fn encode(&self)->String{
  let mut s=format!("version=1\ninitialized=1\nlanguage={}\nopen_sound={}\nlayout={}\nui_scale={}\n",self.language,self.open_sound,self.layout,self.ui_scale);
  for i in 0..crate::binding_model::COUNT{s.push_str(&format!("pad_binding{}={}\nkey_binding{}={}\n",i,self.pad_bindings[i],i,self.key_bindings[i]));}
  self.append_entries(&mut s);s
 }
 pub fn encode_character(&self)->String{let mut s=String::from("version=1\ninitialized=1\n");self.append_entries(&mut s);s}
 fn append_entries(&self,s:&mut String){
  for (i,e) in self.slots.iter().enumerate(){if let Some(e)=e{let r=e.recipe;s.push_str(&format!("slot{}={},{},{},{},{}",i,r.kind,r.id,r.level,r.group,r.points));for m in r.materials{s.push_str(&format!(",{},{}",m.id,m.count))}s.push('\n');s.push_str(&format!("cost{}={}\n",i,self.costs[i]));}}
 }
 pub fn decode(text:&str)->Result<Self,()>{
  if !text.lines().any(|l|l=="version=1"){return Err(())}let mut list=Self::default();
  for line in text.lines(){let Some((key,value))=line.split_once('=') else{continue};
   if let Some(i)=key.strip_prefix("pad_binding"){if let Ok(i)=i.parse::<usize>(){if i<crate::binding_model::COUNT{if let Ok(v)=value.parse::<u32>(){if v>0&&v&!crate::binding_model::PAD_MASK==0{list.pad_bindings[i]=v;}}}}}
   if let Some(i)=key.strip_prefix("key_binding"){if let Ok(i)=i.parse::<usize>(){if i<crate::binding_model::COUNT{if let Ok(v)=value.parse::<u32>(){if crate::binding_model::valid_key(v){list.key_bindings[i]=v;}}}}}
   if key=="layout"{list.layout=value.parse().map_err(|_|())?;if list.layout>=3{return Err(())}}
   if key=="ui_scale"{list.ui_scale=value.parse().map_err(|_|())?;if !(50..=100).contains(&list.ui_scale){return Err(())}}
   if key=="open_sound"{list.open_sound=value.parse().map_err(|_|())?;}
   if key=="language"{list.language=value.parse().map_err(|_|())?;if list.language>=9{return Err(())}}
   if let Some(slot)=key.strip_prefix("cost"){let i:usize=slot.parse().map_err(|_|())?;if i>=CAPACITY{return Err(())}list.costs[i]=value.parse().map_err(|_|())?;}
   if let Some(slot)=key.strip_prefix("slot"){
    let i:usize=slot.parse().map_err(|_|())?;if i>=CAPACITY||list.slots[i].is_some(){return Err(())}
    let v:Vec<u16>=value.split(',').map(|s|s.parse::<u16>()).collect::<Result<_,_>>().map_err(|_|())?;
    if v.len()!=13||v[0]>255||v[2]>255{return Err(())}
    let r=Recipe{kind:v[0] as u8,id:v[1],level:v[2] as u8,group:v[3],points:v[4],materials:core::array::from_fn(|n|Material{id:v[5+n*2],count:v[6+n*2] as u32})};
    list.insert(i,r).map_err(|_|())?;
   }
  }Ok(list)
 }
}
