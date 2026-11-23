//! Pure KatSmithy helpers: module identity, NRO import-slot lookup and the
//! direction gate applied to its Forge input imports while the wishlist owns input.

/// All markers must appear in the module's read-only segment. Our own mods that look for
/// KatSmithy (Better Skillinfo, the Korean patch, Wishlist) carry the same markers, so a
/// module with one of their log prefixes is never KatSmithy.
pub fn identifies(ro:&[u8])->bool{
 let has=|s:&[u8]|ro.windows(s.len()).any(|w|w==s);
 [b"[KatSmithy]".as_slice(),b"katsmithy.ini".as_slice(),b"Skill points by piece".as_slice()].iter().all(|s|has(s))
 &&![b"[better-skillinfo]".as_slice(),b"[korean-".as_slice(),b"[wishlist]".as_slice(),b"[skill-sim".as_slice()].iter().any(|s|has(s))
}

/// Forge input imports KatSmithy reads its tree navigation from.
pub const INPUT_IMPORTS:[&[u8];4]=[b"forge_input_isDown",b"forge_input_isPressed",b"forge_input_getStickL",b"forge_input_getStickR"];
const DT_PLTRELSZ:i32=2;const DT_STRTAB:i32=5;const DT_SYMTAB:i32=6;const DT_STRSZ:i32=10;const DT_JMPREL:i32=23;
const R_ARM_JUMP_SLOT:u32=22;

fn word(image:&[u8],at:u32)->Option<u32>{
 let at=at as usize;image.get(at..at.checked_add(4)?).map(|b|u32::from_le_bytes([b[0],b[1],b[2],b[3]]))
}
/// Dynamic values may be module offsets (as in the file) or already absolute.
fn offset(value:u32,base:u32,len:usize)->Option<u32>{
 let v=if value>=base&&value-base<len as u32{value-base}else{value};
 ((v as usize)<len).then_some(v)
}
/// Module-relative GOT slot of each named import, from MOD0 -> dynamic -> JMPREL.
pub fn import_slots<const N:usize>(image:&[u8],base:u32,names:[&[u8];N])->[Option<u32>;N]{
 let mut out=[None;N];
 let Some(mod0)=word(image,4) else{return out};
 if image.get(mod0 as usize..mod0 as usize+4)!=Some(b"MOD0".as_slice()){return out}
 let Some(rel)=word(image,mod0+4) else{return out};
 let dynamic=mod0.wrapping_add(rel);
 let (mut strtab,mut strsz,mut symtab,mut jmprel,mut pltsz)=(None,None,None,None,None);
 for i in 0..512u32{
  let (Some(tag),Some(value))=(word(image,dynamic+i*8),word(image,dynamic+i*8+4)) else{return out};
  let tag=tag as i32;if tag==0{break}
  let len=image.len();
  match tag{DT_STRTAB=>strtab=offset(value,base,len),DT_SYMTAB=>symtab=offset(value,base,len),DT_JMPREL=>jmprel=offset(value,base,len),DT_STRSZ=>strsz=Some(value),DT_PLTRELSZ=>pltsz=Some(value),_=>{}}
 }
 let (Some(strtab),Some(strsz),Some(symtab),Some(jmprel),Some(pltsz))=(strtab,strsz,symtab,jmprel,pltsz) else{return out};
 if pltsz>0x10000||strsz>0x100000{return out}
 for r in (0..pltsz).step_by(8){
  let (Some(slot),Some(info))=(word(image,jmprel+r),word(image,jmprel+r+4)) else{return out};
  if info&0xff!=R_ARM_JUMP_SLOT{continue}
  let Some(name)=word(image,symtab+(info>>8)*16) else{continue};
  if name>=strsz{continue}
  let start=(strtab+name) as usize;
  let Some(tail)=image.get(start..(strtab+strsz) as usize) else{continue};
  let end=tail.iter().position(|b|*b==0).unwrap_or(tail.len());
  for (i,wanted) in names.iter().enumerate(){if &tail[..end]==*wanted&&(slot as usize)<image.len(){out[i]=Some(slot);}}
 }
 out
}

/// Forge's button indices (Npad order: 0 A, 1 B, ... 12-15 D-pad, 16-23 sticks).
pub const BUTTONS:u32=64;
/// Returns whether a button query must read as not held. While the wishlist owns
/// input every button is withheld; one held when ownership ends (the A that
/// confirmed a registration, for example) stays withheld until it is released.
pub fn button_blocked(bit:u32,down:bool,owned:bool,latch:&mut u64)->bool{
 if bit>=BUTTONS{return false}
 let mask=1u64<<bit;
 if !down{*latch&=!mask;return false}
 if owned{*latch|=mask;return true}
 *latch&mask!=0
}
/// At the end of ownership, everything still held is withheld until released,
/// even if KatSmithy had not queried it while the wishlist was open.
pub fn release_latch(latch:u64,held:u64)->u64{latch|held}
/// Each frame: a latched button that is up again is free, whether or not
/// KatSmithy asked about it in between.
pub fn refresh_latch(latch:u64,held:u64)->u64{latch&held}
const STICK_REST:f32=0.25;
/// The same latch for an analog stick, released once it returns near center.
pub fn stick_blocked(x:f32,y:f32,owned:bool,latch:&mut bool)->bool{
 let moved=x.abs()>=STICK_REST||y.abs()>=STICK_REST;
 if owned{*latch=moved;return true}
 if !moved{*latch=false}
 *latch
}

