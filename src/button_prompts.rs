//! Recomp menu bindings affect artwork only; input remains the engine menu word.
use alloc::{string::{String, ToString}, vec::Vec};

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
        "pad:dpup" => Some(12), "pad:dpdown" => Some(13),
        "pad:start" => Some(14), "pad:back" => Some(15),
        _ => None,
    };
    if let Some(pos) = sprite { return Prompt::Position(pos); }

    // Chords and logical buttons have no single sprite: keep a compact text
    // prompt rather than printing raw "dpup" etc.
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

/// First atlas tile of the extra controller prompts (D-pad up, D-pad down,
/// plus/start, minus/back); equals icons::PAD_EXTRA, checked by the host tests.
pub const PAD_EXTRA_TILE: usize = 159;
// All sixteen physical pad buttons have artwork: twelve base slots (107..118)
// and four extras; icons::draw adds the controller layout offset to both.
pub fn icon_slot(position: u8, _layout: usize) -> usize {
    match position {
        0 => 108, 1 => 107, 2 => 109, 3 => 110,
        4 => 113, 5 => 114, 6 => 116, 7 => 117,
        8 => 118, 9 => 115, 10 => 111, 11 => 112,
        12..=15 => PAD_EXTRA_TILE + (position - 12) as usize,
        _ => 107,
    }
}
/// Kenney prompt name of a HID keyboard usage (Wishlist keyboard bindings).
pub fn hid_key_name(usage: u32) -> Option<&'static str> {
    const LETTERS: [&str; 26] = ["keyboard_a","keyboard_b","keyboard_c","keyboard_d","keyboard_e","keyboard_f","keyboard_g","keyboard_h","keyboard_i","keyboard_j","keyboard_k","keyboard_l","keyboard_m","keyboard_n","keyboard_o","keyboard_p","keyboard_q","keyboard_r","keyboard_s","keyboard_t","keyboard_u","keyboard_v","keyboard_w","keyboard_x","keyboard_y","keyboard_z"];
    const DIGITS: [&str; 10] = ["keyboard_1","keyboard_2","keyboard_3","keyboard_4","keyboard_5","keyboard_6","keyboard_7","keyboard_8","keyboard_9","keyboard_0"];
    const F: [&str; 12] = ["keyboard_f1","keyboard_f2","keyboard_f3","keyboard_f4","keyboard_f5","keyboard_f6","keyboard_f7","keyboard_f8","keyboard_f9","keyboard_f10","keyboard_f11","keyboard_f12"];
    Some(match usage {
        4..=29 => LETTERS[(usage - 4) as usize],
        30..=39 => DIGITS[(usage - 30) as usize],
        58..=69 => F[(usage - 58) as usize],
        40 => "keyboard_enter", 41 => "keyboard_escape", 42 => "keyboard_backspace",
        43 => "keyboard_tab", 44 => "keyboard_space", 45 => "keyboard_minus",
        46 => "keyboard_equals", 47 => "keyboard_bracket_open", 48 => "keyboard_bracket_close",
        49 => "keyboard_slash_back", 51 => "keyboard_semicolon", 52 => "keyboard_apostrophe",
        53 => "keyboard_tilde", 54 => "keyboard_comma", 55 => "keyboard_period",
        56 => "keyboard_slash_forward", 57 => "keyboard_capslock", 70 => "keyboard_printscreen",
        71 => "keyboard_scroll_lock", 72 => "keyboard_pause", 73 => "keyboard_insert",
        74 => "keyboard_home", 75 => "keyboard_page_up", 76 => "keyboard_delete",
        77 => "keyboard_end", 78 => "keyboard_page_down", 79 => "keyboard_arrow_right",
        80 => "keyboard_arrow_left", 81 => "keyboard_arrow_down", 82 => "keyboard_arrow_up",
        83 => "keyboard_numlock", 85 => "keyboard_asterisk", 87 => "keyboard_numpad_plus",
        88 => "keyboard_numpad_enter",
        _ => return None,
    })
}
/// Keys whose Kenney picture is a word on a wide cap: drawn as a keycap with
/// this label instead, so the word stays readable at prompt size.
pub fn wide_label(name: &str) -> Option<&'static str> {
    Some(match name {
        "keyboard_space" => "SPACE", "keyboard_backspace" => "BACK", "keyboard_capslock" => "CAPS",
        "keyboard_tab" => "TAB", "keyboard_shift" => "SHIFT", "keyboard_ctrl" => "CTRL",
        "keyboard_alt" => "ALT", "keyboard_delete" => "DEL", "keyboard_end" => "END",
        "keyboard_escape" => "ESC", "keyboard_home" => "HOME", "keyboard_insert" => "INS",
        "keyboard_page_up" => "PG UP", "keyboard_page_down" => "PG DN", "keyboard_numlock" => "NUM",
        "keyboard_printscreen" => "PRT SC", "keyboard_pause" | "keyboard_pause_break" => "PAUSE",
        "keyboard_scroll_lock" => "SCR LK", "keyboard_enter" | "keyboard_numpad_enter" => "ENTER",
        _ => return None,
    })
}
/// HID usage of a Recomp key name ("Space", "Left Shift"); mouse buttons and
/// wheel notches have none.
pub fn recomp_key_usage(key: &str) -> Option<u32> {
    let k = key.trim().to_ascii_lowercase();
    let k = k.strip_prefix("keypad ").map_or(k.as_str(), |rest| match rest { "+" | "plus" => "numpad plus", "enter" => "numpad enter", "*" | "multiply" => "*", _ => rest });
    Some(match k {
        "return" | "enter" => 40, "escape" | "esc" => 41, "backspace" => 42, "tab" => 43,
        "space" => 44, "-" | "minus" => 45, "=" | "equals" => 46, "[" => 47, "]" => 48,
        "\\" => 49, ";" => 51, "'" => 52, "`" | "grave" => 53, "," => 54, "." => 55,
        "/" => 56, "capslock" | "caps lock" => 57, "printscreen" | "print screen" => 70,
        "scrolllock" | "scroll lock" => 71, "pause" => 72, "insert" => 73, "home" => 74,
        "pageup" | "page up" => 75, "delete" => 76, "end" => 77, "pagedown" | "page down" => 78,
        "right" => 79, "left" => 80, "down" => 81, "up" => 82, "numlock" | "num lock" => 83,
        "*" => 85, "numpad plus" => 87, "numpad enter" => 88,
        "left ctrl" | "ctrl" => 224, "left shift" | "shift" => 225, "left alt" | "alt" => 226,
        "right ctrl" => 228, "right shift" => 229, "right alt" => 230,
        k if k.len() == 1 && k.as_bytes()[0].is_ascii_lowercase() => 4 + u32::from(k.as_bytes()[0] - b'a'),
        k if k.len() == 1 && k.as_bytes()[0].is_ascii_digit() => if k == "0" { 39 } else { 30 + u32::from(k.as_bytes()[0] - b'1') },
        k if k.len() >= 2 && k.starts_with('f') => match k[1..].parse::<u32>() { Ok(n @ 1..=12) => 57 + n, _ => return None },
        _ => return None,
    })
}
/// Kenney prompt name of a Recomp key name ("Space", "Left Shift", "Mouse Left").
pub fn recomp_key_name(key: &str) -> Option<&'static str> {
    match key.trim().to_ascii_lowercase().as_str() {
        "mouse left" => return Some("mouse_left"), "mouse right" => return Some("mouse_right"),
        "mouse middle" => return Some("mouse_scroll"), "wheel up" => return Some("mouse_scroll_up"),
        "wheel down" => return Some("mouse_scroll_down"), "mouse x1" | "mouse 4" => return Some("mouse_side_back"),
        "mouse x2" | "mouse 5" => return Some("mouse_side_forward"),
        _ => {}
    }
    match recomp_key_usage(key)? {
        224 | 228 => Some("keyboard_ctrl"), 225 | 229 => Some("keyboard_shift"), 226 | 230 => Some("keyboard_alt"),
        usage => hid_key_name(usage),
    }
}
/// [Keybinds] row of the pad button each menu action uses by default.
const KEYBIND_ROWS: [&str; 16] = [
    "A", "B", "X", "Y", "L", "R", "ZL", "ZR",
    "DpadLeft", "DpadRight", "DpadUp", "DpadDown", "Plus", "Minus", "", "",
];
/// Keys of the sixteen menu actions for a keyboard player.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct KeyConfig {
    /// Recomp actions manage the game menus ([Actions] Enable and ManageMenus).
    pub menus: bool,
    /// [Keybinds] keys of each action's default pad button.
    pub rows: [Vec<String>; 16],
    /// [Actions.Kbm] actions the player changed; None keeps the default.
    pub changed: [Option<Vec<String>>; 16],
}
/// Inputs of an [Actions.Kbm] line ("key:Space | mouse:Left") as [Keybinds]
/// names ("Space", "Mouse Left"); chords and pad inputs are left out.
fn action_keys(value: &str) -> Vec<String> {
    value.split('|').map(str::trim).filter(|v| !v.contains('+') || v.ends_with('+')).filter_map(|v| {
        let lower = v.to_ascii_lowercase();
        if lower.starts_with("key:") { Some(String::from(v[4..].trim())) }
        else if lower.starts_with("mouse:") { Some(alloc::format!("Mouse {}", v[6..].trim())) }
        else { None }
    }).collect()
}
pub fn parse_keys(ini: &str) -> KeyConfig {
    let mut out = KeyConfig::default();
    let (mut enabled, mut menus, mut section) = (true, true, "");
    for line in ini.lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') { section = &line[1..line.len()-1]; continue; }
        let Some((key, value)) = line.split_once('=') else { continue };
        let (key, value) = (key.trim(), value.trim());
        if section.eq_ignore_ascii_case("Actions") {
            let on = !(value.eq_ignore_ascii_case("off") || value.eq_ignore_ascii_case("false") || value == "0");
            if key.eq_ignore_ascii_case("Enable") { enabled = on; }
            if key.eq_ignore_ascii_case("ManageMenus") { menus = on; }
        } else if section.eq_ignore_ascii_case("Keybinds") {
            for (i, row) in KEYBIND_ROWS.iter().enumerate() {
                if !row.is_empty() && key.eq_ignore_ascii_case(row) {
                    out.rows[i] = value.split(',').map(str::trim)
                        .filter(|k| !k.is_empty() && !k.eq_ignore_ascii_case("none")).map(String::from).collect();
                }
            }
        } else if section.eq_ignore_ascii_case("Actions.Kbm") {
            let Some(i) = ACTION_NAMES.iter().position(|x| key.eq_ignore_ascii_case(x)
                || (*x == "MenuConfirm" && key.eq_ignore_ascii_case("Confirm"))
                || (*x == "MenuCancel" && key.eq_ignore_ascii_case("Cancel"))) else { continue };
            if !(value.is_empty() || value.eq_ignore_ascii_case("default") || value.eq_ignore_ascii_case("original")) {
                out.changed[i] = Some(action_keys(value));
            }
        }
    }
    out.menus = enabled && menus;
    out
}
fn is_mouse(key: &str) -> bool { let k = key.to_ascii_lowercase(); k.starts_with("mouse") || k.starts_with("wheel") }
/// The key shown for a menu action in our window. Its pointer is ours there:
/// confirm is the left click and cancel the right click. Other actions use the
/// changed menu action, else Recomp's menu default (X = Space, the rest as
/// [Keybinds]), or plain [Keybinds] when actions do not manage menus. An action
/// bound only to mouse buttons keeps its pad prompt (None).
pub fn menu_key(c: &KeyConfig, action: usize) -> Option<String> {
    match action { 0 => return Some("Mouse Left".into()), 1 => return Some("Mouse Right".into()), _ => {} }
    action_inputs(c, action).into_iter().find(|k| !is_mouse(k))
}
/// Every key or mouse button that triggers a menu action in Recomp's menus.
pub fn action_inputs(c: &KeyConfig, action: usize) -> Vec<String> {
    if !c.menus { return c.rows.get(action).cloned().unwrap_or_default() }
    if let Some(Some(keys)) = c.changed.get(action) { return keys.clone() }
    match action {
        0 => alloc::vec![String::from("Mouse Left")],
        1 => alloc::vec![String::from("Mouse Right")],
        2 => alloc::vec![String::from("Space")],
        _ => c.rows.get(action).cloned().unwrap_or_default(),
    }
}
/// Menu word action bits of the eight button actions (directions excluded).
pub const ACTION_BITS: [u32; 8] = [0x2000, 0x4000, 0x1000, 0x8000, 0x100, 0x200, 0x400, 0x800];
/// The menu word's button actions from the keys held now, as Recomp's menus
/// map them. Used while our window owns input and the keyboard is in use: a
/// navigation takeover below us (KatSmithy) leaves only the raw pad word, where
/// a key bound to a pad button in [Keybinds] (B = Space) would mean that button.
pub fn keyboard_word(c: &KeyConfig, down: impl Fn(u32) -> bool) -> u32 {
    ACTION_BITS.iter().enumerate().filter(|(i, _)| action_inputs(c, *i).iter().filter_map(|k| recomp_key_usage(k)).any(|u| down(u))).fold(0, |w, (_, b)| w | b)
}
/// Short text for a key without a picture (three letters at most).
pub fn short_key(key: &str) -> String {
    let t: String = key.trim().strip_prefix("Keypad ").unwrap_or(key.trim()).chars().filter(|c| *c != ' ').collect();
    t.to_uppercase().chars().take(3).collect()
}
/// Recomp's [Controls] KeyboardPrompts (on when missing).
pub fn parse_keyboard_prompts(ini: &str) -> bool {
    let mut section = "";
    let mut on = true;
    for line in ini.lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') { section = &line[1..line.len()-1]; continue; }
        let Some((key, value)) = line.split_once('=') else { continue };
        if section.eq_ignore_ascii_case("Controls") && key.trim().eq_ignore_ascii_case("KeyboardPrompts") {
            let v = value.trim();
            on = !(v.eq_ignore_ascii_case("off") || v.eq_ignore_ascii_case("false") || v == "0");
        }
    }
    on
}
#[cfg(not(target_arch="arm"))] pub fn keyboard_prompts() -> bool { false }
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
 static mut KEYBOARD:bool=true;
 static mut KEYS:Option<KeyConfig>=None;
 pub fn keyboard_actions(down:impl Fn(u32)->bool)->Option<u32>{unsafe{(*core::ptr::addr_of!(KEYS)).as_ref().map(|c|keyboard_word(c,down))}}
 pub fn key_prompt(action:usize)->Option<String>{unsafe{(*core::ptr::addr_of!(KEYS)).as_ref().and_then(|c|menu_key(c,action))}}
 pub fn keyboard_prompts()->bool{unsafe{KEYBOARD}}
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
  if n==size{if let Ok(text)=core::str::from_utf8(&bytes){KEYBOARD=parse_keyboard_prompts(text);KEYS=Some(parse_keys(text));let next=parse(text);if (&*core::ptr::addr_of!(PROMPTS)).as_ref()!=Some(&next){crate::diagnostics::event(12,"controller prompt bindings refreshed",[0,0,0,0]);PROMPTS=Some(next);}}}
 }}
 pub fn get(index:usize)->Prompt{unsafe{(&*core::ptr::addr_of!(PROMPTS)).as_ref().map(|p|p[index].clone()).unwrap_or(Prompt::Original)}}
}
#[cfg(target_arch="arm")]pub use runtime::{refresh,get,is_recomp,keyboard_prompts,key_prompt,keyboard_actions};
#[cfg(not(target_arch="arm"))] pub fn keyboard_actions(_: impl Fn(u32) -> bool) -> Option<u32> { None }
#[cfg(not(target_arch="arm"))] pub fn key_prompt(_: usize) -> Option<String> { None }
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
            ("pad:dpup", Prompt::Position(12)),
            ("pad:dpdown", Prompt::Position(13)),
            ("pad:start", Prompt::Position(14)),
            ("pad:back", Prompt::Position(15)),
        ];
        for (name, expected) in items {
            assert_eq!(binding(name), expected, "{name}");
        }
        assert_eq!(icon_slot(10, 0), 111); // L3 atlas sprite
        assert_eq!(icon_slot(11, 0), 112); // R3 atlas sprite
        assert_eq!(icon_slot(12, 0), PAD_EXTRA_TILE);
        assert_eq!(icon_slot(15, 0), PAD_EXTRA_TILE + 3);
    }

    #[test]
    fn all_sixteen_menu_actions_are_recognized() {
        for (i, name) in ACTION_NAMES.iter().enumerate() {
            let ini = format!("[Actions.Pad]\n{name}=pad:lstick");
            let p = parse(&ini);
            assert_eq!(p[i], Prompt::Position(10), "{name}");
            assert_eq!(p.iter().filter(|v| *v != &Prompt::Original).count(), 1);
        }
        assert_eq!(parse("[Actions.Pad]\nMenuUp=pad:dpdown")[10], Prompt::Position(13));
        assert_eq!(parse("[Actions.Pad]\nMenuDown=pad:dpup")[11], Prompt::Position(12));
        assert_eq!(parse("[Actions.Pad]\nMenuPlus=pad:start")[12], Prompt::Position(14));
        assert_eq!(parse("[Actions.Pad]\nMenuMinus=pad:back")[13], Prompt::Position(15));
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
