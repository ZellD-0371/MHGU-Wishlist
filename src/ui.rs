#[cfg(feature="keyboard-input")]
use forge::input::{Input,Key};
use alloc::{vec::Vec,format,string::String,ffi::CString};
use forge::{imgui,sys::imgui::{ImVec2,ImDrawFlags,ImGuiComboFlags,ImGuiSelectableFlags}};
use crate::{STATE,State,model::{Recipe,List},locales as l,generated as g,controller as c,storage,sound::{self,Cue}};
pub(crate) static mut FONT:Option<imgui::ImFont>=None;
// MHXX in-game palette, shared with Skill-Simulator (ImGui ABGR).
const WHITE:u32=0xffe9eceb;const GOLD:u32=0xff6ce6f8;const BLUE:u32=0xffe8c586;const RED:u32=0xff7a8af0;const GREEN:u32=0xff9ad49f;const MUTED:u32=0xffa6a398;
const PANEL:u32=0xff26221b;const INSET:u32=0xff1c1a16;const RAISED:u32=0xff2d281f;const RULE:u32=0xff4c4636;
const SELECT:u32=0xff403a2c;const BLACK:u32=0xff000000;const TEAL:[u32;4]=[0xff867f2b;4];
// Window frame: red double lines over a black band, a green-gold line below.
const RED_TONES:[u32;4]=[0xff35368a,0xff2729a9,0xff2022b6,0xff1c2ea5];const RED_UNDER:[u32;3]=[0xff217485,0xff148964,0xff258a59];
// The game's window_sub corner ornament: (x, y, width, gold) one-unit rows.
const CORNER:&[(u8,u8,u8,bool)]=&[(2,0,10,false),(2,1,10,false),(0,2,2,false),(2,2,10,true),(12,2,2,false),(0,3,2,false),(2,3,10,true),(12,3,2,false),(0,4,2,false),(2,4,2,true),(4,4,10,false),(0,5,2,false),(2,5,2,true),(4,5,10,false),(0,6,2,false),(2,6,2,true),(4,6,2,false),(6,6,6,true),(12,6,2,false),(0,7,2,false),(2,7,2,true),(4,7,2,false),(6,7,6,true),(12,7,2,false),(0,8,2,false),(2,8,2,true),(4,8,6,false),(10,8,2,true),(12,8,2,false),(0,9,2,false),(2,9,2,true),(4,9,6,false),(10,9,2,true),(12,9,2,false),(0,10,2,false),(2,10,10,true),(12,10,2,false),(0,11,2,false),(2,11,10,true),(12,11,2,false),(2,12,10,false),(2,13,10,false)];
// Noto's em includes substantial vertical space: 25 em pixels gives ~18px visible copy.
const BODY:f32=25.;
// Existing HD atlas art: equipment list, material container, decoration.
const SORT_ORDER_ICON:usize=36;const SORT_STOCK_ICON:usize=23;const SORT_DECORATION_ICON:usize=44;
pub(crate) fn push_binding_font()->bool{if let Some(f)=unsafe{FONT}{imgui::push_font(&f,Some(18.));true}else{false}}
pub fn init(){crate::text_cache::clear();crate::language_ui::init();}
fn table_name(table:u8,index:usize,lang:usize)->String{crate::name_data::get(table,index,lang).into()}
pub fn name(r:Recipe,lang:usize)->String{match r.kind{
 1..=5=>table_name(2,r.id as usize*10+r.kind as usize-1,lang),
 7=>table_name(5,r.id as usize*6,lang),8=>table_name(6,r.id as usize*6,lang),
 9=>table_name(7,r.id as usize*6,lang),10=>table_name(8,r.id as usize*6,lang),11=>table_name(9,r.id as usize*6,lang),
 13=>table_name(10,r.id as usize*6,lang),14=>table_name(11,r.id as usize*6,lang),15=>table_name(12,r.id as usize*6,lang),
 16=>table_name(13,r.id as usize*6,lang),17=>table_name(14,r.id as usize*6,lang),18=>table_name(15,r.id as usize*6,lang),
 19=>table_name(16,r.id as usize*6,lang),20=>table_name(17,r.id as usize*6,lang),21=>table_name(18,r.id as usize*6,lang),
 22=>table_name(3,r.id as usize*2,lang),23|24=>table_name(4,r.id as usize*4+(r.kind-23) as usize,lang),
 25=>table_name(1,r.id as usize*2,lang),_=>"?".into()}}
fn item(id:u16,lang:usize)->String{table_name(1,id as usize*2,lang)}
struct Canvas{scale:f32,ox:f32,oy:f32,draw:imgui::DrawList}
impl Canvas{
 fn new()->Option<Self>{let draw=imgui::DrawList::foreground();if draw.raw().is_null(){return None}let sz=imgui::get_io().DisplaySize;let (percent,base)=STATE.with(|s|(s.list.ui_scale,s.base));let view=crate::hud_layout::picture(sz.x,sz.y,percent,crate::hud_layout::current(base))?;Some(Self{scale:view.scale,ox:view.ox,oy:view.oy,draw})}
 fn pixels(&self,size:f32)->f32{((size*self.scale+0.5) as u32).max(1) as f32}
 fn p(&self,x:f32,y:f32)->ImVec2{ImVec2{x:self.ox+x*self.scale,y:self.oy+y*self.scale}}
 fn rect(&self,x:f32,y:f32,w:f32,h:f32,color:u32){self.draw.add_rect_filled(self.p(x,y),self.p(x+w,y+h),color,0.);}
 fn line(&self,x:f32,y:f32,x2:f32,y2:f32,color:u32){self.draw.add_line(self.p(x,y),self.p(x2,y2),color,self.scale);}
 /// The XX window border: colored double lines, black side bands, gold corners.
 fn border(&self,x:f32,y:f32,w:f32,h:f32,tones:[u32;4],under:&[u32],opacity:f32){
  let a=|c:u32|crate::presentation::alpha(c,opacity);let (x1,y1)=(x+w,y+h);
  self.rect(x,y+8.,3.,h-16.,a(BLACK));self.rect(x1-3.,y+8.,3.,h-16.,a(BLACK));
  for (i,&c) in tones.iter().enumerate(){let yy=2.+i as f32;self.rect(x+8.,y+yy,w-16.,1.,a(c));self.rect(x+8.,y1-yy-1.,w-16.,1.,a(c));}
  self.rect(x+8.,y+1.,w-16.,1.,a(BLACK));self.rect(x+8.,y1-2.,w-16.,1.,a(BLACK));
  for (i,&c) in under.iter().enumerate(){let yy=6.+i as f32;self.rect(x+4.,y+yy,w-8.,1.,a(c));self.rect(x+4.,y1-yy-1.,w-8.,1.,a(c));}
  for flip_y in [false,true]{for flip_x in [false,true]{for &(rx,ry,rw,gold) in CORNER{
   let (rx,ry,rw)=(rx as f32,ry as f32,rw as f32);
   let px=if flip_x{x1-rx-rw}else{x+rx};let py=if flip_y{y1-ry-1.}else{y+ry};
   self.rect(px,py,rw,1.,a(if gold{GOLD}else{BLACK}));
  }}}
 }
 /// A main window: dark panel in the red XX frame.
 fn frame(&self,x:f32,y:f32,w:f32,h:f32){self.rect(x,y,w,h,PANEL);self.border(x,y,w,h,RED_TONES,&RED_UNDER,1.);}
 /// A dialog or message box: raised panel in a teal frame.
 fn popup(&self,x:f32,y:f32,w:f32,h:f32){self.rect(x,y,w,h,RAISED);self.border(x,y,w,h,TEAL,&[],1.);}
 /// The selection cursor: a lit row with gold corner brackets.
 fn cursor(&self,x:f32,y:f32,w:f32,h:f32){
  self.rect(x,y,w,h,SELECT);
  for (cx,sx) in [(x,1.),(x+w,-1.)]{for (cy,sy) in [(y,1.),(y+h,-1.)]{
   self.draw.add_line(self.p(cx,cy),self.p(cx+9.*sx,cy),GOLD,2.*self.scale);
   self.draw.add_line(self.p(cx,cy),self.p(cx,cy+9.*sy),GOLD,2.*self.scale);
  }}
 }
 /// A dialog button: the cursor when focused, an outline otherwise.
 fn button(&self,x:f32,y:f32,w:f32,h:f32,focused:bool,label:&str){
  if focused{self.cursor(x,y,w,h);}else{self.draw.add_rect(self.p(x,y),self.p(x+w,y+h),RULE,0.,self.scale,ImDrawFlags::None);}
  self.box_text(x,y,w,h,26.,if focused{WHITE}else{MUTED},label,true);
 }
 fn heading(&self,x:f32,y:f32,w:f32,text:&str){
  self.box_text(x+4.,y,w-8.,38.,28.,GOLD,text,false);self.rect(x,y+39.,w,1.,RULE);
 }
 fn section(&self,x:f32,y:f32,w:f32,text:&str){
  let text_w=self.measure(22.,text).min(w-8.);self.fit(x,y,w,22.,GOLD,text);
  if text_w+18.<w{self.line(x+text_w+12.,y+18.,x+w,y+18.,RULE);}
 }
 fn selection(&self,x:f32,y:f32,w:f32,h:f32,selected:bool,alternate:bool){
  if selected{self.cursor(x,y,w,h);}
  else{self.rect(x,y,w,h,if alternate{0xff2a251e}else{0xff2e2921});}
 }
 fn icon_plate(&self,x:f32,y:f32,size:f32){self.rect(x,y,size,size,INSET);self.draw.add_rect(self.p(x,y),self.p(x+size,y+size),RULE,1.,self.scale,ImDrawFlags::None);}
 fn text(&self,x:f32,y:f32,width:f32,size:f32,color:u32,text:&str){
  let d=self.draw;d.push_clip_rect(self.p(x,y),self.p(x+width,y+size*1.8),true);
  if let Some(font)=unsafe{FONT}{d.add_text_with_font(&font,self.pixels(size),self.p(x,y),color,text)}else{d.add_text(self.p(x,y),color,text)}d.pop_clip_rect();
 }
 fn fit(&self,x:f32,y:f32,w:f32,size:f32,color:u32,text:&str){
  self.box_text(x,y,w,size,size,color,text,false);
 }
 fn center(&self,x:f32,y:f32,w:f32,size:f32,color:u32,text:&str){
  self.box_text(x,y,w,size,size,color,text,true);
 }
 fn box_text(&self,x:f32,y:f32,w:f32,h:f32,size:f32,color:u32,text:&str,center:bool){
  let width=self.measure(size,text);let pixels=((self.pixels(size)*(w/width.max(w))+0.0001) as u32).max(1) as f32;let fitted=pixels/self.scale;
  let metrics=self.metrics(fitted,text);let fitted_width=(metrics.width/self.scale).min(w);
  let [left,right,top,bottom]=metrics.ink.map(|b|b.map(|n|n/self.scale)).unwrap_or([0.,fitted_width,0.,pixels/self.scale]);
  let tx=if center{x+w*0.5-(left+right)*0.5}else{x};let ty=y+h*0.5-(top+bottom)*0.5;
  self.draw.push_clip_rect(self.p(x,y),self.p(x+w,y+h),true);
  if let Some(font)=unsafe{FONT}{self.draw.add_text_with_font(&font,pixels,self.p(tx,ty),color,text)}else{self.draw.add_text(self.p(tx,ty),color,text)}
  self.draw.pop_clip_rect();
 }
 fn raw_icon(&self,tile:usize,x:f32,y:f32,size:f32,tint:u32){let p=self.p(x,y);crate::icons::draw(self.draw,tile,p.x,p.y,self.pixels(size),self.pixels(size),tint,false);}
 fn icon(&self,mut tile:usize,x:f32,y:f32,size:f32,tint:u32){
  // Each sprite is tied to its *menu action*, not a numeric array position.
  // The final six menu actions have no native artwork except L3/R3.
  let icon_actions=[
   (crate::icons::A,0), (crate::icons::B,1),
   (crate::icons::X,2), (crate::icons::Y,3),
   (crate::icons::L,4), (crate::icons::R,5),
   (crate::icons::ZL,6), (crate::icons::ZR,7),
   (crate::icons::DPAD_LEFT,8), (crate::icons::DPAD,9),
   (crate::icons::L3,14), (crate::icons::R3,15),
  ];
  // + is read from the pad bit (not a menu action): only the keyboard prompt changes.
  if tile==crate::button_prompts::PAD_EXTRA_TILE+2&&crate::bindings::keyboard_prompts()&&let Some(key)=crate::button_prompts::key_prompt(12){
   match crate::button_prompts::recomp_key_name(&key).and_then(key_part){
    Some(Part::Tile(t))=>self.raw_icon(t,x,y,size,tint),
    Some(Part::Key(label))=>{self.keycap(label,x,y,size,Some(size),true);}
    _=>{self.rect(x,y,size,size,RAISED);self.box_text(x,y,size,size,size*0.5,WHITE,&crate::button_prompts::short_key(&key),true);}
   }
   return
  }
  if let Some((_,action))=icon_actions.iter().find(|(id,_)|*id==tile){
   // A keyboard player sees the key of the menu action (Recomp only).
   if crate::bindings::keyboard_prompts()&&let Some(key)=crate::button_prompts::key_prompt(*action){
    match crate::button_prompts::recomp_key_name(&key).and_then(key_part){
     Some(Part::Tile(t))=>self.raw_icon(t,x,y,size,tint),
     Some(Part::Key(label))=>{self.keycap(label,x,y,size,Some(size),true);}
     _=>{self.rect(x,y,size,size,RAISED);self.box_text(x,y,size,size,size*0.5,WHITE,&crate::button_prompts::short_key(&key),true);}
    }
    return
   }
   match crate::button_prompts::get(*action){
    crate::button_prompts::Prompt::Original=>{},
    crate::button_prompts::Prompt::Position(p)=>tile=crate::button_prompts::icon_slot(p,0),
    crate::button_prompts::Prompt::Text(text)=>{
     self.rect(x,y,size,size,RAISED);
     self.box_text(x,y,size,size,size*0.65,WHITE,&text,true);
     return
    }
   }
  }
  self.raw_icon(tile,x,y,size,tint);
 }

