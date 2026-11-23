//! Native right/top HUD anchor from the active render picture, without host files.
/// Render width/height and the native HUD's virtual reference width.
#[derive(Clone,Copy,Debug)]
pub struct HudSize{pub render_width:u32,pub render_height:u32,pub ui_width:u32}
pub fn right_top(width:f32,height:f32,hud:Option<HudSize>)->Option<crate::presentation::Viewport>{
 crate::presentation::viewport(width,height,100)?;
 if let Some(HudSize{render_width:rw,render_height:rh,ui_width:uw})=hud{
  if (64..=16384).contains(&rw)&&(64..=16384).contains(&rh)&&(1280..=7200).contains(&uw){
   let aspect=rw as f32/rh as f32;
   let ph=height.min(width/aspect);let pw=ph*aspect;
   let scale=(ph/720.).min(pw/1280.);
   return Some(crate::presentation::Viewport{scale,ox:(width+uw as f32*scale)*0.5-1280.*scale,oy:(height-ph)*0.5})
  }
 }
 // Native 16:9 HUD fallback while the renderer is not ready.
 let scale=(width/1280.).min(height/720.);
 if width/height>=16./9.-0.01{Some(crate::presentation::Viewport{scale,ox:(width-1280.*scale)*0.5,oy:(height-720.*scale)*0.5})}
 else{Some(crate::presentation::Viewport{scale:width/1280.,ox:0.,oy:(height-height.min(width/1.6))*0.5})}
}
/// Our windows' viewport: the 1280x720 canvas fitted into the game's picture
/// (the render aspect may differ from the host window, e.g. 16:10 forced in
/// a 16:9 window), at the user's UI scale. Without renderer data: the window.
pub fn picture(width:f32,height:f32,percent:u16,hud:Option<HudSize>)->Option<crate::presentation::Viewport>{
 crate::presentation::viewport(width,height,100)?;
 let (pw,ph)=match hud{
  Some(HudSize{render_width:rw,render_height:rh,..}) if (64..=16384).contains(&rw)&&(64..=16384).contains(&rh)=>{let aspect=rw as f32/rh as f32;let ph=height.min(width/aspect);(ph*aspect,ph)}
  _=>(width,height),
 };
 let scale=(pw/1280.).min(ph/720.)*percent.clamp(50,100) as f32/100.;
 Some(crate::presentation::Viewport{scale,ox:(width-1280.*scale)*0.5,oy:(height-720.*scale)*0.5})
}
/// A Recomp mouse position (console pixels, 1280x720) in display pixels.
/// Recomp maps the pointer over the game's picture, not the host window, and
/// its x over the HUD's width only when the HUD is narrower than the picture
/// (16:9 HUD on 21:9). Same mapping as Skill-Simulator's picture.rs Area::mouse.
pub fn mouse(x:f32,y:f32,width:f32,height:f32,hud:Option<HudSize>)->(f32,f32){
 let whole=(x*width/1280.,y*height/720.);
 let Some(HudSize{render_width:rw,render_height:rh,ui_width:uw})=hud else{return whole};
 if !(64..=16384).contains(&rw)||!(64..=16384).contains(&rh){return whole}
 let aspect=rw as f32/rh as f32;let ph=height.min(width/aspect);let pw=ph*aspect;
 let span=uw as f32/720.;let canvas=if (1280..=16384).contains(&uw)&&span<aspect{span/aspect}else{1.};
 let u=x/1280.*canvas+(1.-canvas)*0.5;
 ((width-pw)*0.5+u*pw,(height-ph)*0.5+y/720.*ph)
}
#[cfg(target_arch="arm")]
pub fn current(base:u32)->Option<HudSize>{
 if !crate::native::valid(base)||base.checked_add(0x212699c).is_none(){return None}
 let render=unsafe{crate::native::rd(base+0x2126998)};
 if !crate::native::valid(render)||render&3!=0||render.checked_add(0x44bacc).is_none(){crate::diagnostics::event(13,"HUD renderer unavailable: fallback used",[0,render,0,0]);return None}
 let signature=unsafe{crate::native::rd(render)};if signature!=base+0x1754a58{crate::diagnostics::event(13,"HUD renderer signature mismatch: fallback used",[1,signature,base+0x1754a58,0]);return None}
 let hud=unsafe{HudSize{render_width:crate::native::rd(render+0x44bac0),render_height:crate::native::rd(render+0x44bac4),ui_width:crate::native::rd(render+0x44bac8)}};
 crate::diagnostics::event(13,"HUD render width / height / UI reference width",[2,hud.render_width,hud.render_height,hud.ui_width]);Some(hud)
}
#[cfg(not(target_arch="arm"))]
pub fn current(_:u32)->Option<HudSize>{None}

/// Mirror the native right anchor for content attached to the left HUD panel.
pub fn left_top(width:f32,height:f32,hud:Option<HudSize>)->Option<crate::presentation::Viewport>{
 let mut v=right_top(width,height,hud)?;v.ox=width-1280.*v.scale-v.ox;Some(v)
}

/// Content attached to a bottom-anchored game panel on the left (the quest counter's
/// detail). The game's UI space is 1280 wide and as tall as the picture allows
/// (800 at 16:10); such panels keep their distance to the picture's bottom, so on
/// pictures taller than 16:9 they sit lower than the 720-unit layout says.
pub fn left_bottom(width:f32,height:f32,hud:Option<HudSize>)->Option<crate::presentation::Viewport>{
 let mut v=left_top(width,height,hud)?;
 if let Some(HudSize{render_width:rw,render_height:rh,..})=hud{
  if (64..=16384).contains(&rw)&&(64..=16384).contains(&rh){
   let aspect=rw as f32/rh as f32;
   if aspect<16./9.-0.01{let ph=height.min(width/aspect);v.oy=(height-ph)*0.5+ph-720.*v.scale;}
  }
 }
 Some(v)
}
