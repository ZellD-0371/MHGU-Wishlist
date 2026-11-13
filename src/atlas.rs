//! Both releases share sprite IDs and one NRO; file size selects the approved layout.
mod low{include!("atlas_low_memory.rs");}
mod hd{include!("atlas_hd.rs");}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Layout{pub width:u16,pub height:u16,pub uv_width:u16,pub uv_height:u16,pub low_memory:bool}
pub fn layout(bytes:u64)->Option<Layout>{
 if bytes==(low::SOURCE_WIDTH*low::SOURCE_HEIGHT*4) as u64{return Some(Layout{width:low::SOURCE_WIDTH as u16,height:low::SOURCE_HEIGHT as u16,uv_width:low::SOURCE_WIDTH as u16,uv_height:low::SOURCE_HEIGHT as u16,low_memory:false})}
 if bytes==hd::BYTES as u64{return Some(Layout{width:hd::WIDTH as u16,height:hd::HEIGHT as u16,uv_width:hd::WIDTH as u16,uv_height:hd::HEIGHT as u16,low_memory:false})}
 if bytes==low::BYTES as u64{return Some(Layout{width:low::WIDTH as u16,height:low::HEIGHT as u16,uv_width:low::SOURCE_WIDTH as u16,uv_height:low::CROPPED_HEIGHT as u16,low_memory:true})}None
}
