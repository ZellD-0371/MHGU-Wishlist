//! Recomp picture and right/top menu anchors, separate from the plugin's UI scale.
#[derive(Clone,Copy,Default)]
pub struct Settings{pub aspect:f32,pub anchor:u8}
fn number(text:&str)->Option<f32>{
 let mut n=0.;let mut fraction=0.;let mut digits=false;
 for b in text.trim().bytes(){if b==b'.'&&fraction==0.{fraction=1.;continue}if !b.is_ascii_digit(){return None}digits=true;let d=(b-b'0') as f32;if fraction==0.{n=n*10.+d}else{fraction*=10.;n+=d/fraction}}
 (digits&&n.is_finite()).then_some(n)
}
pub fn parse(text:&str)->Settings{
 let mut out=Settings::default();let mut display=false;
 for raw in text.lines(){let line=raw.trim().trim_start_matches('\u{feff}');if line.starts_with('['){display=line.eq_ignore_ascii_case("[Display]");continue}if !display{continue}
  let Some((key,value))=line.split_once('=') else{continue};let value=value.split([';','#']).next().unwrap_or("").trim();
  if key.trim().eq_ignore_ascii_case("UIAnchor"){out.anchor=if value.eq_ignore_ascii_case("Edges"){3}else if value=="21:9"||value=="2"{1}else if value=="32:9"||value=="3"{2}else{0};}
  if key.trim().eq_ignore_ascii_case("AspectRatio"){let value=value.replace('/',":");let ratio=if let Some((a,b))=value.split_once(':'){number(a).zip(number(b)).map(|(a,b)|a/b)}else{number(&value)};if let Some(r)=ratio{if r.is_finite()&&(0.5..=10.).contains(&r){out.aspect=r}}}
 }out
}
pub fn right_top(width:f32,height:f32,settings:Settings)->Option<crate::presentation::Viewport>{
 crate::presentation::viewport(width,height,100)?;
 let mut aspect=if settings.aspect>0.{settings.aspect}else{width/height};
 if aspect<1.6-0.005{aspect=1.6}if (aspect-16./9.).abs()<0.01{aspect=16./9.}
 let pw=width.min(height*aspect);let ph=pw/aspect;let px=(width-pw)*0.5;let py=(height-ph)*0.5;
 if aspect<16./9.{return Some(crate::presentation::Viewport{scale:pw/1280.,ox:px,oy:py})}
 let anchor=match settings.anchor{1=>21./9.,2=>32./9.,3=>aspect,_=>16./9.}.min(aspect);
 let scale=ph/720.;let right=px+(pw+ph*anchor)*0.5;
 Some(crate::presentation::Viewport{scale,ox:right-1280.*scale,oy:py})
}
#[cfg(target_arch="arm")]
pub fn current(dt:f32)->Settings{unsafe{
 use forge::sys::imgui as sys;
 static mut SETTINGS:Settings=Settings{aspect:0.,anchor:0};static mut AGE:f32=2.;
 AGE+=dt.clamp(0.,0.1);if AGE<1.{return SETTINGS}AGE=0.;
 let file=sys::igImFileOpen(c"mhgurecomp.ini".as_ptr(),c"rb".as_ptr());if file.is_null(){return SETTINGS}
 let mut data=[0u8;32768];let len=sys::igImFileRead(data.as_mut_ptr().cast(),1,data.len() as u64,file);sys::igImFileClose(file);
 if len<(data.len() as u64){if let Ok(text)=core::str::from_utf8(&data[..len as usize]){SETTINGS=parse(text);}}
 SETTINGS
}}
#[cfg(not(target_arch="arm"))]
pub fn current(_:f32)->Settings{Settings::default()}
