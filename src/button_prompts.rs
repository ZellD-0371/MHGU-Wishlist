//! Recomp menu bindings affect artwork only; input remains the engine menu word.
use alloc::string::{String, ToString};

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Prompt { Original, Position(u8), Text(String) }

// Keep the original ten action indexes stable. The last six cover the other
// six game-menu actions in MHGU Recomp 0.17.x.
pub const ACTION_NAMES: [&str; 16] = [
    "MenuConfirm", "MenuCancel", "MenuX", "MenuY",
    "MenuTabL", "MenuTabR", "MenuPageL", "MenuPageR",
    "MenuLeft", "MenuRight", "MenuUp", "MenuDown",
    "MenuPlus", "MenuMinus", "MenuL3", "MenuR3",
];

pub fn parse(ini: &str) -> [Prompt; 16] {
    let mut out = core::array::from_fn(|_| Prompt::Original);
    let mut section = "";
    let mut enabled = true;
    let mut menus = true;
    for line in ini.lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = &line[1..line.len()-1];
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let key = key.trim();
        let value = value.trim();
        if section.eq_ignore_ascii_case("Actions") {
            let on = !(value.eq_ignore_ascii_case("off")
                || value.eq_ignore_ascii_case("false") || value == "0");
            if key.eq_ignore_ascii_case("Enable") { enabled = on; }
            if key.eq_ignore_ascii_case("ManageMenus") { menus = on; }
        }
        if !section.eq_ignore_ascii_case("Actions.Pad") { continue; }
        let Some(i) = ACTION_NAMES.iter().position(|x|
            key.eq_ignore_ascii_case(x)
            || (*x == "MenuConfirm" && key.eq_ignore_ascii_case("Confirm"))
            || (*x == "MenuCancel" && key.eq_ignore_ascii_case("Cancel"))
        ) else { continue };
        out[i] = binding(value);
    }
    if !enabled || !menus { return core::array::from_fn(|_| Prompt::Original); }
    out
}

fn binding(value: &str) -> Prompt {
    if value.is_empty() || value.eq_ignore_ascii_case("default")
        || value.eq_ignore_ascii_case("original") {
        return Prompt::Original;
    }
    if value.eq_ignore_ascii_case("none") {
        return Prompt::Text("—".to_string());
    }
    // Recomp accepts alternatives separated by '|'; show the first one.
    // Chords cannot use a single atlas sprite: render their parts as text.
    let first = value.split('|').next().unwrap_or("").trim();
    let input = first.to_ascii_lowercase();
    let sprite = match input.as_str() {
        "pad:south" => Some(0), "pad:east" => Some(1),
        "pad:north" => Some(2), "pad:west" => Some(3),
        "pad:ltrigger" | "pad:lefttrigger" | "pad:lt" => Some(4),
        "pad:rtrigger" | "pad:righttrigger" | "pad:rt" => Some(5),
        "pad:lshoulder" | "pad:leftshoulder" => Some(6),
        "pad:rshoulder" | "pad:rightshoulder" => Some(7),
        "pad:dpleft" => Some(8), "pad:dpright" => Some(9),
        "pad:lstick" => Some(10), "pad:rstick" => Some(11),
        _ => None,
    };
    if let Some(pos) = sprite { return Prompt::Position(pos); }

    // The atlas has no separate sprites for up/down, +/- or analog directions.
    // Preserve a useful, compact prompt rather than printing raw "dpup" etc.
    let mut label = String::new();
    for part in first.split('+') {
        if !label.is_empty() { label.push('+'); }
        let lower = part.trim().to_ascii_lowercase();
        let name = match lower.as_str() {
            "pad:south" => "B", "pad:east" => "A",
            "pad:north" => "X", "pad:west" => "Y",
            "pad:lshoulder" | "pad:leftshoulder" => "L",
            "pad:rshoulder" | "pad:rightshoulder" => "R",
            "pad:ltrigger" | "pad:lefttrigger" | "pad:lt" => "ZL",
            "pad:rtrigger" | "pad:righttrigger" | "pad:rt" => "ZR",
            "pad:lstick" => "L3", "pad:rstick" => "R3",
            "pad:dpleft" => "←", "pad:dpright" => "→",
            "pad:dpup" => "↑", "pad:dpdown" => "↓",
            "pad:start" => "+", "pad:back" => "−",
            // These are *logical* Switch buttons, not physical pad buttons.
            // Render a label rather than pretending to know the Tier-0 remap.
            _ => {
                let token = part.trim();
                token.strip_prefix("pad:")
                    .or_else(|| token.strip_prefix("sw:"))
                    .or_else(|| token.strip_prefix("sw.pad:"))
                    .or_else(|| token.strip_prefix("sw.kbm:"))
                    .unwrap_or(token)
            }
        };
        label.push_str(name);
    }
    Prompt::Text(label)
}

