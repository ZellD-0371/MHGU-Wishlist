// Native GU rarity colours (0x18994B4, read by +560EF0), AABBGGRR.
const RARITY:[u32;11]=[0xffe6e6e6,0xfff292b0,0xff5ed5de,0xff9e8ee8,0xff73c76f,0xfff88e70,0xff5a56da,0xffac9e4c,0xff5d8cee,0xffbd45ff,0xffff007d];
pub fn rarity(value:i16)->u32{value.checked_sub(1).and_then(|v|RARITY.get(v as usize)).copied().unwrap_or(0xffe6e6e6)}
// Fire, water, thunder, ice, dragon. Values and labels use the same element colour.
pub const ELEMENTS:[u32;5]=[0xff6565ff,0xffff9c59,0xff4cefff,0xffffdf9b,0xffe18bc2];
pub const NOTICE_HOLD:f32=5.0;
pub const NOTICE_FADE:f32=0.50;
pub const NOTICE_DURATION:f32=NOTICE_HOLD+NOTICE_FADE;
pub fn alpha(color:u32,opacity:f32)->u32{(color&0x00ffffff)|((((color>>24) as f32*opacity.clamp(0.,1.)) as u32)<<24)}
pub fn notice_opacity(elapsed:f32)->f32{if elapsed<=NOTICE_HOLD{1.}else{(1.-(elapsed-NOTICE_HOLD)/NOTICE_FADE).clamp(0.,1.)}}

/// A single centred 16:9 coordinate space: never stretch X and Y independently.
pub struct Viewport{pub scale:f32,pub ox:f32,pub oy:f32}
pub fn viewport(width:f32,height:f32,percent:u16)->Option<Viewport>{
 if !width.is_finite()||!height.is_finite()||width<=0.||height<=0.{return None}
 let scale=(width/1280.).min(height/720.)*percent.clamp(50,100) as f32/100.;
 Some(Viewport{scale,ox:(width-1280.*scale)*0.5,oy:(height-720.*scale)*0.5})
}