 fn gear(&self,r:Recipe,x:f32,y:f32,size:f32){self.gear_alpha(r,x,y,size,1.);}
 fn gear_alpha(&self,r:Recipe,x:f32,y:f32,size:f32,alpha:f32){if r.kind==25{let (tile,tint)=crate::icons::material(r.id);self.icon(tile,x,y,size,crate::presentation::alpha(tint,alpha));}else{self.icon(crate::icons::equipment(r.kind),x,y,size,crate::presentation::alpha(crate::equipment::icon_tint(r),alpha));}}
 fn sprite(&self,tile:usize,x:f32,y:f32,w:f32,h:f32,tint:u32,flip:bool){let p=self.p(x,y);crate::icons::draw(self.draw,tile,p.x,p.y,w*self.scale,h*self.scale,tint,flip);}
 fn shortcut_bar(&self,x:f32,y:f32,chord:u32,key:u32,label:&str){
  self.bar(x,y,160.,usize::MAX,"");
  let parts=if crate::bindings::keyboard_prompts()&&key&255!=0{key_parts(key)}else{chord_parts(chord)};
  // Up to 90 units of prompts; more parts shrink together.
  let natural=self.parts(&parts,0.,0.,18.,false);let size=18.*(90./natural.max(1.)).min(1.);
  let width=self.parts(&parts,x+8.,y+(28.-size)*0.5,size,true);
  self.box_text(x+width+10.,y,142.-width,28.,24.,WHITE,label,false);
 }
 /// The window's button hints as the game's own prompt bars (assign.gui:
 /// black centre, end caps), each as wide as its label, right-aligned left of
 /// the detail panel's right edge, on one row (scaled down only when it
 /// would reach the game's help bar on the left).
 fn hints(&self,items:&[(usize,&str)]){
  // Fixed order, left to right: other buttons, then − +, X Y, A B.
  let extra=crate::button_prompts::PAD_EXTRA_TILE;
  let rank=|tile:usize|match tile{t if t==extra+3=>1,t if t==extra+2=>2,crate::icons::X=>3,crate::icons::Y=>4,crate::icons::A=>5,crate::icons::B=>6,_=>0};
  let mut items=items.to_vec();items.sort_by_key(|(tile,_)|rank(*tile));let items=&items[..];
  // One long bar with every hint inside it.
  let (gap,pad,right,left,y)=(22.,16.,1262.,612.,662.);
  let widths:Vec<f32>=items.iter().map(|(_,label)|18.+5.+self.measure(24.,label)).collect();
  let inner:f32=widths.iter().sum::<f32>()+gap*(items.len().saturating_sub(1)) as f32;
  let k=((right-left-pad*2.)/inner.max(1.)).min(1.);
  let w=inner*k+pad*2.;let x0=right-w;
  self.bar(x0,y,w,usize::MAX,"");
  let mut x=x0+pad;
  for (i,(tile,label)) in items.iter().enumerate(){
   let icon=18.*k.max(0.8);self.icon(*tile,x,y+(28.-icon)*0.5,icon,0xffecebe7);
   self.box_text(x+icon+5.*k,y,(widths[i]-23.)*k+2.,28.,24.*k.max(0.8),0xffecebe7,label,false);
   x+=(widths[i]+gap)*k;
  }
 }
 fn bar(&self,x:f32,y:f32,w:f32,button:usize,label:&str){
  // Native assign.gui: 160x28, 156px black centre, 12px HD lobby end caps.
  // Preserve the native black backing; omit only the old exterior shadow rectangle.
  self.rect(x+2.,y,w-4.,28.,0xff000000);
  self.sprite(crate::icons::PROMPT_END,x,y,12.,28.,0xffffffff,false);
  self.sprite(crate::icons::PROMPT_END,x+w-12.,y,12.,28.,0xffffffff,true);
  if button!=usize::MAX{self.icon(button,x+8.,y+5.,18.,0xffecebe7);}self.box_text(x+28.,y,w-36.,28.,24.,0xffecebe7,label,false);
 }
 fn measure(&self,size:f32,text:&str)->f32{
  // PushFont/CalcTextSize applies Recomp's window/DPI scale a second time.
  // DrawList text takes an explicit pixel size, so use that same baked font here.
  self.metrics(size,text).width/self.scale
 }
 fn metrics(&self,size:f32,text:&str)->crate::text_cache::Metrics{
  let pixels=self.pixels(size);let font=unsafe{FONT};let key=crate::text_cache::MetricKey{font:font.map_or(0,|f|f.raw() as usize),pixels:pixels as u32};
  crate::text_cache::metrics(key,text,||{
   let mut line=0_f32;let mut width=0_f32;let mut bounds=[f32::MAX,f32::MIN,f32::MAX,f32::MIN];let mut pen=0.;
   if let Some(font)=font{unsafe{
    let baked=forge::sys::imgui::ImFont_GetFontBaked(font.raw(),pixels,1.);
    if !baked.is_null(){for ch in text.chars(){
     if ch=='\n'{width=width.max(line);line=0.;}
     let glyph=forge::sys::imgui::ImFontBaked_FindGlyph(baked,u16::try_from(ch as u32).unwrap_or(0xfffd));
     if !glyph.is_null(){let g=&*glyph;if g.Y1>g.Y0{bounds[0]=bounds[0].min(pen+g.X0);bounds[1]=bounds[1].max(pen+g.X1);bounds[2]=bounds[2].min(g.Y0);bounds[3]=bounds[3].max(g.Y1);}pen+=g.AdvanceX;if ch!='\n'&&ch!='\r'{line+=g.AdvanceX;}}
    }let ink=(bounds.iter().all(|v|v.is_finite())&&bounds[0]<=bounds[1]&&bounds[2]<=bounds[3]).then_some(bounds);return crate::text_cache::Metrics{width:width.max(line),ink}}
   }}crate::text_cache::Metrics{width:pixels*text.chars().count() as f32*0.5,ink:None}
  })
 }
 fn box_bold(&self,x:f32,y:f32,w:f32,h:f32,size:f32,color:u32,text:&str,center:bool){
  let offset=0.5/self.scale;self.box_text(x-offset,y,w,h,size,color,text,center);self.box_text(x+offset,y,w,h,size,color,text,center);
 }
 fn pin(&self,x:f32,y:f32){
  // A small enamel pushpin, drawn at native HUD scale rather than the user's list scale.
  self.draw.add_line(self.p(x-5.,y+12.),self.p(x+3.,y+4.),0xff18130b,4.*self.scale);
  self.draw.add_line(self.p(x-5.,y+12.),self.p(x+3.,y+4.),0xffe9edf0,1.5*self.scale);
  self.draw.add_circle_filled(self.p(x+3.,y+3.),5.5*self.scale,0xff18130b,12);
  self.draw.add_circle_filled(self.p(x+3.,y+3.),4.*self.scale,0xff39bbf3,12);
  self.draw.add_circle_filled(self.p(x+2.,y+2.),1.2*self.scale,0xffb1ecff,8);
 }
 /// A mouse target for the next frame, in this canvas's units.
 fn hit(&self,x:f32,y:f32,w:f32,h:f32,hit:Hit){
  if outlined(hit)&&unsafe{*core::ptr::addr_of!(HOVER)}==Some(hit){self.draw.add_rect(self.p(x,y),self.p(x+w,y+h),HOVER_GOLD,6.*self.scale,2.*self.scale,ImDrawFlags::None);}
  unsafe{(*core::ptr::addr_of_mut!(HITS)).push((x,y,w,h,hit));}
 }
 fn pager(&self,cx:f32,y:f32,page:usize,pages:usize){
  let text=format!("◀ {} / {} ▶",page,pages.max(1));let width=self.measure(24.,&text)+4.;let left=cx-width*0.5;let icon=25.;let gap=16.;
  self.hit(left-gap-icon-4.,y-4.,icon+8.,icon+8.,Hit::Event(c::PREV));self.hit(left+width+gap-4.,y-4.,icon+8.,icon+8.,Hit::Event(c::NEXT));
  self.box_text(left,y,width,icon,24.,WHITE,&text,true);
  self.icon(crate::icons::DPAD_LEFT,left-gap-icon,y,icon,WHITE);
  self.icon(crate::icons::DPAD,left+width+gap,y,icon,WHITE);
 }
 fn slots(&self,x:f32,y:f32,count:i16){for i in 0..3{let px=x+i as f32*25.;if i<count.clamp(0,3){self.sprite(crate::icons::SLOT_RING,px,y,20.,20.,WHITE,false);}else{self.sprite(crate::icons::SLOT_DASH,px,y+8.,20.,6.,WHITE,false);}}}
 fn sharpness(&self,x:f32,y:f32,w:f32,sharp:[u16;7]){
  // Native menu_equipDetail's three slices: 20px hilt, stretch body, 20px tip.
  self.sprite(crate::icons::SHARPNESS_FRAME,x,y,20.,20.,0xff787878,false);
  self.sprite(crate::icons::SHARPNESS_MIDDLE,x+20.,y,w-40.,20.,0xff787878,false);
  self.sprite(crate::icons::SHARPNESS_TIP,x+w-20.,y,20.,20.,0xff787878,false);
  let left=x+21.;let top=y+6.;let usable=w-42.;let mut at=left;
  let colors=[0xff0000ff,0xff0092ff,0xff00ffff,0xff00d200,0xffff0000,0xffffffff,0xffff0064];
  self.rect(left,top,usable,8.,0xff000000);
  for (i,n) in sharp.iter().enumerate(){let width=usable*(*n as f32/450.);if width>0.{self.rect(at,top,width.min((left+usable-at).max(0.)),8.,colors[i]);at+=width;}}
 }
 fn paragraph(&self,x:f32,y:f32,w:f32,h:f32,size:f32,color:u32,text:&str){
  let key=crate::text_cache::LayoutKey{font:unsafe{FONT}.map_or(0,|f|f.raw() as usize),scale:self.scale.to_bits(),width:w.to_bits(),height:h.to_bits(),size:size.to_bits()};
  crate::text_cache::paragraph(key,text,||{
  let layout=|size:f32|{let mut lines=Vec::new();let mut line=String::new();
   for paragraph in text.split('\n'){for word in paragraph.split_whitespace(){
    let candidate=if line.is_empty(){String::from(word)}else{format!("{} {}",line,word)};
    if self.measure(size,&candidate)<=w{line=candidate;continue}
    if !line.is_empty(){lines.push(core::mem::take(&mut line));}
    for ch in word.chars(){line.push(ch);if self.measure(size,&line)>w{line.pop();if !line.is_empty(){lines.push(core::mem::take(&mut line));}line.push(ch);}}
   }if !line.is_empty(){lines.push(core::mem::take(&mut line));}}lines};
  let mut size=size;let mut lines=layout(size);
  while lines.len() as f32*size*1.3>h&&size>15.{size-=1.;lines=layout(size);}
  crate::text_cache::Layout{size,lines}
  },|layout|{
  self.draw.push_clip_rect(self.p(x,y),self.p(x+w,y+h),true);
  for (i,line) in layout.lines.iter().enumerate(){self.text(x,y+i as f32*layout.size*1.3,w,layout.size,color,line);}
  self.draw.pop_clip_rect();
  });
 }
 fn wrap(&self,x:f32,y:f32,width:f32,size:f32,color:u32,text:&str)->f32{
  let mut line=String::new();let mut py=y;
  // Measure with the explicit draw size rather than the host window font scale.
  for word in text.chars(){if word=='\n'{self.text(x,py,width,size,color,&line);line.clear();py+=size*1.45;continue}
   line.push(word);if self.measure(size,&line)>width{line.pop();self.text(x,py,width,size,color,&line);line.clear();line.push(word);py+=size*1.45;}
  }if !line.is_empty(){self.text(x,py,width,size,color,&line);py+=size*1.45}
  py
 }
}
fn save(next:List)->bool{
 let profile=STATE.with(|s|s.profile);
 let result=if let Some(slot)=profile{storage::character(slot).save(&next.encode_character())}else{return false};
 match result{Ok(())=>{STATE.with(|s|{s.list=next;s.notices.retain_registered(&s.list);});true},Err(_)=>{STATE.with(|s|s.status=3);false}}
}
pub(crate) fn apply_preferences(next:&List){crate::language_ui::request(next);}

