#[cfg(feature="keyboard-input")]
use forge::input::{Input,Key};
use alloc::{vec::Vec,format,string::String,ffi::CString};
use forge::{imgui,sys::imgui::{ImVec2,ImDrawFlags,ImGuiComboFlags,ImGuiSelectableFlags}};
use crate::{STATE,State,model::{Recipe,List},locales as l,generated as g,controller as c,storage,sound::{self,Cue}};
pub(crate) static mut FONT:Option<imgui::ImFont>=None;
// Steel workshop palette; native button bars below deliberately keep their own art.
const WHITE:u32=0xffebece9;const GOLD:u32=0xff75bbe9;const BLUE:u32=0xffd3c493;const RED:u32=0xff8da1ff;const GREEN:u32=0xffaccea4;const MUTED:u32=0xffc0c0b7;
const PANEL:u32=0xf4282620;const INSET:u32=0xee1d1a15;const RAISED:u32=0xff373229;const EDGE:u32=0xff807d70;const RULE:u32=0xff575347;
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
 fn new()->Option<Self>{let draw=imgui::DrawList::foreground();if draw.raw().is_null(){return None}let sz=imgui::get_io().DisplaySize;let view=crate::presentation::viewport(sz.x,sz.y,STATE.with(|s|s.list.ui_scale))?;Some(Self{scale:view.scale,ox:view.ox,oy:view.oy,draw})}
 fn pixels(&self,size:f32)->f32{((size*self.scale+0.5) as u32).max(1) as f32}
 fn p(&self,x:f32,y:f32)->ImVec2{ImVec2{x:self.ox+x*self.scale,y:self.oy+y*self.scale}}
 fn rect(&self,x:f32,y:f32,w:f32,h:f32,color:u32){self.draw.add_rect_filled(self.p(x,y),self.p(x+w,y+h),color,0.);}
 fn line(&self,x:f32,y:f32,x2:f32,y2:f32,color:u32){self.draw.add_line(self.p(x,y),self.p(x2,y2),color,self.scale);}
 fn gradient(&self,x:f32,y:f32,w:f32,h:f32,left:u32,right:u32){self.draw.add_rect_filled_multicolor(self.p(x,y),self.p(x+w,y+h),left,right,right,left);}
 fn metal(&self,x:f32,y:f32,w:f32,h:f32,color:u32){
  let cut=8_f32.min(h*0.2);let points=[self.p(x+cut,y),self.p(x+w-cut,y),self.p(x+w,y+cut),self.p(x+w,y+h-cut),self.p(x+w-cut,y+h),self.p(x+cut,y+h),self.p(x,y+h-cut),self.p(x,y+cut)];
  // Recomp mirrors an ImVec2* as one object, so never pass point arrays to its
  // polygon APIs. These value-only primitives preserve the chamfered silhouette.
  self.rect(x+cut,y,w-2.*cut,h,color);self.rect(x,y+cut,cut,h-2.*cut,color);self.rect(x+w-cut,y+cut,cut,h-2.*cut,color);
  for (a,b,c) in [(0,7,self.p(x+cut,y+cut)),(1,2,self.p(x+w-cut,y+cut)),(3,4,self.p(x+w-cut,y+h-cut)),(5,6,self.p(x+cut,y+h-cut))]{self.draw.add_triangle_filled(points[a],points[b],c,color);}
  for i in 0..points.len(){self.draw.add_line(points[i],points[(i+1)%points.len()],EDGE,self.scale);}
  self.line(x+cut+1.,y+2.,x+w-cut-1.,y+2.,0x506f7b7e);self.line(x+cut,y+h-2.,x+w-cut,y+h-2.,0xaa100e0c);
 }
 fn frame(&self,x:f32,y:f32,w:f32,h:f32){
  self.metal(x+2.,y+4.,w,h,0x50000000);self.metal(x,y,w,h,PANEL);
  for px in [x+12.,x+w-15.]{self.rect(px,y+9.,3.,3.,0xffa7a49a);self.rect(px+1.,y+10.,1.,1.,INSET);}
 }
 fn heading(&self,x:f32,y:f32,w:f32,text:&str){
  self.gradient(x,y,w,39.,0xff302f2a,0xff282620);self.rect(x,y+38.,w,2.,GOLD);self.box_text(x+12.,y,w-24.,38.,28.,WHITE,text,false);
 }
 fn section(&self,x:f32,y:f32,w:f32,text:&str){
  let text_w=self.measure(22.,text).min(w-8.);self.fit(x,y,w,22.,GOLD,text);
  if text_w+18.<w{self.line(x+text_w+12.,y+18.,x+w,y+18.,RULE);}
 }
 fn selection(&self,x:f32,y:f32,w:f32,h:f32,selected:bool,alternate:bool){
  if selected{self.gradient(x,y,w,h,0xff454a4b,0xff514b3f);self.rect(x,y,3.,h,GOLD);self.line(x+3.,y+h-1.,x+w,y+h-1.,0xff5a8ba8);}
  else{self.rect(x,y,w,h,if alternate{0xff302d25}else{RAISED});self.rect(x,y,3.,h,0xff676457);}
 }
 fn icon_plate(&self,x:f32,y:f32,size:f32){self.rect(x,y,size,size,0xff1d1810);self.draw.add_rect(self.p(x,y),self.p(x+size,y+size),RULE,1.,self.scale,ImDrawFlags::None);}
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
 fn icon(&self,tile:usize,x:f32,y:f32,size:f32,tint:u32){let p=self.p(x,y);crate::icons::draw(self.draw,tile,p.x,p.y,self.pixels(size),self.pixels(size),tint,false);}
 fn gear(&self,r:Recipe,x:f32,y:f32,size:f32){self.gear_alpha(r,x,y,size,1.);}
 fn gear_alpha(&self,r:Recipe,x:f32,y:f32,size:f32,alpha:f32){if r.kind==25{let (tile,tint)=crate::icons::material(r.id);self.icon(tile,x,y,size,crate::presentation::alpha(tint,alpha));}else{self.icon(crate::icons::equipment(r.kind),x,y,size,crate::presentation::alpha(crate::equipment::icon_tint(r),alpha));}}
 fn sprite(&self,tile:usize,x:f32,y:f32,w:f32,h:f32,tint:u32,flip:bool){let p=self.p(x,y);crate::icons::draw(self.draw,tile,p.x,p.y,w*self.scale,h*self.scale,tint,flip);}
 fn shortcut_bar(&self,x:f32,y:f32,chord:u32,label:&str){
  self.bar(x,y,160.,usize::MAX,"");let count=chord.count_ones() as f32;let width=(count*18.+(count-1.).max(0.)*5.).min(90.);let size=18_f32.min((width-(count-1.).max(0.)*5.)/count.max(1.));let mut px=x+8.;
  for bit in 0..24{if chord&(1<<bit)==0{continue}if px>x+8.{self.box_text(px-5.,y,5.,28.,13.,WHITE,"+",true);}
   let tile=match bit{0=>Some(crate::icons::A),1=>Some(crate::icons::B),2=>Some(crate::icons::X),3=>Some(crate::icons::Y),4=>Some(crate::icons::L3),5=>Some(crate::icons::R3),6=>Some(crate::icons::L),7=>Some(crate::icons::R),8=>Some(crate::icons::ZL),9=>Some(crate::icons::ZR),12=>Some(crate::icons::DPAD_LEFT),14=>Some(crate::icons::DPAD),_=>None};
   if let Some(tile)=tile{self.icon(tile,px,y+(28.-size)*0.5,size,WHITE);}else{let text=match bit{10=>"+",11=>"−",13=>"↑",15=>"↓",16=>"L←",17=>"L↑",18=>"L→",19=>"L↓",20=>"R←",21=>"R↑",22=>"R→",_=>"R↓"};self.rect(px,y+(28.-size)*0.5,size,size,RAISED);self.box_text(px,y,size,28.,16.,WHITE,text,true);}px+=size+5.;
  }self.box_text(x+width+10.,y,142.-width,28.,24.,WHITE,&label,false);
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
 fn bold(&self,x:f32,y:f32,w:f32,size:f32,color:u32,text:&str){self.text(x,y,w,size,color,text);self.text(x+1./self.scale,y,w,size,color,text);}
 fn plus(&self,x:f32,y:f32,w:f32,h:f32){
  self.box_text(x,y,w,h,18.,WHITE,"+",true);
 }
 fn pager(&self,cx:f32,y:f32,page:usize,pages:usize){
  let text=format!("◀ {} / {} ▶",page,pages.max(1));let width=self.measure(24.,&text)+4.;let left=cx-width*0.5;let icon=25.;let gap=16.;
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
   if events&c::DETAIL!=0{s.detail_page=1-s.detail_page;sound::cue(Cue::Tab);}
   if events&(c::PREV|c::NEXT|c::TAB_PREV|c::TAB_NEXT)!=0{
    s.preview_panel=(s.preview_panel+if events&(c::NEXT|c::TAB_NEXT)!=0{1}else{7})%8;s.mode=match s.preview_panel{0=>2,1|2=>1,3=>5,4=>6,5=>3,6=>4,_=>2};s.focus.material=s.preview_panel==7;s.focus.index=0;s.preview_clock=0.;
    sound::cue(if events&(c::TAB_PREV|c::TAB_NEXT)!=0{Cue::Tab}else{Cue::Page});
   }return 0
  }
  if s.mode==5{if events&(c::ACCEPT|c::BACK)!=0{s.mode=0;sound::cue(Cue::Close);}return 0}
  if events&c::BACK!=0{
   sound::cue(Cue::Close);if s.focus.back(){return 0}
   match s.mode{3=>s.mode=1,4=>s.mode=2,_=>{s.mode=0;s.preview=false}}return 0
  }
  if matches!(s.mode,3|4|8){
   if events&(c::PREV|c::NEXT|c::UP|c::DOWN)!=0{s.focus.yes=!s.focus.yes;sound::cue(Cue::Cursor);}
   if events&c::ACCEPT!=0{if s.focus.yes{return if s.mode==3{1}else{2}}s.mode=match s.mode{3=>1,8=>0,_=>2};sound::cue(Cue::Close);}return 0
  }
  if s.tab==0&&events&c::DETAIL!=0{s.detail_page=1-s.detail_page;sound::cue(Cue::Tab);}
  if s.mode==2&&events&(c::TAB|c::TAB_PREV|c::TAB_NEXT)!=0{s.tab=1-s.tab;s.aggregate=crate::interaction::AggregateFocus::default();s.focus.material=false;s.focus.index=0;sound::cue(Cue::Tab);return 0}
  if s.mode==2&&s.tab==1{
   s.material_cache.refresh(&s.list,&s.stock,s.stock_ready,g::PROVISIONS);let totals=s.material_cache.totals();s.aggregate.material=s.aggregate.material.min(totals.len().saturating_sub(1));
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
 });if action==3{apply_sort()}else if action!=0{commit(action==1)}
}
#[cfg(feature="keyboard-input")]
fn keyboard()->u32{let mut e=0;for (key,event) in [(Key::Enter,c::ACCEPT),(Key::Escape,c::BACK),(Key::Up,c::UP),(Key::Down,c::DOWN),(Key::Left,c::PREV),(Key::Right,c::NEXT),(Key::PageUp,c::PREV),(Key::PageDown,c::NEXT),(Key::Delete,c::DELETE),(Key::Tab,c::TAB),(Key::Q,c::DETAIL),(Key::E,c::DETAIL)]{if Input::is_key_pressed(key){e|=event}}e}
#[cfg(not(feature="keyboard-input"))]
fn keyboard()->u32{0}
fn materials(cv:&Canvas,s:&State,r:Recipe,x:f32,y:f32,w:f32){
 let lang=s.list.language;let totals=r.totals();cv.section(x,y,w,l::MATERIALS[lang]);let mut py=y+40.;let step=if r.group>0&&totals.len()>3{35.}else{46.};
 for (i,m) in totals.iter().enumerate(){let owned=s.stock[m.id as usize];let good=s.stock_ready&&owned>=m.count;
  if s.focus.material&&s.focus.index==i{cv.selection(x,py-3.,w,39_f32.min(step-3.),true,false);}
  let height=39_f32.min(step-3.);let row_y=py-3.;
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
  cv.rect(x,465.,w,140.,INSET);cv.line(x,465.,x+w,465.,RULE);let (tile,tint)=crate::icons::material(m.id);cv.icon(tile,x+10.,507.,44.,tint);
  cv.fit(x+10.,471.,w-20.,26.,GOLD,&item(m.id,lang));
  if let Some(data)=crate::icons::ITEMS.get(m.id as usize){cv.wrap(x+8.,555.,65.,20.,crate::presentation::rarity(data.2 as i16),&format!("{} {}",l::RARITY[lang],data.2));}
  cv.paragraph(x+76.,506.,w-88.,94.,BODY,WHITE,&crate::name_data::material_description(m.id,lang));


 }}
}
fn aggregate(cv:&Canvas,s:&State)->Option<Recipe>{
 let lang=s.list.language;let totals=s.material_cache.totals();let cursor=s.aggregate.material.min(totals.len().saturating_sub(1));
 cv.line(542.,172.,542.,606.,RULE);
 cv.section(146.,168.,380.,l::MATERIALS[lang]);cv.section(561.,168.,367.,l::TITLE[lang]);
 let start=cursor/8*8;
 for (row,m) in totals.iter().skip(start).take(8).enumerate(){
  let y=199.+row as f32*35.;let focus=start+row==cursor;let owned=s.stock[m.id as usize];
  let color=if s.stock_ready&&owned<m.count{RED}else{WHITE};
  if focus{cv.selection(146.,y-1.,380.,34.,true,false);}
  let (tile,tint)=crate::icons::material(m.id);cv.icon(tile,153.,y+3.,27.,tint);
  cv.box_text(188.,y-1.,197.,34.,BODY,color,&item(m.id,lang),false);cv.box_text(391.,y-1.,58.,34.,BODY,color,&format!("× {}",m.count),false);
  cv.icon(crate::icons::CHEST,455.,y+5.,22.,WHITE);cv.box_text(487.,y-1.,38.,34.,BODY,color,&if s.stock_ready{format!("{}",owned)}else{"?".into()},false);
  cv.line(150.,y+33.,526.,y+33.,RULE);
 }
 cv.pager(336.,484.,cursor/8+1,totals.len().div_ceil(8));
 let Some(material)=totals.get(cursor) else{return None};
 // Material details follow the only cursor in this tab.
 cv.rect(146.,514.,380.,95.,INSET);cv.fit(155.,515.,235.,23.,GOLD,&item(material.id,lang));
 let (tile,tint)=crate::icons::material(material.id);cv.icon(tile,155.,549.,32.,tint);
 if let Some(data)=crate::icons::ITEMS.get(material.id as usize){cv.fit(404.,518.,113.,18.,crate::presentation::rarity(data.2 as i16),&format!("{} {}",l::RARITY[lang],data.2));}
 cv.paragraph(197.,545.,320.,60.,23.,WHITE,&crate::name_data::material_description(material.id,lang));
 let related=s.material_cache.related(material.id);let count=related.count_ones() as usize;
 // Passive list: every related entry stays visible, with two columns for large lists.
 let columns=if count>16{2}else{1};let rows=count.div_ceil(columns).max(1);let row_height=(355./rows as f32).min(58.);let column_width=367./columns as f32;let icon_size=(row_height-2.).min(30.);let top=225.;
 for (index,slot) in (0..48).filter(|slot|related&(1u64<<slot)!=0).enumerate(){
  let Some(entry)=s.list.slots[slot] else{continue};let x=561.+(index/rows) as f32*column_width;let y=top+(index%rows) as f32*row_height;let font_size=BODY.min(row_height-2.);let label=name(entry.recipe,lang);
  let text_width=cv.measure(font_size,&label).min(column_width-icon_size-18.);let group=x+(column_width-icon_size-9.-text_width)*0.5;
  cv.gear(entry.recipe,group,y,icon_size);cv.box_text(group+icon_size+9.,y,text_width,icon_size,font_size,WHITE,&label,false);
  if row_height>=42.{cv.line(x+9.,y+icon_size+12.,x+column_width-9.,y+icon_size+12.,RULE);}
 }

 (0..48).find(|slot|related&(1u64<<slot)!=0).and_then(|slot|s.list.slots[slot]).map(|e|e.recipe)

}
fn draw_notice(cv:&Canvas,r:Recipe,lang:usize,elapsed:f32,slot:usize){
 let title=format!("{} \"{}\"",l::TITLE[lang],name(r,lang));let body=l::NOTICE[lang];
 let title_size=(23.*842./cv.measure(23.,&title).max(842.)).max(14.);let body_size=25.;let title_w=cv.measure(title_size,&title);let body_w=cv.measure(body_size,body);
 let w=(title_w+40.).max(body_w).min(876.)+32.;
 let right=(imgui::get_io().DisplaySize.x-cv.ox)/cv.scale-18.;let opacity=crate::presentation::notice_opacity(elapsed);let x=right-w;
 // Quest maps occupy the upper-right until ~350; keep the whole toast below them.
 let y=crate::notifications::top(slot);cv.rect(x,y,w,80.,crate::presentation::alpha(0xe0282620,opacity));
 cv.rect(x,y,3.,80.,crate::presentation::alpha(GOLD,opacity));cv.line(x+3.,y,x+w,y,crate::presentation::alpha(EDGE,opacity));
 let group_x=x+(w-title_w-40.)*0.5;cv.gear_alpha(r,group_x,y+9.,30.,opacity);
 cv.box_bold(group_x+40.,y+7.,title_w+1.,34.,title_size,crate::presentation::alpha(GOLD,opacity),&title,true);
 cv.box_bold(x+16.,y+38.,w-32.,34.,body_size,crate::presentation::alpha(0xff35a5ff,opacity),body,true);

}
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
 for (slot,n) in demo.queue.visible(){draw_notice(cv,n.recipe,lang,crate::presentation::NOTICE_DURATION-n.seconds,slot);}
 demo.advance(imgui::get_io().DeltaTime);
}
fn details(cv:&Canvas,s:&State,r:Option<Recipe>){
 let lang=s.list.language;cv.frame(972.,69.,290.,551.);
 cv.heading(984.,82.,266.,l::DETAILS[lang]);
 cv.icon(crate::icons::ZL,985.,132.,26.,WHITE);cv.center(1023.,129.,188.,25.,WHITE,&format!("{} / 2",s.detail_page+1));cv.icon(crate::icons::ZR,1222.,132.,26.,WHITE);
 let Some(r)=r else{cv.box_text(984.,69.,266.,551.,26.,MUTED,l::NO_EQUIPMENT[lang],true);return};
 cv.rect(984.,169.,266.,45.,INSET);cv.rect(984.,169.,3.,45.,GOLD);cv.icon_plate(995.,176.,31.);cv.gear(r,997.,178.,27.);cv.box_text(1036.,169.,205.,45.,BODY,GOLD,&name(r,lang),false);
 cv.rect(984.,224.,266.,382.,INSET);
 if s.detail_page==1{
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
fn confirmation(cv:&Canvas,s:&State){let lang=s.list.language;cv.draw.add_rect_filled(ImVec2{x:0.,y:0.},imgui::get_io().DisplaySize,0x90000000,0.);cv.frame(400.,307.,480.,176.);
   let text=if s.mode==5{l::DONE[lang]}else if s.mode==3{l::REPLACE_QUESTION[lang]}else{l::DELETE_QUESTION[lang]};
   for (i,line) in text.lines().enumerate(){cv.center(418.,327.+i as f32*32.,444.,26.,WHITE,line);}
 if s.mode==5{cv.selection(564.,424.,152.,39.,true,false);cv.box_text(564.,424.,152.,39.,26.,WHITE,l::CLOSE[lang],true);}
 else{for (yes,x,label) in [(true,460.,l::YES[lang]),(false,668.,l::NO[lang])]{cv.selection(x,424.,152.,39.,s.focus.yes==yes,false);cv.box_text(x,424.,152.,39.,26.,WHITE,label,true);}}
  }
fn sort_dialog(cv:&Canvas,s:&State){
 let Some(d)=s.sort else{return};let lang=s.list.language;
 cv.draw.add_rect_filled(ImVec2{x:0.,y:0.},imgui::get_io().DisplaySize,if d.secondary{0xa0000000}else{0x70000000},0.);
 let (x,w)=if d.secondary{(280.,720.)}else{(400.,480.)};cv.frame(x,170.,w,420.);cv.heading(x+18.,183.,w-36.,l::SORT[lang]);
 let left=x+18.;let column=if d.secondary{326.}else{w-36.};cv.section(left,230.,column,l::SORT_PRIMARY[lang]);
 for (i,text) in l::SORT_CRITERIA.iter().enumerate(){let y=275.+i as f32*62.;cv.selection(left,y,column,48.,d.criterion==i,false);let icon_x=left+10.;let icon_y=y+11.;
  if i==0{cv.icon(SORT_ORDER_ICON,icon_x,icon_y,26.,GOLD);}else{
   cv.icon(SORT_STOCK_ICON,icon_x,icon_y,26.,if i==1{GREEN}else{RED});
   let (cx,cy)=(icon_x+20.,icon_y+20.);cv.rect(cx-5.,cy-5.,12.,12.,INSET);
   if i==1{cv.line(cx-4.,cy,cx-1.,cy+4.,GREEN);cv.line(cx-1.,cy+4.,cx+6.,cy-4.,GREEN);}else{cv.line(cx-3.,cy-3.,cx+4.,cy+4.,RED);cv.line(cx-3.,cy+4.,cx+4.,cy-3.,RED);}
  }cv.box_text(left+48.,y,column-58.,48.,BODY,WHITE,text[lang],false);}
 if d.secondary{
  // Only the right column is active; shade the complete previous-stage panel.
  cv.rect(left,224.,column,245.,0x99000000);let right=x+382.;cv.section(right,230.,w-400.,l::SORT_SECONDARY[lang]);
  for (i,text) in l::SORT_CATEGORIES.iter().enumerate(){let y=265.+i as f32*44.;cv.selection(right,y,w-400.,38.,d.category==i,false);let tile=match i{1=>crate::icons::equipment(7),2=>crate::icons::equipment(2),3=>SORT_DECORATION_ICON,4=>crate::icons::equipment(22),_=>crate::icons::equipment(24)};
   if i==0{cv.sprite(crate::icons::SLOT_DASH,right+9.,y+16.,24.,7.,MUTED,false);}else{cv.icon(tile,right+8.,y+6.,26.,if i==3{GOLD}else{WHITE});}
   cv.box_text(right+44.,y,w-454.,38.,24.,WHITE,text[lang],false);}
 }
 if d.message>0{cv.paragraph(x+18.,535.,w-36.,45.,21.,RED,if d.message==1{l::SORT_WAIT[lang]}else{l::SAVE_FAILED[lang]});}
}
pub fn render(){
 let menu=forge::graphics::Graphics::is_menu_visible();let capturing=crate::bindings::capturing();
 let pad_allowed=crate::bindings::pad_shortcuts_allowed();
 let keys=crate::bindings::keyboard_events(menu||capturing);
 if capturing{STATE.with(|s|s.events=0);crate::bindings::capture_overlay(crate::bindings::language(STATE.with(|s|s.list.language)));return}
 if menu{STATE.with(|s|s.events=0);return}
 if !STATE.with(|s|s.installed&&s.loaded&&s.game_ready){return}
 let pending=STATE.with(|s|core::mem::take(&mut s.events));let pad=if pad_allowed{pending}else{0};let mut e=(if STATE.with(|s|s.mode==0){keys}else{keyboard()})|(pad&!(c::PAD_REGISTER|c::PAD_LIST));
 if pad&c::PAD_REGISTER!=0{e|=c::OPEN_REGISTER}if pad&c::PAD_LIST!=0{e|=c::OPEN_LIST}events(e);
 if !STATE.with(|s|s.mode!=0||s.at_smithy()||crate::game_menu::visible(s.base)||!s.notices.is_empty()||crate::native_badges::available()){return}
 // Cache one valid draw list for the frame; never pass a null list to cimgui.
 let Some(cv)=Canvas::new() else{return};
 STATE.with(|s|{
  if !s.installed||!s.loaded||!s.game_ready{return}crate::icons::set_layout(s.list.layout);let lang=s.list.language;
  if s.mode==0{
   if let Some(view)=crate::hud_layout::right_top(imgui::get_io().DisplaySize.x,imgui::get_io().DisplaySize.y,crate::hud_layout::current(s.base)){let native_cv=Canvas{scale:view.scale,ox:view.ox,oy:view.oy,draw:cv.draw};for marker in crate::native_badges::markers(&s.list,s.base){native_cv.pin(marker.x,marker.y);}
   if s.at_smithy()||crate::game_menu::visible(s.base){let y=if crate::game_menu::visible(s.base){398.}else{496.};native_cv.shortcut_bar(1102.,y,s.list.pad_bindings[0],l::TITLE[lang]);}if let Some(r)=s.active(){let registered=s.list.registered(r).is_some();native_cv.shortcut_bar(1102.,528.,s.list.pad_bindings[1],if registered{l::QUICK_REMOVE[lang]}else{l::REGISTER[lang]});}}


   for (slot,n) in s.notices.visible(){draw_notice(&cv,n.recipe,lang,crate::presentation::NOTICE_DURATION-n.seconds,slot);}
   s.notices.advance(imgui::get_io().DeltaTime);return
  }
  s.material_cache.refresh(&s.list,&s.stock,s.stock_ready,g::PROVISIONS);
  if s.mode==8{confirmation(&cv,s);return}
  if s.mode==7{draw_notice_demo(&cv,s);return}
  if s.preview&&s.mode==6{if let Some(r)=s.frozen{draw_notice(&cv,r,lang,s.preview_clock,0);}s.preview_clock+=imgui::get_io().DeltaTime.clamp(0.,0.1);if s.preview_clock>crate::presentation::NOTICE_DURATION+0.6{s.preview_clock=0.;}cv.text(350.,40.,600.,18.,GOLD,&format!("{}    L / R  {}/8   B / Esc",l::PREVIEW[lang],s.preview_panel+1));return}
  let registering=matches!(s.mode,1|3|5);cv.frame(128.,69.,820.,551.);
  cv.heading(146.,82.,782.,if registering{l::REGISTER_TITLE[lang]}else{l::TITLE[lang]});
  if s.preview{cv.text(350.,35.,600.,17.,GOLD,&format!("{}    L / R  {}/8   B / Esc",l::PREVIEW[lang],s.preview_panel+1));}
  if registering{cv.section(146.,123.,320.,l::TARGET[lang]);cv.rect(146.,154.,320.,43.,INSET);cv.icon_plate(154.,160.,31.);if let Some(r)=s.frozen{cv.gear(r,156.,162.,27.);cv.box_text(197.,154.,258.,43.,BODY,WHITE,&name(r,lang),false);}}
  else{
   for (tab,x,w,label) in [(0,185.,341.,l::LIST[lang]),(1,531.,358.,l::TOTAL[lang])]{cv.rect(x,126.,w,35.,if s.tab==tab{0xff2e302d}else{INSET});cv.rect(x,159.,w,2.,if s.tab==tab{GOLD}else{RULE});cv.box_text(x+8.,126.,w-16.,33.,24.,if s.tab==tab{GOLD}else{MUTED},label,true);}
   cv.icon(crate::icons::L,148.,130.,28.,WHITE);cv.icon(crate::icons::R,897.,130.,28.,WHITE);
  }
  let selected=s.list.slots[s.selected].map(|e|e.recipe);let selected=if s.preview&&matches!(s.preview_panel,1|7){s.frozen}else if s.preview&&s.preview_panel==2{None}else{selected};
  if s.tab==0{
   cv.section(146.,198.,320.,l::TITLE[lang]);let start=s.selected/8*8;
   for i in start..start+8{let y=230.+(i-start) as f32*42.;let focus=i==s.selected;
    cv.selection(146.,y,320.,38.,focus&&!s.focus.material,i%2==1);
    if let Some(r)=if focus{selected}else{s.list.slots[i].map(|e|e.recipe)}{let color=if !s.stock_ready{WHITE}else if s.material_cache.ready(i,r).unwrap_or_else(||r.ready(&s.stock,g::PROVISIONS)){GREEN}else{RED};cv.icon_plate(154.,y+4.,30.);cv.gear(r,157.,y+7.,24.);cv.box_text(196.,y,260.,38.,BODY,color,&name(r,lang),false);}else{cv.box_text(159.,y,296.,38.,BODY,MUTED,l::EMPTY[lang],false);}
   }
   cv.pager(306.,580.,s.selected/8+1,6);
   cv.line(482.,172.,482.,605.,RULE);
   if let Some(r)=selected{materials(&cv,s,r,501.,168.,426.);let amount=if s.preview&&s.preview_panel==1{crate::equipment::cost(r)}else{let saved=s.list.costs[s.selected];if saved>0{saved}else{crate::equipment::cost(r)}};cv.rect(501.,422.,426.,35.,INSET);cv.rect(501.,422.,3.,35.,GOLD);cv.box_text(514.,422.,265.,35.,23.,GOLD,l::COST[lang],false);let price=if amount>0{format!("{} z",amount)}else{"—".into()};let pw=cv.measure(26.,&price).min(140.);cv.box_text(915.-pw,422.,pw,35.,26.,GOLD,&price,false);}
   else if registering{action_a(&cv,lang);}
  }
  if s.tab==1{aggregate(&cv,s);}else{details(&cv,s,selected);}
  cv.frame(18.,632.,600.,75.);let status=match s.status{2=>l::DUPLICATE[lang],3=>l::SAVE_FAILED[lang],_=>if registering{l::PICK[lang]}else if s.tab==0{if s.focus.material{l::INSPECT_ENTRY[lang]}else{l::PICK_ENTRY[lang]}}else{l::SHARED[lang]}};
  if registering&&selected.is_some()&&s.status<2{cv.fit(34.,637.,568.,BODY,WHITE,l::PICK[lang]);cv.fit(34.,668.,568.,23.,0xff35a5ff,l::REGISTERED_SLOT[lang]);}
  else{cv.paragraph(34.,640.,568.,61.,BODY,WHITE,status);}
  // Dedicated black prompt bars sit left of the game's native right-hand prompts.
  if registering{cv.bar(925.,642.,160.,crate::icons::A,l::CONFIRM[lang]);cv.bar(925.,676.,160.,crate::icons::B,l::BACK[lang]);}
  else{if s.tab==0&&!s.focus.material&&s.sort.is_none(){cv.bar(755.,642.,160.,crate::icons::A,l::CONFIRM[lang]);cv.bar(925.,642.,160.,crate::icons::X,l::DELETE[lang]);if s.mode==2&&s.list.slots.iter().any(Option::is_some){cv.bar(925.,676.,160.,crate::icons::Y,l::SORT[lang]);}}cv.bar(755.,676.,160.,crate::icons::B,l::BACK[lang]);}
  if matches!(s.mode,3|4|5){confirmation(&cv,s);}
  if s.sort.is_some(){sort_dialog(&cv,s);cv.bar(755.,642.,160.,crate::icons::A,l::CONFIRM[lang]);cv.bar(755.,676.,160.,crate::icons::B,l::BACK[lang]);}

 });
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