// Only these twelve physical buttons have dedicated artwork in Wishlist's atlas.
// D-pad up/down and Start/Back deliberately remain text fallbacks.
pub fn icon_slot(position: u8, _layout: usize) -> usize {
    match position {
        0 => 108, 1 => 107, 2 => 109, 3 => 110,
        4 => 113, 5 => 114, 6 => 116, 7 => 117,
        8 => 118, 9 => 115, 10 => 111, 11 => 112,
        _ => 107,
    }
}
#[cfg(not(target_arch="arm"))] pub fn is_recomp()->bool { false }
#[cfg(not(target_arch="arm"))] pub fn refresh(_: f32) {}
#[cfg(not(target_arch="arm"))] pub fn get(_: usize) -> Prompt { Prompt::Original }
pub fn recomp_imports(a:usize,b:usize)->bool{
 [a,b].iter().all(|&p|(0x03000000..0x03100000).contains(&p)&&p&15==0)
}
#[cfg(target_arch="arm")]
mod runtime{
 use super::*;
 use forge::sys::imgui as sys;
 static mut PROMPTS:Option<[Prompt;16]>=None;
 static mut WAIT:f32=0.;
 unsafe extern "C"{fn forge_graphics_isMenuVisible()->bool;}
 pub fn is_recomp()->bool { recomp_imports(forge_graphics_isMenuVisible as *const () as usize,sys::igImFileOpen as *const () as usize) }
 pub fn refresh(dt:f32){unsafe{
  // Recomp resolves both imports directly to its reserved HLE address range.
  // Fail closed on hardware/emulators: never send a PC path to nn::fs.
  if !recomp_imports(forge_graphics_isMenuVisible as *const () as usize,sys::igImFileOpen as *const () as usize){return}
  WAIT-=dt.max(0.);if WAIT>0.{return}WAIT=1.;
  let file=sys::igImFileOpen(c"mhgurecomp.ini".as_ptr(),c"rb".as_ptr());if file.is_null(){return}
  let size=sys::igImFileGetSize(file);
  if size>128*1024{sys::igImFileClose(file);return}
  let mut bytes=alloc::vec![0u8;size as usize];let n=sys::igImFileRead(bytes.as_mut_ptr().cast(),1,size,file);sys::igImFileClose(file);
  if n==size{if let Ok(text)=core::str::from_utf8(&bytes){let next=parse(text);if (&*core::ptr::addr_of!(PROMPTS)).as_ref()!=Some(&next){crate::diagnostics::event(12,"controller prompt bindings refreshed",[0,0,0,0]);PROMPTS=Some(next);}}}
 }}
 pub fn get(index:usize)->Prompt{unsafe{(&*core::ptr::addr_of!(PROMPTS)).as_ref().map(|p|p[index].clone()).unwrap_or(Prompt::Original)}}
}
#[cfg(target_arch="arm")]pub use runtime::{refresh,get,is_recomp};
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn ten_previous_actions_keep_their_indexes() {
        let p = parse("[Actions]\nEnable=1\nManageMenus=1\n[Actions.Pad]\n\
            MenuConfirm=pad:south\nMenuCancel=pad:east\nMenuX=pad:west\nMenuY=pad:north\n\
            MenuTabL=pad:ltrigger\nMenuTabR=pad:rtrigger\n\
            MenuPageL=pad:lshoulder\nMenuPageR=pad:rshoulder\n\
            MenuLeft=pad:dpleft\nMenuRight=pad:dpright");
        assert_eq!(&p[..10], &[
            Prompt::Position(0), Prompt::Position(1),
            Prompt::Position(3), Prompt::Position(2),
            Prompt::Position(4), Prompt::Position(5),
            Prompt::Position(6), Prompt::Position(7),
            Prompt::Position(8), Prompt::Position(9),
        ][..]);
        assert_eq!(p.len(), 16);
        assert_eq!(icon_slot(0, 0), 108);
        assert_eq!(icon_slot(1, 0), 107);
        assert_eq!(icon_slot(8, 0), 118);
        assert_eq!(icon_slot(9, 0), 115);
    }

    #[test]
    fn all_sixteen_pad_inputs_are_readable() {
        let items = [
            ("pad:south", Prompt::Position(0)),
            ("pad:east", Prompt::Position(1)),
            ("pad:north", Prompt::Position(2)),
            ("pad:west", Prompt::Position(3)),
            ("pad:ltrigger", Prompt::Position(4)),
            ("pad:rtrigger", Prompt::Position(5)),
            ("pad:lshoulder", Prompt::Position(6)),
            ("pad:rshoulder", Prompt::Position(7)),
            ("pad:dpleft", Prompt::Position(8)),
            ("pad:dpright", Prompt::Position(9)),
            ("pad:lstick", Prompt::Position(10)),
            ("pad:rstick", Prompt::Position(11)),
            ("pad:dpup", Prompt::Text("↑".into())),
            ("pad:dpdown", Prompt::Text("↓".into())),
            ("pad:start", Prompt::Text("+".into())),
            ("pad:back", Prompt::Text("−".into())),
        ];
        for (name, expected) in items {
            assert_eq!(binding(name), expected, "{name}");
        }
        assert_eq!(icon_slot(10, 0), 111); // L3 atlas sprite
        assert_eq!(icon_slot(11, 0), 112); // R3 atlas sprite
    }

    #[test]
    fn all_sixteen_menu_actions_are_recognized() {
        for (i, name) in ACTION_NAMES.iter().enumerate() {
            let ini = format!("[Actions.Pad]\n{name}=pad:lstick");
            let p = parse(&ini);
            assert_eq!(p[i], Prompt::Position(10), "{name}");
            assert_eq!(p.iter().filter(|v| *v != &Prompt::Original).count(), 1);
        }
        assert_eq!(parse("[Actions.Pad]\nMenuUp=pad:dpdown")[10], Prompt::Text("↓".into()));
        assert_eq!(parse("[Actions.Pad]\nMenuDown=pad:dpup")[11], Prompt::Text("↑".into()));
        assert_eq!(parse("[Actions.Pad]\nMenuPlus=pad:start")[12], Prompt::Text("+".into()));
        assert_eq!(parse("[Actions.Pad]\nMenuMinus=pad:back")[13], Prompt::Text("−".into()));
        assert_eq!(parse("[Actions.Pad]\nMenuL3=pad:lstick")[14], Prompt::Position(10));
        assert_eq!(parse("[Actions.Pad]\nMenuR3=pad:rstick")[15], Prompt::Position(11));
    }

    #[test]
    fn chords_none_default_and_disabled_flags() {
        let p = parse("[Actions.Pad]\nMenuX=pad:lstick+pad:north\n\
            MenuY=pad:rstick+pad:dpup\nMenuLeft=None\nMenuRight=Default");
        assert_eq!(p[2], Prompt::Text("L3+X".into()));
        assert_eq!(p[3], Prompt::Text("R3+↑".into()));
        assert_eq!(p[8], Prompt::Text("—".into()));
        assert_eq!(p[9], Prompt::Original);
        for key in ["Enable", "ManageMenus"] {
            assert_eq!(parse(&format!("[Actions.Pad]\nMenuConfirm=pad:south\n[Actions]\n{key}=Off")),
                core::array::from_fn(|_| Prompt::Original));
        }
        assert_eq!(parse("[Actions.Kbm]\nMenuConfirm=key:e")[0], Prompt::Original);
    }

    #[test]
    fn original_dpad_remap_and_recomp_import_guard() {
        let p = parse("[Actions.Pad]\nMenuLeft=pad:lshoulder\nMenuRight=pad:rshoulder\nMenuTabL=None\nMenuTabR=None");
        assert_eq!(p[4], Prompt::Text("—".into()));
        assert_eq!(p[5], Prompt::Text("—".into()));
        assert_eq!(p[8], Prompt::Position(6));
        assert_eq!(p[9], Prompt::Position(7));
        assert!(recomp_imports(0x03000100, 0x03000200));
        for a in [0, 0x02400100, 0x03200100, 0x71000000, 0x03000101] {
            assert!(!recomp_imports(a, 0x03000200));
            assert!(!recomp_imports(0x03000100, a));
        }
    }
}