// No fixed addresses: KatSmithy's armor list and weapon tree are found by their
// content in its data segment, so a rebuild that moves its globals keeps working.
// Only the record shapes below are assumed; anything else fails the checks and
// leaves the game's own pins in place.
fn u16_at(m:&[u8],at:usize)->Option<u16>{m.get(at..at.checked_add(2)?).map(|b|u16::from_le_bytes([b[0],b[1]]))}
fn i16_at(m:&[u8],at:usize)->Option<i16>{u16_at(m,at).map(|v|v as i16)}
fn u32_at(m:&[u8],at:usize)->Option<u32>{m.get(at..at.checked_add(4)?).map(|b|u32::from_le_bytes([b[0],b[1],b[2],b[3]]))}
pub fn f32_at(m:&[u8],at:usize)->Option<f32>{u32_at(m,at).map(f32::from_bits)}

/// Armor list state record: count, cursor row, cursor piece (0 head .. 4 legs),
/// selected set ID, first visible row (f32). Set IDs (u16) end at `count`'s address,
/// the unused tail zero-filled.
pub const ARMOR_ROW:usize=4;pub const ARMOR_PIECE:usize=8;pub const ARMOR_SELECTED:usize=12;pub const ARMOR_TOP:usize=16;
#[derive(Clone,Copy,PartialEq,Debug)]
pub struct ArmorState{pub ids:usize,pub count:u32,pub row:u32,pub piece:u32,pub top:f32}
/// The armor record at `at`, or None. Lists hold many sets, so short ones are rejected.
pub fn armor_at(m:&[u8],at:usize)->Option<ArmorState>{
 let (count,row,piece,selected)=(u32_at(m,at)?,u32_at(m,at+ARMOR_ROW)?,u32_at(m,at+ARMOR_PIECE)?,u32_at(m,at+ARMOR_SELECTED)?);
 let top=f32_at(m,at+ARMOR_TOP)?;
 if !(16..=4096).contains(&count)||row>=count||piece>4||selected==0||selected>0xffff{return None}
 if !top.is_finite()||top<0.||top>row as f32||(top as u32) as f32!=top{return None}
 let mut zeros=0usize;while zeros<8192&&at>=2*(zeros+1)&&u16_at(m,at-2*(zeros+1))==Some(0){zeros+=1;}
 let ids=at.checked_sub(2*(zeros+count as usize))?;
 let mut seen=[0u64;1024];
 for i in 0..count as usize{let id=u16_at(m,ids+2*i)?;if id==0{return None}let (w,b)=(id as usize/64,id as u64%64);if seen[w]>>b&1!=0{return None}seen[w]|=1<<b;}
 (u16_at(m,ids+2*row as usize)?==selected as u16).then_some(ArmorState{ids,count,row,piece,top})
}
pub fn find_armor(m:&[u8])->Option<usize>{(4..m.len().saturating_sub(20)).step_by(4).find(|&at|armor_at(m,at).is_some())}
/// Per-frame read of a record already found by `armor_at`. Moving the cursor
/// updates the row first and the selected set a few frames later, and the first
/// row scrolls through fractions, so only the shape is checked here.
pub fn armor_live(m:&[u8],at:usize,ids:usize)->Option<ArmorState>{
 let (count,row,piece)=(u32_at(m,at)?,u32_at(m,at+ARMOR_ROW)?,u32_at(m,at+ARMOR_PIECE)?);let top=f32_at(m,at+ARMOR_TOP)?;
 let capacity=(at.checked_sub(ids)?/2) as u32;
 ((1..=capacity).contains(&count)&&row<count&&piece<=4&&top.is_finite()&&top>=0.&&top<count as f32).then_some(ArmorState{ids,count,row,piece,top})
}
pub fn armor_id(m:&[u8],s:ArmorState,row:u32)->Option<u16>{if row<s.count{u16_at(m,s.ids+2*row as usize)}else{None}}

// The list's geometry in KatSmithy's 1280x720 space (measured at 1920x1080, scale 1.5).
const ICON_X:f32=784.5/1.5;const ICON_PITCH_X:f32=60./1.5;
const ICON_Y:f32=155./1.5;const ROW_PITCH:f32=98.75/1.5;const ICON_HALF:f32=25./1.5;
const LIST_TOP:f32=88./1.5;const LIST_BOTTOM:f32=715./1.5;
/// Pin position (head drawn at +3,+3) on the top-right corner of a piece icon,
/// or None while that icon is scrolled outside the list.
pub fn armor_pin(top:f32,row:u32,piece:u32)->Option<(f32,f32)>{
 if piece>4||!top.is_finite(){return None}
 let cx=ICON_X+piece as f32*ICON_PITCH_X;let cy=ICON_Y+(row as f32-top)*ROW_PITCH;
 if cy-ICON_HALF<LIST_TOP||cy+ICON_HALF>LIST_BOTTOM{return None}
 Some((cx+ICON_HALF-5.,cy-ICON_HALF-1.))
}
/// Visible rows for a list position: the partially shown row is included and
/// left to `armor_pin` to clip.
pub fn visible_rows(top:f32,count:u32)->core::ops::Range<u32>{
 if !top.is_finite()||top<0.{return 0..0}
 let first=top as u32;first.min(count)..(first+8).min(count)
}

