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