fn save_preferences()->bool{
 let next=STATE.with(|s|s.list.clone());
 match storage::save_settings(&next){
  Ok(())=>{STATE.with(|s|s.settings_notice=2);true},
  Err(_)=>{STATE.with(|s|s.settings_notice=3);false}
 }
}
fn commit(register:bool){
 let change=STATE.with(|s|{let mut next=s.list.clone();let selected;
  if register{let Some(r)=s.frozen else{return None};let Ok(slot)=next.register(s.selected,r,s.frozen_cost)else{s.status=2;return None};selected=slot;if s.stock_ready{if let Some(e)=next.slots[slot].as_mut(){e.ready=Some(r.ready(&s.stock,g::PROVISIONS));}}}
  else{next.remove_slot(s.selected);selected=s.selected;}Some((next,selected))});
 if let Some((next,selected))=change{if save(next){STATE.with(|s|{s.selected=selected;s.status=1;s.mode=if register{5}else if s.mode==8{0}else{2};s.preview=false;});sound::cue(if register{Cue::Registered}else{Cue::Inspect});}}
}
/// Empty the whole wishlist (after its confirmation).
fn clear_all(){
 let next=STATE.with(|s|{let mut next=s.list.clone();for i in 0..crate::model::CAPACITY{next.remove_slot(i);}next});
 if save(next){STATE.with(|s|{s.mode=2;s.selected=0;s.status=0;s.focus=crate::interaction::Focus::default();s.detail_page=0;});sound::cue(Cue::Inspect);}
 else{STATE.with(|s|s.mode=2);}
}
fn apply_sort(){
 let next=STATE.with(|s|{let dialog=s.sort.as_mut()?;let mut next=s.list.clone();if next.sort_display(dialog.criterion,dialog.category,&s.stock,s.stock_ready,g::PROVISIONS).is_err(){dialog.message=1;return None}Some(next)});
 if let Some(next)=next{if save(next){STATE.with(|s|{s.sort=None;s.selected=0;s.focus=crate::interaction::Focus::default();s.detail_page=0;s.status=0;});sound::cue(Cue::Open);}else{STATE.with(|s|{if let Some(d)=s.sort.as_mut(){d.message=2;}});}}
}
fn events(events:u32){
 if events&c::OPEN_LIST!=0{STATE.with(|s|s.open(false));return}
 if events&c::OPEN_REGISTER!=0{STATE.with(|s|s.open(true));return}
 let action=STATE.with(|s|{
  if s.mode==0{return 0}
  if let Some(d)=s.sort.as_mut(){
   let was_secondary=d.secondary;let direction=if events&c::UP!=0{-1}else if events&c::DOWN!=0{1}else{0};
   let result=d.input(direction,events&c::ACCEPT!=0,events&c::BACK!=0);
   if events&c::BACK!=0{sound::cue(Cue::Close);}else if direction!=0{sound::cue(Cue::Cursor);}else if !was_secondary&&d.secondary{sound::cue(Cue::Inspect);}
   match result{crate::sorting::Action::Cancel=>s.sort=None,crate::sorting::Action::Apply=>return 3,_=>{}}return 0
  }
  if s.mode==2&&s.tab==0&&!s.focus.material&&!s.preview&&events&c::CLEAR!=0{if s.list.slots.iter().any(Option::is_some){s.mode=9;s.focus.yes=false;sound::cue(Cue::Inspect);}return 0}
  if s.mode==2&&s.tab==0&&!s.focus.material&&!s.preview&&events&c::SORT!=0{if s.list.slots.iter().any(Option::is_some){s.sort=Some(crate::sorting::Dialog::default());sound::cue(Cue::Inspect);}return 0}
  if s.mode==7{
   if events&c::BACK!=0{s.mode=0;s.notice_demo=None;sound::cue(Cue::Close);}
   else if let Some(demo)=s.notice_demo.as_mut(){
    if events&c::ACCEPT!=0{demo.restart();sound::cue(Cue::Inspect);}
    if events&(c::PREV|c::NEXT)!=0{demo.scenario=(demo.scenario+if events&c::NEXT!=0{1}else{2})%3;s.notice_test_case=demo.scenario;demo.restart();sound::cue(Cue::Page);}
   }return 0
  }
  if s.preview{
   if events&c::BACK!=0{s.mode=0;s.preview=false;sound::cue(Cue::Close);return 0}
   if s.mode==2&&events&c::TAB!=0{s.tab=1-s.tab;sound::cue(Cue::Tab);}
   if s.tab==0&&selected_recipe(s).is_some()&&events&c::DETAIL!=0{s.detail_page=1-s.detail_page;sound::cue(Cue::Tab);}
   if events&(c::PREV|c::NEXT|c::TAB_PREV|c::TAB_NEXT)!=0{
    s.preview_panel=(s.preview_panel+if events&(c::NEXT|c::TAB_NEXT)!=0{1}else{7})%8;s.mode=match s.preview_panel{0=>2,1|2=>1,3=>5,4=>6,5=>3,6=>4,_=>2};s.focus.material=s.preview_panel==7;s.focus.index=0;s.preview_clock=0.;
    sound::cue(if events&(c::TAB_PREV|c::TAB_NEXT)!=0{Cue::Tab}else{Cue::Page});
   }return 0
  }
  if s.mode==5{if events&(c::ACCEPT|c::BACK)!=0{s.mode=0;sound::cue(Cue::Close);}return 0}
  if events&c::BACK!=0{
   sound::cue(Cue::Close);if s.focus.back(){return 0}
   match s.mode{3=>s.mode=1,4|9=>s.mode=2,_=>{s.mode=0;s.preview=false}}return 0
  }
  if matches!(s.mode,3|4|8|9){
   if events&(c::PREV|c::NEXT|c::UP|c::DOWN)!=0{s.focus.yes=!s.focus.yes;sound::cue(Cue::Cursor);}
   if events&c::ACCEPT!=0{if s.focus.yes{return match s.mode{3=>1,9=>5,_=>2}}s.mode=match s.mode{3=>1,8=>0,_=>2};sound::cue(Cue::Close);}return 0
  }
  if s.tab==0&&selected_recipe(s).is_some()&&events&c::DETAIL!=0{s.detail_page=1-s.detail_page;sound::cue(Cue::Tab);}
  if s.mode==2&&events&(c::TAB|c::TAB_PREV|c::TAB_NEXT)!=0{s.tab=1-s.tab;s.aggregate=crate::interaction::AggregateFocus::default();s.focus.material=false;s.focus.index=0;sound::cue(Cue::Tab);return 0}
  if s.mode==2&&s.tab==1{
   s.material_cache.refresh(&s.list,&s.stock,s.stock_ready,g::PROVISIONS);let totals=s.material_cache.totals();s.aggregate.material=s.aggregate.material.min(totals.len().saturating_sub(1));
   if totals.is_empty(){return 0}
   let d=if events&c::UP!=0{-1}else if events&c::DOWN!=0{1}else{0};let page=if events&c::PREV!=0{-1}else if events&c::NEXT!=0{1}else{0};
   s.aggregate.material_step(totals.len(),d,page);
   if page!=0{sound::cue(Cue::Page);}else if d!=0{sound::cue(Cue::Cursor);}return 0
  }
  if s.mode==2&&s.tab==0{
   let count=s.list.slots[s.selected].map_or(0,|e|e.recipe.totals().len());
   if !s.focus.material&&events&c::ACCEPT!=0{if count>0{s.focus.enter(count);sound::cue(Cue::Inspect);}return 0}
   if s.focus.material{let d=if events&c::UP!=0{-1}else if events&c::DOWN!=0{1}else{0};s.focus.step(count,d);if d!=0{sound::cue(Cue::Cursor);}return 0}
  }
  let d=if events&c::UP!=0{-1}else if events&c::DOWN!=0{1}else{0};let page=if events&c::PREV!=0{-1}else if events&c::NEXT!=0{1}else{0};
  s.selected=crate::navigation::move_slot(s.selected,d,page);
  if page!=0{sound::cue(Cue::Page);}else if d!=0{sound::cue(Cue::Cursor);}
  if s.mode==2&&s.tab==0&&events&c::DELETE!=0&&s.list.slots[s.selected].is_some(){s.mode=4;s.focus.yes=false;sound::cue(Cue::Inspect);}
  if s.mode==1&&events&c::ACCEPT!=0{if s.list.slots[s.selected].is_some(){s.mode=3;s.focus.yes=false;sound::cue(Cue::Inspect);}else{return 1}}
  0
 });if action==5{clear_all()}else if action==3{apply_sort()}else if action!=0{commit(action==1)}
}
#[cfg(feature="keyboard-input")]
fn keyboard()->u32{let mut e=0;for (key,event) in [(Key::Enter,c::ACCEPT),(Key::Escape,c::BACK),(Key::Up,c::UP),(Key::Down,c::DOWN),(Key::Left,c::PREV),(Key::Right,c::NEXT),(Key::PageUp,c::PREV),(Key::PageDown,c::NEXT),(Key::Delete,c::DELETE),(Key::Tab,c::TAB),(Key::Q,c::DETAIL),(Key::E,c::DETAIL)]{if Input::is_key_pressed(key){e|=event}}e}
#[cfg(not(feature="keyboard-input"))]
fn keyboard()->u32{0}
fn materials(cv:&Canvas,s:&State,r:Recipe,x:f32,y:f32,w:f32){
 let lang=s.list.language;let totals=r.totals();cv.section(x,y,w,l::MATERIALS[lang]);let mut py=y+40.;let step=if r.group>0&&totals.len()>3{35.}else{46.};
 for (i,m) in totals.iter().enumerate(){let owned=s.stock[m.id as usize];let good=s.stock_ready&&owned>=m.count;
  if s.focus.material&&s.focus.index==i{cv.selection(x,py-3.,w,39_f32.min(step-3.),true,false);}
  let height=39_f32.min(step-3.);let row_y=py-3.;cv.hit(x,row_y,w,height,Hit::Material(i));
  let (tile,tint)=crate::icons::material(m.id);cv.icon(tile,x+4.,row_y+(height-27.)*0.5,27.,tint);
  cv.box_text(x+38.,row_y,w-193.,height,BODY,if good{WHITE}else{RED},&item(m.id,lang),false);
  cv.box_text(x+w-146.,row_y,65.,height,BODY,WHITE,&format!("× {}",m.count),false);
  // Original inventory chest icon from the native item sheet.
  cv.icon(crate::icons::CHEST,x+w-79.,row_y+(height-23.)*0.5,23.,WHITE);
  cv.box_text(x+w-48.,row_y,48.,height,BODY,if good{WHITE}else{RED},&if s.stock_ready{format!("{}",owned)}else{"?".into()},false);
  cv.line(x,py+37_f32.min(step-2.),x+w,py+37_f32.min(step-2.),RULE);py+=step;
 }
 if r.group>0{cv.section(x,351.,w,l::GROUP[lang]);
  // Category labels and points use the same row-centering path as ordinary materials.
  let row_y=380.;let height=39.;
  if let Some(category)=crate::equipment::CATEGORIES.get(r.group as usize){cv.icon(category.0 as usize,x+4.,row_y+(height-27.)*0.5,27.,crate::icons::PALETTE[category.1 as usize]);cv.box_text(x+38.,row_y,w-145.,height,BODY,WHITE,category.2[lang],false);}
  let points=format!("{} pt",r.points);let points_width=cv.measure(BODY,&points).min(100.);cv.box_text(x+w-points_width,row_y,points_width,height,BODY,WHITE,&points,false);
 }

 if s.focus.material{if let Some(m)=totals.get(s.focus.index){
  cv.rect(x,451.,w,140.,INSET);cv.line(x,451.,x+w,451.,RULE);let (tile,tint)=crate::icons::material(m.id);cv.icon(tile,x+10.,493.,44.,tint);
  cv.fit(x+10.,471.,w-20.,26.,GOLD,&item(m.id,lang));
  if let Some(data)=crate::icons::ITEMS.get(m.id as usize){cv.wrap(x+8.,541.,65.,20.,crate::presentation::rarity(data.2 as i16),&format!("{} {}",l::RARITY[lang],data.2));}
  cv.paragraph(x+76.,492.,w-88.,94.,BODY,WHITE,&crate::name_data::material_description(m.id,lang));


 }}
}
fn aggregate(cv:&Canvas,s:&State)->Option<Recipe>{
 let lang=s.list.language;let totals=s.material_cache.totals();let cursor=s.aggregate.material.min(totals.len().saturating_sub(1));
 cv.line(542.,172.,542.,588.,RULE);
 cv.section(146.,168.,380.,l::MATERIALS[lang]);cv.section(561.,168.,367.,l::TITLE[lang]);
 if totals.is_empty(){return None}
 let start=cursor/8*8;
 for (row,m) in totals.iter().skip(start).take(8).enumerate(){
  let y=199.+row as f32*40.;let focus=start+row==cursor;let owned=s.stock[m.id as usize];
  let color=if s.stock_ready&&owned<m.count{RED}else{WHITE};
  if focus{cv.selection(146.,y-1.,380.,38.,true,false);}cv.hit(146.,y-1.,380.,38.,Hit::Aggregate(start+row));
  let (tile,tint)=crate::icons::material(m.id);cv.icon(tile,153.,y+5.,27.,tint);
  cv.box_text(188.,y-1.,197.,38.,BODY,color,&item(m.id,lang),false);cv.box_text(391.,y-1.,58.,38.,BODY,color,&format!("× {}",m.count),false);
  cv.icon(crate::icons::CHEST,455.,y+7.,22.,WHITE);cv.box_text(487.,y-1.,38.,38.,BODY,color,&if s.stock_ready{format!("{}",owned)}else{"?".into()},false);
  cv.line(150.,y+37.,526.,y+37.,RULE);
 }
 cv.pager(336.,528.,cursor/8+1,totals.len().div_ceil(8));
 let Some(material)=totals.get(cursor) else{return None};
 // Material details follow the only cursor in this tab, under the related list.
 let (dx,dw,dy)=(561.,367.,452.);
 cv.line(dx,dy,dx+dw,dy,RULE);cv.fit(dx,dy+6.,250.,23.,GOLD,&item(material.id,lang));
 if let Some(data)=crate::icons::ITEMS.get(material.id as usize){cv.fit(dx+dw-110.,dy+9.,110.,18.,crate::presentation::rarity(data.2 as i16),&format!("{} {}",l::RARITY[lang],data.2));}
 let (tile,tint)=crate::icons::material(material.id);cv.icon(tile,dx,dy+42.,32.,tint);
 cv.paragraph(dx+42.,dy+38.,dw-42.,96.,23.,WHITE,&crate::name_data::material_description(material.id,lang));
 let related=s.material_cache.related(material.id);let count=related.count_ones() as usize;
 // Up to three columns of six rows; beyond that the last cell says "+N more".
 const ROWS:usize=6;let columns=count.div_ceil(ROWS).clamp(1,3);let capacity=ROWS*3;
 let shown=if count>capacity{capacity-1}else{count};let column_width=367./columns as f32;let (row_height,icon_size,font_size)=(35.,24.,21.);let top=200.;
 for (index,slot) in (0..48).filter(|slot|related&(1u64<<slot)!=0).take(shown).enumerate(){
  let Some(entry)=s.list.slots[slot] else{continue};let x=561.+(index/ROWS) as f32*column_width;let y=top+(index%ROWS) as f32*row_height;
  cv.gear(entry.recipe,x+4.,y+(row_height-icon_size)*0.5,icon_size);cv.box_text(x+icon_size+10.,y,column_width-icon_size-16.,row_height,font_size,WHITE,&name(entry.recipe,lang),false);
 }
 if count>shown{let index=shown;let x=561.+(index/ROWS) as f32*column_width;let y=top+(index%ROWS) as f32*row_height;cv.box_text(x+4.,y,column_width-8.,row_height,font_size,MUTED,&l::MORE[lang].replace("{}",&format!("{}",count-shown)),false);}

 (0..48).find(|slot|related&(1u64<<slot)!=0).and_then(|slot|s.list.slots[slot]).map(|e|e.recipe)

}
fn draw_notice(cv:&Canvas,r:Recipe,progress:Option<crate::notifications::Progress>,lang:usize,elapsed:f32,slot:usize){
 let title=format!("{} \"{}\"",l::TITLE[lang],name(r,lang));
 // A partial gain names the material and its count; a full set keeps the gathered text.
 let progress_text=progress.map(|p|format!("{} ( {} / {} )",item(p.material,lang),p.have,p.need));let body=progress_text.as_deref().unwrap_or(l::NOTICE[lang]);
 let title_size=(21.*842./cv.measure(21.,&title).max(842.)).max(14.);let body_size=23.;let title_w=cv.measure(title_size,&title);let body_w=cv.measure(body_size,body);
 let w=(title_w+40.).max(body_w).min(876.)+32.;
 let right=(imgui::get_io().DisplaySize.x-cv.ox)/cv.scale-18.;let opacity=crate::presentation::notice_opacity(elapsed);let x=right-w;
 // Quest maps occupy the upper-right until ~350; keep the whole toast below them.
 let y=crate::notifications::top(slot);cv.rect(x,y,w,80.,crate::presentation::alpha(0xf02d281f,opacity));cv.border(x,y,w,80.,TEAL,&[],opacity);
 let group_x=x+(w-title_w-40.)*0.5;cv.gear_alpha(r,group_x,y+9.,30.,opacity);
 cv.box_bold(group_x+40.,y+7.,title_w+1.,34.,title_size,crate::presentation::alpha(GOLD,opacity),&title,true);
 cv.box_bold(x+16.,y+38.,w-32.,34.,body_size,crate::presentation::alpha(WHITE,opacity),body,true);

}
#[cfg_attr(not(feature="debug-ui"),allow(dead_code))] // debug-ui builds
fn start_notice_demo(s:&mut State){
 let mut recipes=Vec::new();for entry in s.list.slots.iter().flatten(){if !recipes.contains(&entry.recipe){recipes.push(entry.recipe);}if recipes.len()==4{break}}
 for r in g::RECIPES{if recipes.len()==4{break}if !recipes.contains(r){recipes.push(*r);}}
 if recipes.len()!=4{return}
 s.notice_demo=Some(crate::notifications::Demo::new(core::array::from_fn(|i|recipes[i]),s.notice_test_case));s.mode=7;s.preview=false;sound::cue(Cue::Open);
}
fn draw_notice_demo(cv:&Canvas,s:&mut State){
 let lang=s.list.language;let Some(demo)=s.notice_demo.as_mut() else{return};
 cv.frame(128.,110.,610.,268.);cv.heading(146.,124.,574.,l::NOTICE_TEST[lang]);
 cv.fit(146.,178.,574.,26.,GOLD,l::NOTICE_SCENARIOS[demo.scenario][lang]);
 cv.paragraph(146.,218.,574.,78.,24.,WHITE,l::NOTICE_TEST_HELP[lang]);
 cv.text(146.,306.,574.,24.,WHITE,&format!("{}: {}    {}: {}",l::VISIBLE[lang],demo.queue.visible_count(),l::QUEUED[lang],demo.queue.waiting_count()));
 if demo.queue.is_empty(){cv.fit(146.,343.,574.,22.,GREEN,l::TEST_FINISHED[lang]);}
 cv.bar(755.,642.,160.,crate::icons::A,l::REPLAY[lang]);cv.bar(755.,676.,160.,crate::icons::B,l::BACK[lang]);
 for (slot,n) in demo.queue.visible(){draw_notice(cv,n.recipe,n.progress,lang,crate::presentation::NOTICE_DURATION-n.seconds,slot);}
 demo.advance(imgui::get_io().DeltaTime);
}
fn selected_recipe(s:&State)->Option<Recipe>{
 if s.preview&&matches!(s.preview_panel,1|7){s.frozen}
 else if s.preview&&s.preview_panel==2{None}
 else{s.list.slots[s.selected].map(|e|e.recipe)}
}
fn details(cv:&Canvas,s:&State,r:Option<Recipe>){
 let lang=s.list.language;cv.frame(964.,52.,296.,568.);
 cv.heading(984.,82.,266.,l::DETAILS[lang]);
 let Some(r)=r else{cv.box_text(984.,69.,266.,551.,26.,MUTED,l::NO_EQUIPMENT[lang],true);return};
 cv.hit(981.,128.,34.,34.,Hit::Detail(0));cv.hit(1218.,128.,34.,34.,Hit::Detail(1));cv.icon(crate::icons::ZL,985.,132.,26.,WHITE);cv.center(1023.,129.,188.,25.,WHITE,&format!("{} / 2",s.detail_page+1));cv.icon(crate::icons::ZR,1222.,132.,26.,WHITE);
 cv.rect(984.,169.,266.,45.,INSET);cv.rect(984.,169.,3.,45.,GOLD);cv.icon_plate(995.,176.,31.);cv.gear(r,997.,178.,27.);cv.box_text(1036.,169.,205.,45.,BODY,GOLD,&name(r,lang),false);
 cv.rect(984.,224.,266.,382.,INSET);
 if s.detail_page==1{
  // Better Skillinfo draws its panel left of this frame, inside the list window.
  let (edge,foot)=(cv.p(964.,69.),cv.p(964.,620.));crate::skill_view::show(r.kind,r.id,r.level,edge.x,edge.y,foot.y);
  let description=if r.kind==25{crate::name_data::material_description(r.id,lang)}else{crate::text_data::description(r.kind,r.id,lang)};
  cv.paragraph(995.,231.,244.,177.,BODY,WHITE,&description);cv.line(995.,417.,1239.,417.,RULE);
  cv.section(995.,426.,244.,l::SKILLS[lang]);
  if let Some(d)=crate::equipment::get(r){if d.skills.is_empty(){cv.center(995.,466.,244.,23.,MUTED,"—");}for (i,(id,pt)) in d.skills.iter().enumerate(){let y=465.+i as f32*33.;cv.box_text(995.,y,189.,30.,24.,WHITE,crate::equipment::skill(*id,lang),false);cv.box_text(1190.,y,49.,30.,24.,if *pt>0{GREEN}else{RED},&format!("{:+}",pt),false);cv.line(995.,y+30.,1239.,y+30.,RULE);}}
 }else if let Some(d)=crate::equipment::get(r){
  let armor=(1..=5).contains(&r.kind)||(23..=24).contains(&r.kind);
  let weapon=!armor&&r.kind!=25;let sharp=weapon&&d.sharp.iter().any(|n|*n>0);
  let count=3+if weapon{2+d.elements.iter().filter(|(element,value)|*element>0&&*value>0).count()}else{0}+if armor||d.defense!=0{1}else{0}+if armor{5}else{0};
  let step=((371.-if sharp{51.}else{0.})/count as f32).min(35.);let font=24_f32.min(step-3.);let mut y=231.;
  let row_color=|label:&str,value:&str,py:f32,color:u32|{cv.box_text(995.,py,168.,step-2.,font,color,label,false);cv.box_text(1175.,py,64.,step-2.,font+1.,color,value,false);cv.line(995.,py+step-2.,1239.,py+step-2.,RULE);};
  let row=|label:&str,value:&str,py:f32|row_color(label,value,py,WHITE);
  let rarity=if r.kind==25{crate::icons::ITEMS.get(r.id as usize).map_or(d.rarity,|i|i.2 as i16)}else{d.rarity};row_color(l::RARITY[lang],&format!("{}",rarity),y,crate::presentation::rarity(rarity));y+=step;row("Lv",&format!("{}",r.level.max(1)),y);y+=step;
  if !armor&&r.kind!=25{
   row(l::ATTACK[lang],&format!("{}",d.attack),y);y+=step;
   for (element,value) in d.elements{if element>0&&value>0{let idx=element as usize-1;let label=crate::equipment::ELEMENTS.get(idx).map_or("",|v|v[lang]);cv.icon(crate::icons::ELEMENT_START+idx,995.,y+3.,22.,WHITE);cv.box_text(1023.,y,140.,step-2.,font,WHITE,&label,false);cv.box_text(1175.,y,64.,step-2.,font+1.,WHITE,&format!("{}",value),false);cv.line(995.,y+step-2.,1239.,y+step-2.,RULE);y+=step;}}
   if sharp{cv.fit(995.,y,244.,font,BLUE,l::SHARPNESS[lang]);y+=26.;cv.sharpness(995.,y,244.,d.sharp);y+=25.;}
   row(l::AFFINITY[lang],&format!("{}%",d.affinity),y);y+=step;
  }
  if r.kind!=25&&(armor||d.defense!=0){row(l::DEFENSE[lang],&format!("{}",d.defense),y);y+=step;}
  cv.box_text(995.,y,151.,step-2.,font,WHITE,l::SLOTS[lang],false);cv.slots(1167.,y+3.,d.slots);cv.line(995.,y+step-2.,1239.,y+step-2.,RULE);y+=step;
  let resist=[ ["Fire","火","火","火耐性","불 내성","Feuer","Feu","Fuoco","Fuego"],["Water","水","水","水耐性","물 내성","Wasser","Eau","Acqua","Agua"],["Thunder","雷","雷","雷耐性","번개 내성","Donner","Foudre","Tuono","Rayo"],["Ice","冰","冰","氷耐性","얼음 내성","Eis","Glace","Ghiaccio","Hielo"],["Dragon","龙","龍","龍耐性","용 내성","Drache","Dragon","Drago","Dragón"] ];
  if armor{for i in 0..5{row_color(resist[i][lang],&format!("{}",d.resist[i]),y,crate::presentation::ELEMENTS[i]);y+=step;}}

 }
}
fn action_a(cv:&Canvas,lang:usize){let x=501.;let w=426.;cv.box_text(x,319.,w,35.,29.,WHITE,l::REGISTER_ACTION[lang],true);cv.icon(crate::icons::A,x+(w-34.)*0.5,362.,34.,WHITE);}
/// Registered-equipment pins on the game's lists, or on KatSmithy's armor list
/// when it replaces the hunter's production list.
fn draw_pins(native_cv:&Canvas,s:&crate::State){
 let armor=crate::katsmithy::armor_view(s.base);let tree=crate::katsmithy::weapon_view(s.base);
 for marker in crate::native_badges::markers_with(&s.list,s.base,armor.is_none()&&tree.is_none()){native_cv.pin(marker.x,marker.y);}
 // KatSmithy's lists use its own centered 1280x720 transform, not the game HUD's.
 let io=imgui::get_io();let Some((ox,oy,scale))=crate::katsmithy::transform(io.DisplaySize.x,io.DisplaySize.y) else{return};
 let kat=Canvas{scale,ox,oy,draw:native_cv.draw};
 if let Some(view)=armor{for (x,y) in crate::katsmithy::armor_pins(view,&s.list){kat.pin(x,y);}}
 if let Some(view)=tree{for (x,y) in crate::katsmithy::weapon_pins(view,&s.list){kat.pin(x,y);}}
}
fn confirmation(cv:&Canvas,s:&State){let lang=s.list.language;clear_hits();cv.draw.add_rect_filled(ImVec2{x:0.,y:0.},imgui::get_io().DisplaySize,0x90000000,0.);cv.popup(400.,307.,480.,176.);
   let text=if s.mode==5{l::DONE[lang]}else if s.mode==3{l::REPLACE_QUESTION[lang]}else if s.mode==9{l::CLEAR_QUESTION[lang]}else{l::DELETE_QUESTION[lang]};
   for (i,line) in text.lines().enumerate(){cv.center(418.,327.+i as f32*32.,444.,26.,WHITE,line);}
 if s.mode==5{cv.hit(564.,424.,152.,39.,Hit::Event(c::ACCEPT));cv.button(564.,424.,152.,39.,true,l::CLOSE[lang]);}
 else{for (yes,x,label) in [(true,460.,l::YES[lang]),(false,668.,l::NO[lang])]{cv.hit(x,424.,152.,39.,Hit::Yes(yes));cv.button(x,424.,152.,39.,s.focus.yes==yes,label);}}
  }