// KatSmithy draws its UI in a 1280x720 space centered on the screen (its renderer
// computes this from the viewport each frame), so it follows 21:9, 32:9 and 16:10.
pub fn transform(width:f32,height:f32)->Option<(f32,f32,f32)>{
 if !(width>0.&&height>0.&&width.is_finite()&&height.is_finite()){return None}
 let scale=(width/1280.).min(height/720.);Some((width*0.5-640.*scale,(height-720.*scale)*0.5,scale))
}

// KatSmithy's weapon tree renderer, found from code rather than by address: the
// function that logs "FULL sel(id*100+lv)". From it come the screen transform it
// draws with and a check that its tree layout constants are the ones used here.
pub const RENDERER_LOG:&[u8]=b"FULL sel(id*100+lv)";
/// Layout constants of the tree panel (cell, half cell, panel size/top, label
/// column, minimum scale) that must all appear in the renderer's literal pool.
pub const TREE_CONSTANTS:[f32;7]=[54.,36.,702.,590.,62.,126.,0.85];
const RENDERER_BEFORE:usize=0x3000;const RENDERER_AFTER:usize=0x6000;
fn find_bytes(hay:&[u8],needle:&[u8])->Option<usize>{if needle.is_empty()||hay.len()<needle.len(){return None}(0..=hay.len()-needle.len()).find(|&i|&hay[i..i+needle.len()]==needle)}
fn word_at(m:&[u8],at:usize)->Option<u32>{u32_at(m,at)}
/// PC-relative address loads: `ldr rT,[pc,#lit]` then `add rD,pc,rT` within 16
/// instructions. Yields (add address, destination register, target).
fn pic_loads(m:&[u8],from:usize,to:usize,mut f:impl FnMut(usize,u32,usize)->bool)->bool{
 let to=to.min(m.len().saturating_sub(4));let mut p=from&!3;
 while p<to{
  let w=word_at(m,p).unwrap_or(0);
  if w>>28==0xe&&(w&0x0f7f_0000)==0x051f_0000{
   let rt=(w>>12)&15;let imm=(w&0xfff) as usize;
   let lit=if w&(1<<23)!=0{p+8+imm}else{(p+8).wrapping_sub(imm)};
   if let Some(v)=word_at(m,lit){
    for q in (p+4..(p+64).min(to)).step_by(4){
     let x=word_at(m,q).unwrap_or(0);
     if x>>28==0xe&&(x&0x0fff_0ff0)==0x008f_0000&&(x&15)==rt{
      if f(q,(x>>12)&15,(q+8).wrapping_add(v as usize)){return true}break
     }
     if (x>>12)&15==rt{break}
    }
   }
  }
  p+=4;
 }
 false
}
/// Whether an ARM instruction overwrites core register `r` (loads and data
/// processing; compares and stores do not).
fn writes(x:u32,r:u32)->bool{
 if x>>28!=0xe||(x>>12)&15!=r{return false}
 match (x>>26)&3{
  0=>!(8..=11).contains(&((x>>21)&15)),
  1=>x&(1<<20)!=0,
  _=>false,
 }
}
#[derive(Clone,Copy,PartialEq,Debug)]
pub struct Renderer{pub at:usize,pub view:Option<usize>,pub constants:bool}
/// `image` is KatSmithy's loaded image (text then rodata), `text` its code size.
pub fn renderer(image:&[u8],text:usize)->Option<Renderer>{
 let s=find_bytes(image,RENDERER_LOG)?;
 let start=image[..s].iter().rposition(|b|*b==0).map_or(0,|i|i+1);
 let mut at=None;pic_loads(image,0,text,|q,_,target|{if target==start{at=Some(q);true}else{false}});
 let at=at?;let (lo,hi)=(at.saturating_sub(RENDERER_BEFORE),(at+RENDERER_AFTER).min(text));
 // The transform: a PC-relative global read as floats at +4 (origin x) and +0xC (scale).
 let mut view=None;
 pic_loads(image,lo,hi,|q,rd,target|{
  let mut has=[false;2];
  for i in 1..=8{let x=word_at(image,q+4*i).unwrap_or(0);
   if x>>28==0xe&&(x&0x0fb0_0f00)==0x0d90_0a00&&(x>>16)&15==rd{if x&0xff==1{has[0]=true}if x&0xff==3{has[1]=true}}
   if writes(x,rd){break}}
  if has[0]&&has[1]{view=Some(target);true}else{false}
 });
 // Literal floats loaded by `vldr sN,[pc,#imm]` in the renderer.
 let mut found=[false;TREE_CONSTANTS.len()];
 for p in (lo..hi).step_by(4){
  let x=word_at(image,p).unwrap_or(0);
  if x>>28!=0xe||(x&0x0f3f_0f00)!=0x0d1f_0a00{continue}
  let imm=((x&0xff)*4) as usize;let lit=if x&(1<<23)!=0{p+8+imm}else{(p+8).wrapping_sub(imm)};
  if let Some(v)=f32_at(image,lit){for (i,c) in TREE_CONSTANTS.iter().enumerate(){if (v-c).abs()<1e-4{found[i]=true}}}
 }
 Some(Renderer{at,view,constants:found.iter().all(|f|*f)})
}

