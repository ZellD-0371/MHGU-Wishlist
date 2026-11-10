//! Original XX window_sub skin, drawn with portable Forge primitives.
use forge::{imgui::DrawList,sys::imgui::{self as sys,ImVec2}};
mod art { include!("skin_generated.rs"); }

pub fn begin() {
    let d=DrawList::window().raw();
    unsafe { sys::ImDrawList_ChannelsSplit(d,2); sys::ImDrawList_ChannelsSetCurrent(d,1); }
}

pub fn end(scale:f32,opacity:f32) {
    let d=DrawList::window();
    unsafe {
        sys::ImDrawList_ChannelsSetCurrent(d.raw(),0);
        sys::ImDrawList_PushClipRectFullScreen(d.raw());
        let p=sys::igGetWindowPos();let size=sys::igGetWindowSize();
        let x=p.x;let y=p.y;let w=size.x;let h=size.y;
        // window_sub uses a dark blue patterned fill and cyan-tinted edges.
        let a=(opacity.clamp(0.,1.)*255.+0.5) as u32;
        d.add_rect_filled(ImVec2{x:x+4.*scale,y:y+4.*scale},ImVec2{x:x+w-4.*scale,y:y+h-4.*scale},(a<<24)|0x001d1712,0.);
        sys::ImDrawList_PushClipRect(d.raw(),ImVec2{x:x+4.*scale,y:y+4.*scale},ImVec2{x:x+w-4.*scale,y:y+h-4.*scale},true);
        // Keep the small repeating diamonds as geometry rather than thousands
        // of individual source pixels per tile. This stays renderer-independent.
        let step=18.*scale;let half=step*0.5;
        let pattern=((a*38/255)<<24)|0x003b3027;
        let mut py=y+4.*scale;
        while py<y+h {
            let mut px=x+4.*scale;
            while px<x+w {
                let points=[ImVec2{x:px+half,y:py},ImVec2{x:px+step,y:py+half},ImVec2{x:px+half,y:py+step},ImVec2{x:px,y:py+half}];
                for i in 0..4 {d.add_line(points[i],points[(i+1)%4],pattern,scale);}
                px+=step;
            }
            py+=step;
        }
        sys::ImDrawList_PopClipRect(d.raw());
        for px in [x,x+w-4.*scale] {
            d.add_rect_filled(ImVec2{x:px,y:y+8.*scale},ImVec2{x:px+4.*scale,y:y+h-8.*scale},0xff000000,0.);
        }
        // Original edge is uniform along X, so stretch each source row once.
        for &(_,ry,_,col) in art::EDGE {
            let col=(col&0xff000000)|((((col>>16)&255)*134/255)<<16)|((((col>>8)&255)*127/255)<<8)|((col&255)*43/255);
            let top=y+ry as f32*scale;
            let bottom=y+h-(ry as f32+1.)*scale;
            for py in [top,bottom] {
                d.add_rect_filled(ImVec2{x:x+8.*scale,y:py},ImVec2{x:x+w-8.*scale,y:py+scale},col,0.);
            }
        }
        for flip_y in [false,true] {for flip_x in [false,true] {
            for &(rx,ry,rw,col) in art::CORNER {
                let px=if flip_x{x+w-(rx+rw) as f32*scale}else{x+rx as f32*scale};
                let py=if flip_y{y+h-(ry+1) as f32*scale}else{y+ry as f32*scale};
                // GUI VertexColor for the four original corner sprites.
                let r=(col&255)*248/255;let g=((col>>8)&255)*230/255;let b=((col>>16)&255)*108/255;
                let tint=(col&0xff000000)|(b<<16)|(g<<8)|r;
                d.add_rect_filled(ImVec2{x:px,y:py},ImVec2{x:px+rw as f32*scale,y:py+scale},tint,0.);
            }
        }}
        sys::ImDrawList_PopClipRect(d.raw());
        sys::ImDrawList_ChannelsMerge(d.raw());
    }
}