fn sort_dialog(cv:&Canvas,s:&State){
 let Some(d)=s.sort else{return};let lang=s.list.language;clear_hits();
 cv.draw.add_rect_filled(ImVec2{x:0.,y:0.},imgui::get_io().DisplaySize,if d.secondary{0xa0000000}else{0x70000000},0.);
 let (x,w)=if d.secondary{(280.,720.)}else{(400.,480.)};cv.popup(x,170.,w,420.);cv.heading(x+18.,183.,w-36.,l::SORT[lang]);
 let left=x+18.;let column=if d.secondary{326.}else{w-36.};cv.section(left,230.,column,l::SORT_PRIMARY[lang]);
 for (i,text) in l::SORT_CRITERIA.iter().enumerate(){let y=275.+i as f32*62.;if !d.secondary{cv.hit(left,y,column,48.,Hit::Criterion(i));}cv.selection(left,y,column,48.,d.criterion==i,false);let icon_x=left+10.;let icon_y=y+11.;
  if i==0{cv.icon(SORT_ORDER_ICON,icon_x,icon_y,26.,GOLD);}else{
   cv.icon(SORT_STOCK_ICON,icon_x,icon_y,26.,if i==1{GREEN}else{RED});
   let (cx,cy)=(icon_x+20.,icon_y+20.);cv.rect(cx-5.,cy-5.,12.,12.,INSET);
   if i==1{cv.line(cx-4.,cy,cx-1.,cy+4.,GREEN);cv.line(cx-1.,cy+4.,cx+6.,cy-4.,GREEN);}else{cv.line(cx-3.,cy-3.,cx+4.,cy+4.,RED);cv.line(cx-3.,cy+4.,cx+4.,cy-3.,RED);}
  }cv.box_text(left+48.,y,column-58.,48.,BODY,WHITE,text[lang],false);}
 if d.secondary{
  // Only the right column is active; shade the complete previous-stage panel.
  cv.rect(left,224.,column,245.,0x99000000);let right=x+382.;cv.section(right,230.,w-400.,l::SORT_SECONDARY[lang]);
  for (i,text) in l::SORT_CATEGORIES.iter().enumerate(){let y=265.+i as f32*44.;cv.hit(right,y,w-400.,38.,Hit::Category(i));cv.selection(right,y,w-400.,38.,d.category==i,false);let tile=match i{1=>crate::icons::equipment(7),2=>crate::icons::equipment(2),3=>SORT_DECORATION_ICON,4=>crate::icons::equipment(22),_=>crate::icons::equipment(24)};
   if i==0{cv.sprite(crate::icons::SLOT_DASH,right+9.,y+16.,24.,7.,MUTED,false);}else{cv.icon(tile,right+8.,y+6.,26.,if i==3{GOLD}else{WHITE});}
   cv.box_text(right+44.,y,w-454.,38.,24.,WHITE,text[lang],false);}
 }
 if d.message>0{cv.paragraph(x+18.,535.,w-36.,45.,21.,RED,if d.message==1{l::SORT_WAIT[lang]}else{l::SAVE_FAILED[lang]});}
}
/// A Recomp pointer position (console pixels) in display pixels, mapped over
/// the game's picture (see hud_layout::mouse).
fn screen_mouse(x:f32,y:f32)->(f32,f32){
 let io=imgui::get_io();let base=STATE.with(|s|s.base);
 crate::hud_layout::mouse(x,y,io.DisplaySize.x,io.DisplaySize.y,crate::hud_layout::current(base))
}
/// One prompt part: an atlas sprite or a short text in a raised box.
enum Part{Tile(usize),Text(String),Key(&'static str)}
/// Sprites for a pad chord in the shortcut bit layout (HID order).
fn chord_parts(chord:u32)->Vec<Part>{
 let extra=crate::button_prompts::PAD_EXTRA_TILE;
 (0..24).filter(|bit|chord&(1<<bit)!=0).map(|bit|match bit{
  0=>Part::Tile(crate::icons::A),1=>Part::Tile(crate::icons::B),2=>Part::Tile(crate::icons::X),3=>Part::Tile(crate::icons::Y),
  4=>Part::Tile(crate::icons::L3),5=>Part::Tile(crate::icons::R3),6=>Part::Tile(crate::icons::L),7=>Part::Tile(crate::icons::R),
  8=>Part::Tile(crate::icons::ZL),9=>Part::Tile(crate::icons::ZR),10=>Part::Tile(extra+2),11=>Part::Tile(extra+3),
  12=>Part::Tile(crate::icons::DPAD_LEFT),13=>Part::Tile(extra),14=>Part::Tile(crate::icons::DPAD),15=>Part::Tile(extra+1),
  16=>Part::Text("L←".into()),17=>Part::Text("L↑".into()),18=>Part::Text("L→".into()),19=>Part::Text("L↓".into()),
  20=>Part::Text("R←".into()),21=>Part::Text("R↑".into()),22=>Part::Text("R→".into()),_=>Part::Text("R↓".into()),
 }).collect()
}
/// The prompt part of a key by its Kenney name: a wide keycap or a sprite.
fn key_part(name:&str)->Option<Part>{
 if let Some(label)=crate::button_prompts::wide_label(name){return Some(Part::Key(label))}
 crate::icons::key_tile(name).map(Part::Tile)
}
/// Sprites for a keyboard binding (Ctrl/Shift/Alt + one HID key).
fn key_parts(binding:u32)->Vec<Part>{
 let mut out=Vec::new();
 for (bit,name) in [(1<<16,"keyboard_ctrl"),(1<<17,"keyboard_shift"),(1<<18,"keyboard_alt")]{if binding&bit!=0{if let Some(p)=key_part(name){out.push(p);}}}
 let usage=binding&255;
 if usage!=0{out.push(crate::button_prompts::hid_key_name(usage).and_then(key_part).unwrap_or_else(||Part::Text(crate::bindings::key_label(usage))));}
 out
}
impl Canvas{
 /// A Recomp pointer position (console pixels, 1280x720) in this canvas's units.
 fn from_console(&self,x:f32,y:f32)->(f32,f32){let (sx,sy)=screen_mouse(x,y);((sx-self.ox)/self.scale,(sy-self.oy)/self.scale)}
 /// A light keycap with a word, as wide as the word needs (or `fit` wide).
 fn keycap(&self,label:&str,x:f32,y:f32,size:f32,fit:Option<f32>,draw:bool)->f32{
  let inset=size*0.1;let text=size*0.42;let w=fit.unwrap_or_else(||size.max(self.measure(text,label)+size*0.5));
  if draw{
   self.draw.add_rect_filled(self.p(x+inset*0.5,y+inset),self.p(x+w-inset*0.5,y+size-inset),0xfff2f2f2,size*0.16*self.scale);
   self.box_text(x+inset,y+inset,w-inset*2.,size-inset*2.,text,0xff26221d,label,true);
  }
  w
 }
 /// Prompt parts left to right from x; returns the width used.
 fn parts(&self,parts:&[Part],x:f32,y:f32,size:f32,draw:bool)->f32{
  let mut px=x;
  for (i,part) in parts.iter().enumerate(){
   if i>0{if draw{self.box_text(px,y,size*0.3,size,size*0.6,WHITE,"+",true);}px+=size*0.3;}
   match part{
    Part::Tile(t)=>{if draw{self.raw_icon(*t,px,y,size,WHITE);}px+=size;}
    Part::Text(text)=>{let w=size.max(self.measure(size*0.55,text)+size*0.3);if draw{self.rect(px,y,w,size,RAISED);self.box_text(px,y,w,size,size*0.55,WHITE,text,true);}px+=w;}
    Part::Key(label)=>{px+=self.keycap(label,px,y,size,None,draw);}
   }
  }
  px-x
 }
}
// Skill-Simulator guest link (render thread only).
struct Guest{dialog:Option<crate::sksim_model::Dialog>,gate:crate::sksim_model::Gate,keys:crate::sksim_model::Edges,shortcut:crate::binding_model::Edges}
static mut GUEST:Guest=Guest{dialog:None,gate:crate::sksim_model::Gate::new(),keys:crate::sksim_model::Edges::new(),shortcut:crate::binding_model::Edges::new()};
fn guest()->&'static mut Guest{unsafe{&mut *core::ptr::addr_of_mut!(GUEST)}}
/// The craftable recipe of an armor piece, as on KatSmithy's armor list.
fn armor_recipe(part:u8,series:u16)->Option<Recipe>{g::RECIPES.iter().find(|r|(r.kind==part||r.kind==0)&&r.id==series&&r.level==0).map(|r|Recipe{kind:part,..*r})}
/// Dialog geometry in Skill-Simulator's canvas (1280x720 units around its
/// window centre, its own scale), sized like its own dialogs: top, height, rows.
fn guest_layout(d:&crate::sksim_model::Dialog)->(f32,f32,usize){
 use crate::sksim_model::Plan;
 let rows=if d.outcome.is_some(){0}else{match &d.plan{Plan::Register(r)=>r.len()+d.left_out.len(),Plan::Remove(_)=>1,Plan::None=>0}};
 let h=24.+56.+if rows>0{10.+rows as f32*34.}else{0.}+18.+38.+20.;
 (360.-h*0.5,h,rows)
}
const GUEST_W:f32=440.;
/// Buttons of the dialog: (answer is yes, x, y).
fn guest_buttons(d:&crate::sksim_model::Dialog)->Vec<(bool,f32,f32)>{
 let (top,h,_)=guest_layout(d);let y=top+h-58.;
 if d.outcome.is_some(){alloc::vec![(false,565.,y)]}else{alloc::vec![(true,467.,y),(false,663.,y)]}
}
fn guest_text(d:&crate::sksim_model::Dialog,lang:usize)->String{
 use crate::sksim_model::{Plan,Outcome};
 match d.outcome{
  Some(Outcome::Registered(_))=>l::DONE[lang].into(),Some(Outcome::Removed)=>l::REMOVED[lang].into(),
  Some(Outcome::Short(n))=>l::SHORT[lang].replace("{}",&format!("{}",n)),Some(Outcome::Failed)=>l::SAVE_FAILED[lang].into(),
  None=>match &d.plan{Plan::Register(r) if r.len()>1=>l::SET_QUESTION[lang].replace("{}",&format!("{}",r.len())),Plan::Remove(_)=>l::DELETE_QUESTION[lang].into(),_=>l::PART_QUESTION[lang].into()},
 }
}
/// The canvas the dialog is drawn in: Skill-Simulator's scale, centred on its
/// window (it can sit to one side); our list canvas when that is unknown.
fn guest_canvas(v:&crate::sksim_model::View)->Option<Canvas>{
 let draw=imgui::DrawList::foreground();if draw.raw().is_null(){return None}
 match crate::sksim::window_rect(){
  Some(r) if v.scale.is_finite()&&v.scale>0.=>Some(Canvas{scale:v.scale,ox:(r[0]+r[2])*0.5-640.*v.scale,oy:(r[1]+r[3])*0.5-360.*v.scale,draw}),
  _=>Canvas::new(),
 }
}
fn guest_dialog(cv:&Canvas,d:&crate::sksim_model::Dialog,list:&List,lang:usize){
 use crate::sksim_model::Plan;
 let (top,h,rows)=guest_layout(d);let x=640.-GUEST_W*0.5;
 // Dim Skill-Simulator's window only, as its own dialogs do.
 match crate::sksim::window_rect(){
  Some(r)=>cv.draw.add_rect_filled(ImVec2{x:r[0],y:r[1]},ImVec2{x:r[2],y:r[3]},0x90000000,0.),
  None=>cv.draw.add_rect_filled(ImVec2{x:0.,y:0.},imgui::get_io().DisplaySize,0x90000000,0.),
 }
 cv.popup(x,top,GUEST_W,h);
 let text=guest_text(d,lang);
 // One or two centred lines; a long single message wraps.
 if text.contains('\n'){for (i,line) in text.lines().enumerate(){cv.center(x+16.,top+24.+i as f32*28.,GUEST_W-32.,22.,WHITE,line);}}
 else if cv.measure(24.,&text)<=GUEST_W-32.{cv.box_text(x+16.,top+22.,GUEST_W-32.,58.,24.,WHITE,&text,true);}
 else{cv.paragraph(x+16.,top+22.,GUEST_W-32.,58.,21.,WHITE,&text);}
 if rows>0{
  let recipes:Vec<Recipe>=match &d.plan{Plan::Register(r)=>r.clone(),Plan::Remove(slot)=>list.slots[*slot].map(|e|e.recipe).into_iter().collect(),Plan::None=>Vec::new()};
  for (i,r) in recipes.iter().enumerate(){let y=top+90.+i as f32*34.;cv.icon_plate(x+48.,y+2.,29.);cv.gear(*r,x+50.,y+4.,25.);cv.box_text(x+88.,y,GUEST_W-120.,33.,22.,WHITE,&name(*r,lang),false);}
  // Pieces of the set that stay out, dimmed, each with its reason.
  for (i,(part,series,reason)) in d.left_out.iter().enumerate(){
   use crate::sksim_model::Reason;
   let y=top+90.+(recipes.len()+i) as f32*34.;let r=Recipe{kind:*part,id:*series,..Recipe::default()};
   cv.icon_plate(x+48.,y+2.,29.);cv.gear_alpha(r,x+50.,y+4.,25.,0.5);cv.box_text(x+88.,y,GUEST_W-230.,33.,22.,MUTED,&name(r,lang),false);
   let why=match reason{Reason::Owned=>l::REASON_OWNED[lang],Reason::Reward=>l::REASON_REWARD[lang],Reason::Registered=>l::REASON_REGISTERED[lang],Reason::NoRecipe=>l::REASON_NO_RECIPE[lang]};
   let w=cv.measure(18.,why).min(110.);cv.box_text(x+GUEST_W-48.-w,y,w,33.,18.,RED,why,false);
  }
 }
 cv.rect(x+24.,top+h-72.,GUEST_W-48.,1.,RULE);
 for (yes,bx,by) in guest_buttons(d){
  let label=if d.outcome.is_some(){l::CLOSE[lang]}else if yes{l::YES[lang]}else{l::NO[lang]};
  let focused=d.outcome.is_some()||d.yes==yes;
  if focused{cv.cursor(bx,by,150.,38.);}else{cv.draw.add_rect(cv.p(bx,by),cv.p(bx+150.,by+38.),RULE,0.,cv.scale,ImDrawFlags::None);}
  cv.box_text(bx,by,150.,38.,20.,if focused{WHITE}else{MUTED},label,true);
 }
}
/// The pointer over the dialog: hovering picks a button, a click answers.
fn guest_mouse(cv:&Canvas,d:&mut crate::sksim_model::Dialog)->Option<crate::sksim_model::Answer>{
 use crate::sksim_model::Answer;
 let m=crate::kbm::mouse()?;let (x,y)=cv.from_console(m.x,m.y);
 if m.right{return Some(Answer::Close)}
 let hit=guest_buttons(d).into_iter().find(|(_,bx,by)|(*bx..bx+152.).contains(&x)&&(*by..by+39.).contains(&y));
 if let Some((yes,_,_))=hit{
  if d.outcome.is_none()&&m.moved{d.yes=yes;}
  if m.left{return Some(if d.outcome.is_none()&&yes{Answer::Yes}else{Answer::Close})}
 }
 None
}
/// Apply a confirmed plan; the dialog then shows the outcome.
fn guest_commit(plan:&crate::sksim_model::Plan)->crate::sksim_model::Outcome{
 use crate::sksim_model::{Plan,Outcome};
 let next=STATE.with(|s|{
  let mut next=s.list.clone();
  match plan{
   Plan::Register(recipes)=>{for r in recipes{let Some(slot)=next.slots.iter().position(Option::is_none) else{return None};if next.register(slot,*r,crate::equipment::cost(*r)).is_err(){return None}if s.stock_ready{if let Some(e)=next.slots[slot].as_mut(){e.ready=Some(r.ready(&s.stock,g::PROVISIONS));}}}}
   Plan::Remove(slot)=>next.remove_slot(*slot),
   Plan::None=>return None,
  }
  Some(next)
 });
 let Some(next)=next else{return Outcome::Failed};
 if !save(next){return Outcome::Failed}
 sound::cue(if matches!(plan,Plan::Remove(_)){Cue::Inspect}else{Cue::Registered});
 match plan{Plan::Register(r)=>Outcome::Registered(r.len()),_=>Outcome::Removed}
}
/// Every frame while the list is closed. Returns whether Skill-Simulator's
/// window is open (our own prompts and pins then stay off the screen).
fn guest_frame(sim:Option<crate::sksim_model::View>,key_register:bool,pad_ok:bool)->bool{
 use crate::sksim_model::{self as m,Answer,Plan};
 let gs=guest();
 let Some(v)=sim else{gs.dialog=None;gs.gate=m::Gate::new();crate::sksim::request(0.,false);return false};
 let pressed=gs.keys.step(v.raw);
 let (list,lang)=STATE.with(|s|(s.list.clone(),s.list.language));
 let shortcut=gs.shortcut.step(crate::binding_model::active(v.raw&crate::binding_model::PAD_MASK,&list.pad_bindings))&2!=0&&pad_ok;
 let plan=m::plan(&v,&list,armor_recipe);
 if gs.dialog.is_none()&&(shortcut||key_register)&&plan!=Plan::None{let mut d=m::Dialog::new(plan.clone(),&list);d.left_out=m::left_out(&v,&list,armor_recipe);gs.dialog=Some(d);sound::cue(Cue::Inspect);}
 let cv=if gs.dialog.is_some(){guest_canvas(&v)}else{Canvas::new()};
 if let Some(d)=gs.dialog.as_mut(){
  let before=d.yes;
  let answer=d.input(pressed).or_else(||cv.as_ref().and_then(|cv|guest_mouse(cv,d)));
  if d.yes!=before{sound::cue(Cue::Cursor);}
  match answer{
   Some(Answer::Yes)=>{let outcome=guest_commit(&d.plan);d.outcome=Some(outcome);}
   Some(Answer::Close)=>{gs.dialog=None;sound::cue(Cue::Close);}
   None=>{}
  }
 }
 let modal=gs.gate.modal(gs.dialog.is_some(),v.raw);
 let mut width=0.;
 if let Some(cv)=cv.as_ref(){
  if let Some(d)=gs.dialog.as_ref(){guest_dialog(cv,d,&list,lang);}
  else if plan!=Plan::None&&!v.overlay{
   let label=match &plan{Plan::Remove(_)=>l::QUICK_REMOVE[lang],_ if v.kind==1=>l::SET_REGISTER[lang],_=>l::REGISTER[lang]};
   let px=Canvas{scale:1.,ox:0.,oy:0.,draw:cv.draw};
   let s=if v.scale.is_finite()&&v.scale>0.{v.scale}else{1.};
   let parts=if crate::bindings::keyboard_prompts()&&list.key_bindings[1]&255!=0{key_parts(list.key_bindings[1])}else{chord_parts(list.pad_bindings[1])};
   let (top,bottom)=(v.hint[1],v.hint[3]);let h=if bottom>top{bottom-top}else{40.*s};
   let icon=32.*s;let gap=6.*s;let size=24.*s;
   let text=px.measure(size,label);
   let drawn=v.hint[2]>v.hint[0];
   let used=px.parts(&parts,v.hint[0],top+(h-icon)*0.5,icon,drawn);
   if drawn{px.box_text(v.hint[0]+used+gap,top,text+2.,h,size,WHITE,label,false);}
   width=used+gap+text+2.;
  }
 }
 crate::sksim::request(width,modal);
 true
}
/// Mouse targets of our window, recorded while drawing and used next frame.
#[derive(Clone,Copy,Debug,PartialEq)]
enum Hit{Slot(usize),Tab(usize),Material(usize),Aggregate(usize),Yes(bool),Criterion(usize),Category(usize),Event(u32),Detail(u8)}
static mut HITS:Vec<(f32,f32,f32,f32,Hit)>=Vec::new();
/// The control under the pointer, as of this frame's input.
static mut HOVER:Option<Hit>=None;
/// Skill-Simulator's hover gold.
const HOVER_GOLD:u32=0xff6ce6f8;
/// Controls that show a gold outline under the pointer; rows move the cursor instead.
fn outlined(hit:Hit)->bool{matches!(hit,Hit::Tab(_)|Hit::Event(_)|Hit::Detail(_))}
/// The list canvas transform the targets were recorded with.
static mut HIT_VIEW:(f32,f32,f32)=(0.,0.,1.);
fn clear_hits(){unsafe{(*core::ptr::addr_of_mut!(HITS)).clear();}}
/// Pointer input on our open window (Recomp): hovering moves the cursor, a
/// left click selects and confirms, a right click goes back and the wheel
/// turns pages. Clicks become the same events as buttons.
fn mouse_events()->u32{
 let Some(m)=crate::kbm::mouse() else{unsafe{*core::ptr::addr_of_mut!(HOVER)=None;}return 0};
 let (ox,oy,scale)=unsafe{*core::ptr::addr_of!(HIT_VIEW)};
 let (sx,sy)=screen_mouse(m.x,m.y);let (x,y)=((sx-ox)/scale,(sy-oy)/scale);
 let mut e=0;if m.right{e|=c::BACK;}if m.wheel>0{e|=c::PREV;}if m.wheel<0{e|=c::NEXT;}
 let hit=unsafe{(*core::ptr::addr_of!(HITS)).iter().rev().find(|(hx,hy,hw,hh,_)|x>=*hx&&x<hx+hw&&y>=*hy&&y<hy+hh).map(|h|h.4)};
 // Entering a control sounds like a cursor move.
 let hover=hit.filter(|h|outlined(*h));
 if unsafe{core::ptr::addr_of_mut!(HOVER).replace(hover)}!=hover&&hover.is_some(){sound::cue(Cue::Cursor);}
 let Some(hit)=hit else{return e};
 if !m.moved&&!m.left{return e}
 STATE.with(|s|{
  let mut cursor=false;
  match hit{
   Hit::Slot(i)=>{if m.left{if s.focus.material{s.focus.back();}s.selected=i;e|=c::ACCEPT;}else if !s.focus.material&&s.selected!=i{s.selected=i;cursor=true;}}
   Hit::Tab(t)=>{if m.left&&s.mode==2&&s.tab!=t{e|=c::TAB;}}
   Hit::Material(i)=>{
    let count=s.list.slots[s.selected].map_or(0,|e|e.recipe.totals().len());
    if m.left&&!s.focus.material&&s.mode==2{s.focus.enter(count);if s.focus.material{s.focus.index=i;sound::cue(Cue::Inspect);}}
    else if s.focus.material&&s.focus.index!=i{s.focus.index=i;cursor=true;}
   }
   Hit::Aggregate(i)=>{if s.aggregate.material!=i{s.aggregate.material=i;cursor=true;}}
   Hit::Yes(yes)=>{if s.focus.yes!=yes{s.focus.yes=yes;cursor=true;}if m.left{e|=c::ACCEPT;}}
   Hit::Criterion(i)=>{if let Some(d)=s.sort.as_mut(){if !d.secondary{if d.criterion!=i{d.criterion=i;d.message=0;cursor=true;}if m.left{e|=c::ACCEPT;}}}}
   Hit::Category(i)=>{if let Some(d)=s.sort.as_mut(){if d.secondary{if d.category!=i{d.category=i;d.message=0;cursor=true;}if m.left{e|=c::ACCEPT;}}}}
   Hit::Event(event)=>{if m.left{e|=event;}}
   // Either detail page icon turns the page; each gets its own outline.
   Hit::Detail(_)=>{if m.left{e|=c::DETAIL;}}
  }
  if cursor{sound::cue(Cue::Cursor);}
 });
 e
}
/// A transparent full-screen window while ours is open: clicks never reach
/// the game's menus or other ImGui windows behind it.
fn mouse_shield(){
 use forge::sys::imgui::{ImGuiCond,ImGuiWindowFlags as F};
 let io=imgui::get_io();
 imgui::set_next_window_pos(ImVec2{x:0.,y:0.},ImGuiCond::Always,ImVec2{x:0.,y:0.});
 imgui::set_next_window_size(ImVec2{x:io.DisplaySize.x,y:io.DisplaySize.y},ImGuiCond::Always);
 imgui::begin(c"##wishlist-mouse-shield",None,F::NoTitleBar|F::NoResize|F::NoMove|F::NoSavedSettings|F::NoScrollbar|F::NoScrollWithMouse|F::NoBackground|F::NoNav|F::NoFocusOnAppearing|F::NoBringToFrontOnFocus);
 imgui::end();
}
pub fn render(){
 crate::skill_view::begin_frame();
 crate::button_prompts::refresh(imgui::get_io().DeltaTime);
 crate::kbm::poll(crate::bindings::pad_pressed());
 let menu=forge::graphics::Graphics::is_menu_visible();let capturing=crate::bindings::capturing();
 let pad_allowed=crate::bindings::pad_shortcuts_allowed();
 let keys=crate::bindings::keyboard_events(menu||capturing);
 if capturing{STATE.with(|s|s.events=0);crate::bindings::capture_overlay(crate::bindings::language(STATE.with(|s|s.list.language)));return}
 if menu{STATE.with(|s|s.events=0);return}
 if !STATE.with(|s|s.installed&&s.loaded&&s.game_ready){return}
 let pending=STATE.with(|s|core::mem::take(&mut s.events));
 // The keyboard guard holds back pad shortcuts only: window actions may come
 // from keys that Recomp maps onto menu actions.
 let pad=if pad_allowed{pending}else{pending&!(c::PAD_REGISTER|c::PAD_LIST)};
 let closed=STATE.with(|s|s.mode==0);
 let mut e=(if closed{keys}else{keyboard()|mouse_events()})|(pad&!(c::PAD_REGISTER|c::PAD_LIST|c::MENU_OPEN));
 // While Skill-Simulator's window is open our register shortcut acts on its
 // cursor, and the list shortcut does nothing.
 let sim=crate::sksim::frame();crate::game_menu::set_other_open(sim.is_some());
 let engaged=guest_frame(if closed{sim}else{None},e&c::OPEN_REGISTER!=0,pad_allowed);
 if engaged{e&=!(c::OPEN_REGISTER|c::OPEN_LIST);}else{if pad&c::PAD_REGISTER!=0{e|=c::OPEN_REGISTER}if pad&c::PAD_LIST!=0{e|=c::OPEN_LIST}
  // The menu entry is a decision, not a pad shortcut: no keyboard guard.
  if pending&c::MENU_OPEN!=0{
   // In the 'Mods' panel: open the chosen mod. On the Hunter's Notes row: one
   // participant opens directly, more open the game's own third panel.
   let target=crate::game_menu::panel_choice().or_else(||match crate::mod_menu_model::pick(&crate::mod_menu::entries()){
    Some(crate::mod_menu_model::Pick::Open(a))=>Some(a),
    Some(crate::mod_menu_model::Pick::List)=>{crate::game_menu::request_mods_panel();None}
    None=>None,
   });
   if let Some(a)=target{if crate::mod_menu::is_ours(a){e|=c::OPEN_LIST}else{crate::mod_menu::request(a);}}
  }
  // Another mod hosting the shared entry asked us to open.
  if crate::mod_menu::take_request(){e|=c::OPEN_LIST}}
 events(e);
 let open=STATE.with(|s|s.mode!=0);crate::kbm::set_free(open);if !open{crate::game_menu::set_help(None);}if open&&crate::button_prompts::is_recomp(){mouse_shield();}
 if !open{clear_hits();unsafe{*core::ptr::addr_of_mut!(HOVER)=None;}}
 if !STATE.with(|s|s.mode!=0||s.at_smithy()||crate::game_menu::visible(s.base)||!s.notices.is_empty()||crate::native_badges::available()){return}
 // Cache one valid draw list for the frame; never pass a null list to cimgui.
 let Some(cv)=Canvas::new() else{return};
 unsafe{*core::ptr::addr_of_mut!(HIT_VIEW)=(cv.ox,cv.oy,cv.scale);}clear_hits();
 STATE.with(|s|{
  if !s.installed||!s.loaded||!s.game_ready{return}crate::icons::set_layout(s.list.layout);let lang=s.list.language;
  if s.mode==0{
   // KatSmithy's material dialog covers the smithy: no pins or bars over it.
   if !engaged&&!crate::katsmithy::tree_dialog(s.base)&&let Some(view)=crate::hud_layout::right_top(imgui::get_io().DisplaySize.x,imgui::get_io().DisplaySize.y,crate::hud_layout::current(s.base)){let native_cv=Canvas{scale:view.scale,ox:view.ox,oy:view.oy,draw:cv.draw};draw_pins(&native_cv,s);
   // KatSmithy's weapon detail panel ends lower; lift both bars clear of its status box there.
   let lift=if crate::katsmithy::weapon_view(s.base).is_some(){36.}else{0.};
   // The game menu has our Hunter's Notes entry: its list bar is not needed there.
   if s.at_smithy()||(crate::game_menu::visible(s.base)&&!crate::game_menu::entry()){let y=if crate::game_menu::visible(s.base){398.}else{496.-lift};native_cv.shortcut_bar(1102.,y,s.list.pad_bindings[0],s.list.key_bindings[0],l::TITLE[lang]);}if let Some(r)=s.active(){let registered=s.list.registered(r).is_some();native_cv.shortcut_bar(1102.,528.-lift,s.list.pad_bindings[1],s.list.key_bindings[1],if registered{l::QUICK_REMOVE[lang]}else{l::REGISTER[lang]});}}


   for (slot,n) in s.notices.visible(){draw_notice(&cv,n.recipe,n.progress,lang,crate::presentation::NOTICE_DURATION-n.seconds,slot);}
   s.notices.advance(imgui::get_io().DeltaTime);return
  }
  s.material_cache.refresh(&s.list,&s.stock,s.stock_ready,g::PROVISIONS);
  // The quick-removal question still shows the pins; they go once it is confirmed.
  if s.mode==8{if let Some(view)=crate::hud_layout::right_top(imgui::get_io().DisplaySize.x,imgui::get_io().DisplaySize.y,crate::hud_layout::current(s.base)){draw_pins(&Canvas{scale:view.scale,ox:view.ox,oy:view.oy,draw:cv.draw},s);}confirmation(&cv,s);return}
  if s.mode==7{draw_notice_demo(&cv,s);return}
  if s.preview&&s.mode==6{if let Some(r)=s.frozen{draw_notice(&cv,r,None,lang,s.preview_clock,0);}s.preview_clock+=imgui::get_io().DeltaTime.clamp(0.,0.1);if s.preview_clock>crate::presentation::NOTICE_DURATION+0.6{s.preview_clock=0.;}cv.text(350.,40.,600.,18.,GOLD,&format!("{}    L / R  {}/8   B / Esc",l::PREVIEW[lang],s.preview_panel+1));return}
  let registering=matches!(s.mode,1|3|5);cv.frame(128.,52.,820.,546.);crate::skill_view::open();
  cv.heading(146.,82.,782.,if registering{l::REGISTER_TITLE[lang]}else{l::TITLE[lang]});
  if s.preview{cv.text(350.,35.,600.,17.,GOLD,&format!("{}    L / R  {}/8   B / Esc",l::PREVIEW[lang],s.preview_panel+1));}
  if registering{cv.section(146.,123.,320.,l::TARGET[lang]);cv.rect(146.,154.,320.,43.,INSET);cv.icon_plate(154.,160.,31.);if let Some(r)=s.frozen{cv.gear(r,156.,162.,27.);cv.box_text(197.,154.,258.,43.,BODY,WHITE,&name(r,lang),false);}}
  else{
   for (tab,x,w,label) in [(0,185.,341.,l::LIST[lang]),(1,531.,358.,l::TOTAL[lang])]{cv.hit(x,126.,w,35.,Hit::Tab(tab));let here=s.tab==tab;let hover=unsafe{*core::ptr::addr_of!(HOVER)}==Some(Hit::Tab(tab));if here{cv.draw.add_rect_filled(cv.p(x,128.),cv.p(x+w,158.),GOLD,6.*cv.scale);}cv.rect(x,159.,w,1.,RULE);cv.box_text(x+8.,126.,w-16.,33.,24.,if here{0xff26221b}else if hover{WHITE}else{MUTED},label,true);}
   cv.hit(144.,126.,36.,36.,Hit::Event(c::TAB_PREV));cv.hit(893.,126.,36.,36.,Hit::Event(c::TAB_NEXT));cv.icon(crate::icons::L,148.,130.,28.,WHITE);cv.icon(crate::icons::R,897.,130.,28.,WHITE);
  }
  let selected=selected_recipe(s);
  if s.tab==0{
   // The registration screen has the target box above: its list starts lower.
   let (list_top,row0,step,pager_y)=if registering{(205.,236.,40.,560.)}else{(168.,200.,42.,550.)};
   cv.section(146.,list_top,320.,l::TITLE[lang]);let start=s.selected/8*8;
   for i in start..start+8{let y=row0+(i-start) as f32*step;let focus=i==s.selected;
    cv.selection(146.,y,320.,38.,focus&&!s.focus.material,i%2==1);cv.hit(146.,y,320.,38.,Hit::Slot(i));
    if let Some(r)=if focus{selected}else{s.list.slots[i].map(|e|e.recipe)}{let color=if !s.stock_ready{WHITE}else if s.material_cache.ready(i,r).unwrap_or_else(||r.ready(&s.stock,g::PROVISIONS)){GREEN}else{RED};cv.icon_plate(154.,y+4.,30.);cv.gear(r,157.,y+7.,24.);cv.box_text(196.,y,260.,38.,BODY,color,&name(r,lang),false);}else{cv.box_text(159.,y,296.,38.,BODY,MUTED,l::EMPTY[lang],false);}
   }
   cv.pager(306.,pager_y,s.selected/8+1,6);
   cv.line(482.,172.,482.,588.,RULE);
   if let Some(r)=selected{materials(&cv,s,r,501.,168.,426.);let amount=if s.preview&&s.preview_panel==1{crate::equipment::cost(r)}else{let saved=s.list.costs[s.selected];if saved>0{saved}else{crate::equipment::cost(r)}};cv.rect(501.,422.,426.,35.,INSET);cv.rect(501.,422.,3.,35.,GOLD);cv.box_text(514.,422.,265.,35.,23.,GOLD,l::COST[lang],false);let price=if amount>0{format!("{} z",amount)}else{"—".into()};let pw=cv.measure(26.,&price).min(140.);cv.box_text(915.-pw,422.,pw,35.,26.,GOLD,&price,false);}
   else if registering{action_a(&cv,lang);}
  }
  if s.tab==1{aggregate(&cv,s);}else{details(&cv,s,selected);}
  let status=match s.status{2=>l::DUPLICATE[lang],3=>l::SAVE_FAILED[lang],_=>if registering{l::PICK[lang]}else if s.tab==0{if s.focus.material{l::INSPECT_ENTRY[lang]}else{l::PICK_ENTRY[lang]}}else{l::SHARED[lang]}};
  // The message goes into the game's own help bar (its panel and font); our
  // own box only when that bar is not available.
  let message=if registering&&selected.is_some()&&s.status<2{format!("{}\n{}",l::PICK[lang],l::REGISTERED_SLOT[lang])}else{String::from(status)};
  if crate::game_menu::help_available(){crate::game_menu::set_help(Some(&message));}
  else{
   cv.popup(18.,632.,600.,75.);
   if registering&&selected.is_some()&&s.status<2{cv.fit(34.,637.,568.,BODY,WHITE,l::PICK[lang]);cv.fit(34.,668.,568.,23.,BLUE,l::REGISTERED_SLOT[lang]);}
   else{cv.paragraph(34.,640.,568.,61.,BODY,WHITE,status);}
  }
  // One hint line, left of the game's native right-hand prompts.
  let mut hints:Vec<(usize,&str)>=Vec::new();
  if registering{hints.push((crate::icons::A,l::CONFIRM[lang]));}
  else if s.tab==0&&!s.focus.material&&s.sort.is_none(){hints.push((crate::icons::A,l::CONFIRM[lang]));hints.push((crate::icons::X,l::DELETE[lang]));if matches!(s.mode,2|4|9)&&s.list.slots.iter().any(Option::is_some){hints.push((crate::icons::Y,l::CLEAR_ALL[lang]));hints.push((crate::button_prompts::PAD_EXTRA_TILE+2,l::SORT[lang]));}}
  hints.push((crate::icons::B,l::BACK[lang]));
  if s.sort.is_none(){cv.hints(&hints);}
  if matches!(s.mode,3|4|5|9){confirmation(&cv,s);}
  // A modal covers the detail page: no skill panel over it.
  if matches!(s.mode,3|4|5|9)||s.sort.is_some(){crate::skill_view::hide();}
  if s.sort.is_some(){sort_dialog(&cv,s);cv.hints(&[(crate::icons::A,l::CONFIRM[lang]),(crate::icons::B,l::BACK[lang])]);}

 });
 // After the whole window: a linked panel lands on the same foreground list, above it.
 crate::skill_view::overlay();
}
pub fn menu(){
 if !imgui::tree_node(c"Wishlist"){return}
 let font=unsafe{FONT};if let Some(f)=font{imgui::push_font(&f,Some(18.));}
 let (lang,installed,input)=STATE.with(|s|(s.list.language,s.installed,s.input_ready));
 #[cfg(feature="debug-ui")]{
 let capture_label=CString::new(if crate::sound::capturing(){l::STOP_SOUND[lang]}else{l::CAPTURE_SOUND[lang]}).unwrap();
 if imgui::button(&capture_label,ImVec2{x:0.,y:0.}){crate::sound::capture(!crate::sound::capturing());}
 for id in crate::sound::captured(){imgui::text(&format!("SE {}",id));}
 imgui::text(l::SOUND_IDS[lang]);
 }
 let layouts=["Nintendo","Xbox","PlayStation"];let layout=STATE.with(|s|s.list.layout);
 let current=CString::new(layouts[layout]).unwrap();
 if imgui::begin_combo(c"Button layout",&current,ImGuiComboFlags::None){for (i,name) in layouts.iter().enumerate(){let text=CString::new(*name).unwrap();if imgui::selectable(&text,i==layout,ImGuiSelectableFlags::None,ImVec2{x:0.,y:0.}){let mut list=STATE.with(|s|s.list.clone());list.layout=i;apply_preferences(&list);}}imgui::end_combo();}

 if !installed||!input{imgui::text("Unavailable: game signature, input, or save data check failed.");}
 let current=CString::new(l::LANGUAGES[lang]).unwrap();if imgui::begin_combo(c"Language",&current,ImGuiComboFlags::None){for (i,label) in l::LANGUAGES.iter().enumerate(){let text=CString::new(*label).unwrap();if imgui::selectable(&text,i==lang,ImGuiSelectableFlags::None,ImVec2{x:0.,y:0.}){let mut list=STATE.with(|s|s.list.clone());list.language=i;apply_preferences(&list);}}imgui::end_combo();}
 let scale=STATE.with(|s|s.list.ui_scale);let current=CString::new(format!("{}%",scale)).unwrap();
 if imgui::begin_combo(c"UI scale",&current,ImGuiComboFlags::None){for value in [50,60,70,80,90,100]{let text=CString::new(format!("{}%",value)).unwrap();if imgui::selectable(&text,value==scale,ImGuiSelectableFlags::None,ImVec2{x:0.,y:0.}){let mut list=STATE.with(|s|s.list.clone());list.ui_scale=value;apply_preferences(&list);}}imgui::end_combo();}
 let mut logging=STATE.with(|s|s.list.diagnostics);let label=CString::new(l::ENABLE_LOGGING[lang]).unwrap();
 if unsafe{forge::sys::imgui::igCheckbox(label.as_ptr(),&mut logging)}{let mut list=STATE.with(|s|s.list.clone());list.diagnostics=logging;apply_preferences(&list);crate::diagnostics::set_enabled(logging);}
 crate::bindings::ui();
 let defaults=CString::new(l::SETTINGS_DEFAULTS[lang]).unwrap();
 if imgui::button(&defaults,ImVec2{x:0.,y:0.}){let mut list=STATE.with(|s|s.list.clone());list.reset_preferences();apply_preferences(&list);}
 imgui::same_line(0.,-1.);
 imgui::begin_disabled(STATE.with(|s|s.pending_preferences.is_some()));let save=CString::new(l::SETTINGS_SAVE[lang]).unwrap();if imgui::button(&save,ImVec2{x:0.,y:0.}){save_preferences();}imgui::end_disabled();
 match STATE.with(|s|s.settings_notice){1=>imgui::text(l::SETTINGS_PENDING[lang]),2=>imgui::text(l::SETTINGS_SAVED[lang]),3=>imgui::text(l::SETTINGS_FAILED[lang]),4=>imgui::text(l::LANGUAGE_FAILED[lang]),_=>{}}
 #[cfg(feature="debug-ui")]{
 let playing=STATE.with(|s|s.game_ready&&s.loaded&&s.profile.is_some());
 if !playing{imgui::text(l::ENTER_GAME[lang]);}
 imgui::begin_disabled(!playing);
 let open=CString::new(l::TITLE[lang]).unwrap();if imgui::button(&open,ImVec2{x:0.,y:0.}){STATE.with(|s|s.open(false));}
 let test_case=STATE.with(|s|s.notice_test_case);let scenario_label=CString::new(l::TEST_CASE[lang]).unwrap();let current=CString::new(l::NOTICE_SCENARIOS[test_case][lang]).unwrap();
 if imgui::begin_combo(&scenario_label,&current,ImGuiComboFlags::None){for (i,scenario) in l::NOTICE_SCENARIOS.iter().enumerate(){let label=CString::new(scenario[lang]).unwrap();if imgui::selectable(&label,i==test_case,ImGuiSelectableFlags::None,ImVec2{x:0.,y:0.}){STATE.with(|s|s.notice_test_case=i);}}imgui::end_combo();}
 let test=CString::new(l::NOTICE_TEST[lang]).unwrap();if imgui::button(&test,ImVec2{x:0.,y:0.}){STATE.with(|s|{if s.game_ready&&s.loaded&&s.profile.is_some(){start_notice_demo(s);}});}
 #[cfg(feature="debug-ui")]{let preview=CString::new(l::PREVIEW[lang]).unwrap();if imgui::button(&preview,ImVec2{x:0.,y:0.}){STATE.with(|s|{if !s.game_ready||!s.loaded||s.profile.is_none(){return}sound::cue(Cue::Open);s.mode=2;s.preview=true;s.preview_panel=0;s.preview_clock=0.;s.aggregate=crate::interaction::AggregateFocus::default();s.detail_page=0;s.focus=crate::interaction::Focus::default();s.frozen=g::RECIPES.iter().find(|r|r.kind==2&&r.id==4&&r.level==0).copied().or_else(||g::RECIPES.first().copied());s.selected=0;});}}
 imgui::end_disabled();
 }
 if font.is_some(){imgui::pop_font()}imgui::tree_pop();
}