// Weapon tree record (relative to its start): columns, first column and row of
// each line (by line ID), displayed rows with their row position, height and line ID,
// then the selection, view mode, scroll and the material dialog flag. KatSmithy 1.5.0
// doubled its capacities: more lines and rows, so everything after the first table
// moved. Both layouts are accepted; `tab` (Tree / Forge / Upgrade) exists from 1.5.0.
#[derive(Debug,PartialEq)]
pub struct TreeLayout{
 pub columns:usize,pub line_col:usize,pub line_row:usize,pub rows:usize,pub row_pos:usize,pub row_height:usize,pub row_line:usize,
 pub selected_id:usize,pub selected_level:usize,pub mode:usize,pub scroll_y:usize,pub scroll_x:usize,
 // 1 while KatSmithy's material selection dialog (forge/upgrade) covers the tree.
 pub dialog:usize,pub end:usize,pub lines:u16,pub max_rows:u32,pub tab:Option<usize>,
 // 1.5.0: what it drew this frame (tab buttons, tree nodes, bottom buttons), 0x18 bytes
 // each: u32 (value << 16 | type), u32 level, then the rectangle. Type 1 is a node
 // (value = line ID). The Forge and Upgrade tabs leave out nodes they hide.
 pub hits:Option<(usize,usize)>,
}
/// Index 0: up to KatSmithy 1.4. Index 1: KatSmithy 1.5.0 (measured in a running game).
pub const TREE_LAYOUTS:[TreeLayout;2]=[
 TreeLayout{columns:0x10,line_col:0x14,line_row:0x154,rows:0x294,row_pos:0x298,row_height:0x3ec,row_line:0x540,
  selected_id:0x26b0,selected_level:0x26b8,mode:0x26d0,scroll_y:0x2754,scroll_x:0x2758,dialog:0x30ec,end:0x30f0,lines:160,max_rows:0xaa,tab:None,hits:None},
 TreeLayout{columns:0x10,line_col:0x14,line_row:0x254,rows:0x494,row_pos:0x498,row_height:0x6ec,row_line:0x940,
  selected_id:0x449c,selected_level:0x44b8,mode:0x44d0,scroll_y:0x4554,scroll_x:0x4558,dialog:0x5698,end:0xf158,lines:288,max_rows:0x12a,tab:Some(0xf154),hits:Some((0x8ed0,0xeed0))},
];
#[derive(Clone,Copy,PartialEq,Debug)]
pub struct TreeState{pub at:usize,pub columns:i32,pub rows:u32,pub layout:u8}
impl TreeState{pub fn layout(&self)->&'static TreeLayout{&TREE_LAYOUTS[self.layout as usize]}}
/// The weapon tree at `at` in `layout`: rows start at 0 and grow by 3-8 half cells,
/// each row's line is unique, and that line's own row and column agree with the row table.
pub fn tree_at(m:&[u8],at:usize,layout:u8)->Option<TreeState>{
 let l=TREE_LAYOUTS.get(layout as usize)?;
 if at.checked_add(l.end)?>m.len(){return None}
 let rows=u32_at(m,at+l.rows)?;let columns=u32_at(m,at+l.columns)? as i32;
 if !(1..=l.max_rows).contains(&rows)||!(1..=64).contains(&columns){return None}
 let mut prev=-1i32;let mut seen=[0u64;5];
 for i in 0..rows as usize{
  let pos=i16_at(m,at+l.row_pos+2*i)? as i32;let height=i16_at(m,at+l.row_height+2*i)?;let line=i16_at(m,at+l.row_line+2*i)?;
  if (i==0&&pos!=0)||(i>0&&!(3..=8).contains(&(pos-prev)))||!(1..=4).contains(&height){return None}
  if !(0..l.lines as i16).contains(&line){return None}let line=line as usize;
  if seen[line/64]>>(line%64)&1!=0{return None}seen[line/64]|=1<<(line%64);
  if i16_at(m,at+l.line_row+2*line)? as i32!=pos{return None}
  let col=i16_at(m,at+l.line_col+2*line)? as i32;if !(0..columns).contains(&col){return None}
  prev=pos;
 }
 Some(TreeState{at,columns,rows,layout})
}
/// The first tree record in either layout, as its offset with the layout index in
/// the low two bits (offsets are word aligned).
pub fn find_tree(m:&[u8])->Option<usize>{
 (0..m.len()).step_by(4).find_map(|at|(0..TREE_LAYOUTS.len() as u8).find(|&l|tree_at(m,at,l).is_some()).map(|l|at|l as usize))
}
/// Per-frame read of a tree already found by `find_tree` (offset | layout): its shape only.
pub fn tree_live(m:&[u8],found:usize)->Option<TreeState>{
 let (at,layout)=(found&!3,(found&3) as u8);let l=TREE_LAYOUTS.get(layout as usize)?;
 if at.checked_add(l.end)?>m.len(){return None}
 let rows=u32_at(m,at+l.rows)?;let columns=u32_at(m,at+l.columns)? as i32;
 ((1..=l.max_rows).contains(&rows)&&(1..=64).contains(&columns)&&i16_at(m,at+l.row_pos)?==0).then_some(TreeState{at,columns,rows,layout})
}
/// Displayed line `i` as (line id, first column, row).
pub fn tree_line(m:&[u8],t:TreeState,i:u32)->Option<(u16,i16,i16)>{
 let l=t.layout();if i>=t.rows{return None}let line=i16_at(m,t.at+l.row_line+2*i as usize)? as u16;
 Some((line,i16_at(m,t.at+l.line_col+2*line as usize)?,i16_at(m,t.at+l.line_row+2*line as usize)?))
}
pub fn tree_u32(m:&[u8],t:TreeState,field:usize)->Option<u32>{u32_at(m,t.at+field)}
pub const TREE_HIT_SIZE:usize=0x18;pub const TREE_HIT_MAX:u32=1024;
/// Whether the node (line, level) is drawn now. Layouts without a hit list draw every node.
pub fn tree_node_drawn(m:&[u8],t:TreeState,line:u16,level:u32)->bool{
 let Some((list,count))=t.layout().hits else{return true};
 let Some(count)=u32_at(m,t.at+count) else{return false};
 (0..count.min(TREE_HIT_MAX) as usize).any(|i|{let at=t.at+list+i*TREE_HIT_SIZE;
  u32_at(m,at)==Some(u32::from(line)<<16|1)&&u32_at(m,at+4)==Some(level)})
}

// Tree panel in KatSmithy's 1280x720 space: cells are 54 units wide, rows 18
// units per half cell, icons 32 units, all times the panel scale.
const TREE_LEFT:f32=152.;const TREE_TOP:f32=74.;const TREE_WIDTH:f32=552.;const TREE_HEIGHT:f32=566.;
const TREE_ORIGIN_X:f32=157.;const TREE_FIT_WIDTH:f32=536.;
/// The panel scale KatSmithy derives from the column count.
pub fn tree_scale(columns:i32)->f32{
 if columns<=0{return 0.85}
 let u=TREE_FIT_WIDTH/(columns as f32*54.);
 if u<0.85{0.85}else if u>1.25{1.25}else{u}
}
/// Pin on the top-right corner of a weapon node, in KatSmithy's 1280x720 space,
/// or None while the node is scrolled outside the tree panel.
pub fn weapon_pin(line_col:i16,line_row:i16,level:u32,columns:i32,scroll_x:f32,scroll_y:f32)->Option<(f32,f32)>{
 if level==0||!scroll_x.is_finite()||!scroll_y.is_finite(){return None}
 let u=tree_scale(columns);
 let left=TREE_ORIGIN_X-scroll_x+54.*u*(line_col as f32+level as f32-1.);
 let center=TREE_TOP-scroll_y+18.*u*(line_row as f32+1.);
 let (right,top,bottom)=(left+32.*u,center-16.*u,center+16.*u);
 if left<TREE_LEFT||right>TREE_LEFT+TREE_WIDTH||top<TREE_TOP||bottom>TREE_TOP+TREE_HEIGHT{return None}
 Some((right-5.,top-1.))
}
/// A weapon recipe at `recipe_level` is node level `level`: level 0 forges level 1.
pub fn weapon_node_level(recipe_level:u8)->u32{u32::from(recipe_level.max(1))}

#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn module_identity_requires_all_markers(){
  assert!(identifies(b"[KatSmithy] katsmithy.ini Skill points by piece"));
  assert!(!identifies(b"[KatSmithy] katsmithy.ini"));
  // A mod that only looks for KatSmithy carries the markers too.
  assert!(!identifies(b"[KatSmithy] katsmithy.ini Skill points by piece [better-skillinfo]"));
  assert!(!identifies(b"[KatSmithy] katsmithy.ini Skill points by piece [korean-katsmithy]"));
 }
 fn put(image:&mut Vec<u8>,at:usize,v:u32){if image.len()<at+4{image.resize(at+4,0);}image[at..at+4].copy_from_slice(&v.to_le_bytes());}
 // A minimal NRO image: MOD0 -> dynamic -> JMPREL with two jump slots.
 fn image(absolute:u32)->Vec<u8>{
  let mut m=vec![0u8;0x400];
  put(&mut m,4,0x40);m[0x40..0x44].copy_from_slice(b"MOD0");put(&mut m,0x44,0x40); // dynamic at 0x80
  let strtab=0x200;let names=b"\0forge_input_isDown\0forge_input_getStickL\0";m[strtab..strtab+names.len()].copy_from_slice(names);
  for (i,(tag,v)) in [(DT_STRTAB,strtab as u32),(DT_STRSZ,names.len() as u32),(DT_SYMTAB,0x300),(DT_JMPREL,0x180),(DT_PLTRELSZ,16),(0,0)].into_iter().enumerate(){
   put(&mut m,0x80+i*8,tag as u32);put(&mut m,0x84+i*8,if tag==DT_STRSZ||tag==DT_PLTRELSZ||tag==0{v}else{v+absolute});
  }
  put(&mut m,0x310,1);put(&mut m,0x320,20); // symbols 1 and 2: name offsets
  put(&mut m,0x180,0x3a0);put(&mut m,0x184,(1<<8)|22);
  put(&mut m,0x188,0x3a4);put(&mut m,0x18c,(2<<8)|22);
  m
 }
 #[test]fn import_slots_follow_the_dynamic_section(){
  for absolute in [0,0x033e0000]{
   let slots=import_slots(&image(absolute),0x033e0000,INPUT_IMPORTS);
   assert_eq!(slots,[Some(0x3a0),None,Some(0x3a4),None]);
  }
  assert_eq!(import_slots(&[0u8;64],0,INPUT_IMPORTS),[None;4]);
 }
 #[test]fn buttons_are_withheld_while_owned_and_until_released(){
  let mut latch=0;
  for bit in [0,15,40]{assert!(button_blocked(bit,true,true,&mut latch));}
  assert!(button_blocked(0,true,false,&mut latch)); // the confirming A, still held after closing
  assert!(!button_blocked(0,false,false,&mut latch));
  assert!(!button_blocked(0,true,false,&mut latch)); // a fresh press passes
  assert!(!button_blocked(64,true,true,&mut latch));
 }
 #[test]fn button_held_at_close_is_withheld_even_if_never_queried_while_open(){
  let mut latch=release_latch(0,1);
  assert!(button_blocked(0,true,false,&mut latch));
  assert!(!button_blocked(0,false,false,&mut latch));
  assert!(!button_blocked(0,true,false,&mut latch));
 }
 #[test]fn latched_b_is_freed_by_the_frame_refresh_even_if_never_queried_up(){
  // Wishlist closed with B held; KatSmithy only asks about B while it is down.
  let mut latch=release_latch(0,1<<1);
  latch=refresh_latch(latch,0); // a frame with B up, no KatSmithy query
  assert!(!button_blocked(1,true,false,&mut latch)); // the next B press reaches KatSmithy
  assert_eq!(refresh_latch(1<<1|1,1),1); // still-held A stays withheld
 }
 #[test]fn armor_pin_follows_the_screenshot_rows(){
  // Bherna chest at 3/87 (top 0, row 2): icon box x 820..869, y 327.5..377.5 at 1920x1080.
  let (x,y)=armor_pin(0.,2,1).unwrap();
  assert!(((x+3.)*1.5-866.).abs()<3.&&((y+3.)*1.5-330.).abs()<3.,"{x},{y}");
  // After scrolling to top 1, the same row moves up one pitch.
  let (_,y1)=armor_pin(1.,2,1).unwrap();assert!(((y-y1)*1.5-98.75).abs()<0.01);
  // The seventh row is only partly visible; above the top is clipped too.
  assert!(armor_pin(0.,5,0).is_some());assert!(armor_pin(0.,6,0).is_none());assert!(armor_pin(3.,2,0).is_none());
  assert!(armor_pin(0.,0,5).is_none());assert!(armor_pin(f32::NAN,0,0).is_none());
 }
 #[test]fn visible_rows_are_bounded_by_the_list(){
  assert_eq!(visible_rows(0.,87),0..8);assert_eq!(visible_rows(84.,87),84..87);
  assert_eq!(visible_rows(-1.,87),0..0);assert_eq!(visible_rows(0.,3),0..3);
 }
 #[test]fn weapon_pin_follows_the_great_sword_screenshot(){
  // 1920x1080: KatSmithy's view is origin 0, scale 1.5. Node centers measured:
  // Petrified Blade 1 (line col 0, row 0) at (256,133), Halberd 1 (col 4, row 9)
  // at (531,341), Iron Sword 1 (col 0, row 13) at (256,432). Icons are 27 px.
  let u=tree_scale(16);assert!((u-0.85).abs()<1e-6);
  for (col,row,cx,cy) in [(0i16,0i16,256.,133.),(4,9,531.,341.),(0,13,256.,432.)]{
   let (x,y)=weapon_pin(col,row,1,16,0.,0.).unwrap();
   let (cx,cy)=(cx/1.5,cy/1.5);let half=16.*u;
   assert!((x+5.-(cx+half)).abs()<1.5&&(y+1.-(cy-half)).abs()<1.5,"{col},{row}: {x},{y}");
  }
  // Higher levels move right by one cell each; scrolling moves the whole tree.
  let (x1,_)=weapon_pin(0,0,1,16,0.,0.).unwrap();let (x5,_)=weapon_pin(0,0,5,16,0.,0.).unwrap();
  assert!((x5-x1-4.*54.*u).abs()<1e-3);
  let (_,y)=weapon_pin(0,13,1,16,0.,0.).unwrap();let (_,ys)=weapon_pin(0,13,1,16,0.,100.).unwrap();
  assert!((y-ys-100.).abs()<1e-3);
  // Outside the panel (scrolled away, or past the bottom row) there is no pin.
  assert!(weapon_pin(0,0,1,16,0.,40.).is_none());assert!(weapon_pin(0,60,1,16,0.,0.).is_none());
  assert!(weapon_pin(0,0,0,16,0.,0.).is_none());
 }
 #[test]fn tree_scale_is_clamped_like_katsmithy(){
  assert_eq!(tree_scale(16),0.85);assert_eq!(tree_scale(1),1.25);
  assert!((tree_scale(10)-536./540.).abs()<1e-6);
  assert_eq!(weapon_node_level(0),1);assert_eq!(weapon_node_level(1),1);assert_eq!(weapon_node_level(6),6);
 }
 #[test]fn transform_matches_katsmithy_at_any_aspect(){
  assert_eq!(transform(1920.,1080.),Some((0.,0.,1.5)));
  assert_eq!(transform(2560.,1080.),Some((320.,0.,1.5)));  // 21:9 pillar-boxed
  assert_eq!(transform(1920.,1200.),Some((0.,60.,1.5)));   // 16:10 letter-boxed
  assert_eq!(transform(0.,1080.),None);
 }
 fn w16(m:&mut [u8],at:usize,v:u16){m[at..at+2].copy_from_slice(&v.to_le_bytes());}
 fn w32(m:&mut [u8],at:usize,v:u32){m[at..at+4].copy_from_slice(&v.to_le_bytes());}
 #[test]fn armor_record_is_found_by_content_anywhere(){
  for shift in [0x100usize,0x1844]{
   let mut m=vec![0u8;0x4000];let at=shift+2*1300;
   for i in 0..87u16{w16(&mut m,shift+2*i as usize,i+1);}
   w32(&mut m,at,87);w32(&mut m,at+ARMOR_ROW,3);w32(&mut m,at+ARMOR_PIECE,1);w32(&mut m,at+ARMOR_SELECTED,4);m[at+ARMOR_TOP..at+ARMOR_TOP+4].copy_from_slice(&1f32.to_le_bytes());
   assert_eq!(find_armor(&m),Some(at));let s=armor_at(&m,at).unwrap();
   assert_eq!((s.ids,s.count,s.row,s.piece,s.top),(shift,87,3,1,1.));assert_eq!(armor_id(&m,s,3),Some(4));
   // A selection that is not the cursor's set, or a short list, is rejected.
   w32(&mut m,at+ARMOR_SELECTED,5);assert!(armor_at(&m,at).is_none());
   w32(&mut m,at+ARMOR_SELECTED,4);w32(&mut m,at,8);assert!(armor_at(&m,at).is_none());
  }
 }
 #[test]fn found_records_stay_readable_while_the_cursor_moves(){
  let mut m=vec![0u8;0x2000];let (ids,at)=(0x100usize,0x100+2*1300);
  for i in 0..87u16{w16(&mut m,ids+2*i as usize,i+1);}
  w32(&mut m,at,87);w32(&mut m,at+ARMOR_ROW,3);w32(&mut m,at+ARMOR_PIECE,1);w32(&mut m,at+ARMOR_SELECTED,4);
  assert!(armor_at(&m,at).is_some());
  // Row moved, selected set not yet synced, first row mid-scroll: strict fails, live reads.
  w32(&mut m,at+ARMOR_ROW,4);m[at+ARMOR_TOP..at+ARMOR_TOP+4].copy_from_slice(&1.5f32.to_le_bytes());
  assert!(armor_at(&m,at).is_none());let s=armor_live(&m,at,ids).unwrap();assert_eq!((s.row,s.top),(4,1.5));
  w32(&mut m,at,0);assert!(armor_live(&m,at,ids).is_none());
 }
 #[test]fn renderer_gives_the_transform_global_and_checks_layout_constants(){
  // Synthetic ARM: the log string's PC-relative load, a reused register (first a
  // different global, then the transform read at +4/+0xC), and the literal pool.
  let build=|consts:&[f32]|{
   let mut m=vec![0u8;0x1000];let s=0x801;m[0x800]=0;m[s..s+RENDERER_LOG.len()].copy_from_slice(RENDERER_LOG);
   let put=|m:&mut Vec<u8>,at:usize,v:u32|m[at..at+4].copy_from_slice(&v.to_le_bytes());
   put(&mut m,0x100,0xe59f1010);put(&mut m,0x104,0xe08f1001);put(&mut m,0x118,(s-0x10c) as u32);
   put(&mut m,0x120,0xe59f3020);put(&mut m,0x124,0xe08f3003);put(&mut m,0x148,0x3f0-0x12c);  // other global
   put(&mut m,0x128,0xed937a00);                                                                // vldr s14,[r3]
   put(&mut m,0x12c,0xe59f3018);put(&mut m,0x130,0xe08f3003);put(&mut m,0x14c,0x3e0-0x138);  // transform
   put(&mut m,0x134,0xed937a01);put(&mut m,0x138,0xedd37a03);                                 // [r3,#4] [r3,#0xC]
   for (i,c) in consts.iter().enumerate(){put(&mut m,0x180+4*i,0xed9f0a1e);put(&mut m,0x200+4*i,c.to_bits());}
   m
  };
  let m=build(&TREE_CONSTANTS);
  assert_eq!(renderer(&m,0x400),Some(Renderer{at:0x104,view:Some(0x3e0),constants:true}));
  let m=build(&TREE_CONSTANTS[..6]);assert_eq!(renderer(&m,0x400).map(|r|r.constants),Some(false));
  assert_eq!(renderer(&[0u8;64],32),None);
 }
 #[test]fn tree_record_is_found_by_content_anywhere_in_either_layout(){
  for layout in [0u8,1]{let l=&TREE_LAYOUTS[layout as usize];
  for at in [0x40usize,0x1000]{
   let mut m=vec![0u8;at+l.end+0x40];
   w32(&mut m,at+l.columns,16);w32(&mut m,at+l.rows,4);
   for (i,(pos,line,col)) in [(0u16,1u16,0u16),(3,6,3),(6,7,3),(9,3,4)].into_iter().enumerate(){
    w16(&mut m,at+l.row_pos+2*i,pos);w16(&mut m,at+l.row_height+2*i,1);w16(&mut m,at+l.row_line+2*i,line);
    w16(&mut m,at+l.line_row+2*line as usize,pos);w16(&mut m,at+l.line_col+2*line as usize,col);
   }
   let found=find_tree(&m).unwrap();assert_eq!((found&!3,found&3),(at,layout as usize));
   let t=tree_live(&m,found).unwrap();assert_eq!(t,tree_at(&m,at,layout).unwrap());
   assert_eq!(tree_line(&m,t,3),Some((3,4,9)));
   // A line whose own row disagrees with the row table is not a tree.
   w16(&mut m,at+l.line_row+2*3,10);assert!(tree_at(&m,at,layout).is_none());
  }}
 }
 #[test]fn hidden_nodes_get_no_pin_on_layouts_with_a_hit_list(){
  let l=&TREE_LAYOUTS[1];let (list,count)=l.hits.unwrap();
  let mut m=vec![0u8;l.end+0x40];
  // Two tab buttons, then node (line 3, level 1).
  w32(&mut m,list,2);w32(&mut m,list+TREE_HIT_SIZE,1<<16|2);w32(&mut m,list+2*TREE_HIT_SIZE,3<<16|1);w32(&mut m,list+2*TREE_HIT_SIZE+4,1);
  // A stale entry past the count is not drawn.
  w32(&mut m,list+3*TREE_HIT_SIZE,1<<16|1);w32(&mut m,list+3*TREE_HIT_SIZE+4,5);
  w32(&mut m,count,3);
  let t=TreeState{at:0,columns:4,rows:1,layout:1};
  assert!(tree_node_drawn(&m,t,3,1));assert!(!tree_node_drawn(&m,t,3,2));assert!(!tree_node_drawn(&m,t,1,5));
  assert!(tree_node_drawn(&m,TreeState{layout:0,..t},1,5));
 }
 #[test]fn katsmithy_150_layout_follows_its_doubled_tables(){
  let l=&TREE_LAYOUTS[1];
  assert_eq!(l.line_row,l.line_col+2*l.lines as usize);assert_eq!(l.rows,l.line_row+2*l.lines as usize);
  assert_eq!(l.row_pos,l.rows+4);assert_eq!(l.row_height,l.row_pos+2*l.max_rows as usize);assert_eq!(l.row_line,l.row_height+2*l.max_rows as usize);
  // The fields after the tables keep their order and spacing from 1.4, 0x14 further from the selection.
  let o=&TREE_LAYOUTS[0];
  for (new,old) in [(l.mode,o.mode),(l.scroll_y,o.scroll_y),(l.scroll_x,o.scroll_x)]{assert_eq!(new-l.selected_id,old-o.selected_id+0x14);}
 }
 #[test]fn stick_is_zeroed_while_owned_and_until_it_recenters(){
  let mut latch=false;
  assert!(stick_blocked(0.,1.,true,&mut latch));
  assert!(stick_blocked(0.,0.9,false,&mut latch));
  assert!(!stick_blocked(0.,0.1,false,&mut latch));
  assert!(!stick_blocked(0.,0.9,false,&mut latch));
 }
}

/// A click consumed by the overlay cannot reach the tree on the closing frame
/// or while the same physical mouse button remains held.
pub fn mouse_buttons(raw:u32,owned:bool,latch:&mut u32)->u32{
 *latch&=raw;
 if owned{*latch|=raw;0}else{raw&!*latch}
}
#[cfg(test)]mod mouse_tests{
 use super::*;
 #[test]fn closing_right_click_stays_consumed_until_release(){
  let mut latch=0;
  assert_eq!(mouse_buttons(4,true,&mut latch),0);
  assert_eq!(mouse_buttons(4,false,&mut latch),0);
  assert_eq!(mouse_buttons(4,false,&mut latch),0);
  assert_eq!(mouse_buttons(0,false,&mut latch),0);
  assert_eq!(mouse_buttons(4,false,&mut latch),4);
 }
 #[test]fn unrelated_new_click_passes_after_ownership_ends(){
  let mut latch=0;mouse_buttons(4,true,&mut latch);
  assert_eq!(mouse_buttons(5,false,&mut latch),1);
  assert_eq!(mouse_buttons(0,false,&mut latch),0);
 }
}
