//! Settings capture follows Better Skillinfo: close Forge, input, reopen, confirm.
use alloc::{string::String,ffi::CString};
use core::sync::atomic::{AtomicU32,AtomicBool,Ordering};
use forge::{imgui,sys::imgui::ImVec2};
use crate::{STATE,binding_model::*};
static RAW:AtomicU32=AtomicU32::new(0);
static CAPTURING:AtomicBool=AtomicBool::new(false);
pub fn capturing()->bool{CAPTURING.load(Ordering::Acquire)}
pub fn publish(raw:u32){RAW.store(raw,Ordering::Release);}
pub fn snapshot(buttons:u32,lx:i32,ly:i32,rx:i32,ry:i32)->u32{let mut v=buttons&0xffff;for (x,y,bit) in [(lx,ly,16),(rx,ry,20)]{if x<=-18000{v|=1<<bit}if x>=18000{v|=1<<(bit+2)}if y>=18000{v|=1<<(bit+1)}if y<=-18000{v|=1<<(bit+3)}}v}
pub fn label(chord:u32)->String{let names=["A","B","X","Y","L3","R3","L","R","ZL","ZR","+","-","D Left","D Up","D Right","D Down","L Stick Left","L Stick Up","L Stick Right","L Stick Down","R Stick Left","R Stick Up","R Stick Right","R Stick Down"];let mut out=String::new();for (i,n) in names.iter().enumerate(){if chord&(1<<i)!=0{if !out.is_empty(){out.push_str(" + ");}out.push_str(n);}}out}
pub fn keyboard_active()->bool{use forge::input::Input;Input::is_alt_down()||Input::is_ctrl_down()||Input::is_shift_down()||(4..=231).any(|k|unsafe{hid_down(k)})}
// Keyboard bindings contain one HID key plus Ctrl/Shift/Alt modifiers.
#[cfg(feature="keyboard-input")]
pub const CTRL:u32=1<<16;
#[cfg(feature="keyboard-input")]
pub const SHIFT:u32=1<<17;
#[cfg(feature="keyboard-input")]
pub const ALT:u32=1<<18;


// Forge's C ABI accepts HID usage IDs; using integers also covers keys omitted
// by the Rust SDK enum, without constructing invalid enum discriminants.
unsafe extern "C"{#[link_name="forge_input_isKeyDown"] fn hid_down(key:u32)->bool;}
#[cfg(feature="keyboard-input")]
fn valid_hid(n:u32)->bool{(4..=101).contains(&n)||n==103}
#[cfg(feature="keyboard-input")]
fn gui_key(n:u32)->Option<forge::sys::imgui::ImGuiKey>{
    use forge::sys::imgui::ImGuiKey as K;
    let v=match n{
        4..=29=>546+n-4,30..=38=>537+n-30,39=>536,58..=69=>572+n-58,
        40=>K::Enter as u32,41=>K::Escape as u32,42=>K::Backspace as u32,43=>512,
        44=>K::Space as u32,45=>K::Minus as u32,46=>K::Equal as u32,47=>K::LeftBracket as u32,
        48=>K::RightBracket as u32,49=>K::Backslash as u32,51=>K::Semicolon as u32,
        52=>K::Apostrophe as u32,53=>K::GraveAccent as u32,54=>K::Comma as u32,
        55=>K::Period as u32,56=>K::Slash as u32,57=>K::CapsLock as u32,
        50=>K::Oem102 as u32,70=>K::PrintScreen as u32,71=>K::ScrollLock as u32,72=>K::Pause as u32,
        83=>K::NumLock as u32,84=>K::KeypadDivide as u32,85=>K::KeypadMultiply as u32,
        86=>K::KeypadSubtract as u32,87=>K::KeypadAdd as u32,88=>K::KeypadEnter as u32,
        89..=97=>K::Keypad1 as u32+n-89,98=>K::Keypad0 as u32,99=>K::KeypadDecimal as u32,
        100=>K::Oem102 as u32,101=>K::Menu as u32,103=>K::KeypadEqual as u32,
        73=>K::Insert as u32,74=>K::Home as u32,75=>K::PageUp as u32,76=>K::Delete as u32,
        77=>K::End as u32,78=>K::PageDown as u32,79=>K::RightArrow as u32,80=>K::LeftArrow as u32,
        81=>K::DownArrow as u32,82=>K::UpArrow as u32,_=>return None};
    Some(unsafe{core::mem::transmute::<u32,K>(v)})
}
#[cfg(feature="keyboard-input")]
pub fn key_label(v:u32)->String{
    let mut s=String::new();
    for (bit,name) in [(CTRL,"Ctrl"),(SHIFT,"Shift"),(ALT,"Alt")]{if v&bit!=0{if !s.is_empty(){s.push_str(" + ");}s.push_str(name);}}
    if let Some(k)=gui_key(v&255){
        let ptr=unsafe{forge::sys::imgui::igGetKeyName(k)};
        if !ptr.is_null(){let name=match v&255{80=>"<-",79=>"->",81=>"Down",82=>"Up",_=>unsafe{core::ffi::CStr::from_ptr(ptr)}.to_str().unwrap_or("?")};if !s.is_empty(){s.push_str(" + ");}s.push_str(name);}
    }
    if s.is_empty(){s.push('—');}s
}
#[cfg(feature="keyboard-input")]
pub fn key_down(v:u32)->bool{
    use forge::input::Input;
    valid_hid(v&255)&&unsafe{hid_down(v&255)}
        &&(v&CTRL!=0)==Input::is_ctrl_down()&&(v&SHIFT!=0)==Input::is_shift_down()&&(v&ALT!=0)==Input::is_alt_down()
}
#[cfg(feature="keyboard-input")]
fn keyboard_input()->u32{
    use forge::input::Input;
    let mods=if Input::is_ctrl_down(){CTRL}else{0}|if Input::is_shift_down(){SHIFT}else{0}|if Input::is_alt_down(){ALT}else{0};
    let key=(4..=103).filter(|n|valid_hid(*n)).find(|n|*n!=73&&unsafe{hid_down(*n)}).unwrap_or(0);
    mods|key
}
// Keep saved keyboard assignments for compatibility, but compile no keyboard
// event processing in normal builds. Input-source guards remain active for pads.
#[cfg(not(feature="keyboard-input"))]
fn keyboard_input()->u32{0}
#[cfg(not(feature="keyboard-input"))]
fn key_label(_:u32)->String{String::new()}
#[cfg(not(feature="keyboard-input"))]
pub fn keyboard_events(_:bool)->u32{0}
static mut CAPTURE:Capture=Capture::EMPTY;
static mut GUARD:KeyboardGuard=KeyboardGuard::new();
#[cfg(feature="keyboard-input")]
static mut KEY_EDGES:Edges=Edges::new();
fn sample_capture()->u32{let raw=unsafe{if CAPTURE.target<=COUNT as u8{let mut g=GUARD;let v=g.filter(RAW.load(Ordering::Acquire),keyboard_active());GUARD=g;v}else{keyboard_input()}};unsafe{let mut c=CAPTURE;c.sample(raw);CAPTURE=c;}raw}
#[cfg(feature="keyboard-input")]
pub fn keyboard_events(blocked:bool)->u32{let keys=STATE.with(|s|s.list.key_bindings);let mut current=0;for (i,key) in keys.iter().enumerate(){if key_down(*key){current|=1<<i;}}
let edges=unsafe{let mut e=KEY_EDGES;let bits=e.step(current);if blocked{e.block();}KEY_EDGES=e;bits};if blocked{0}else{events(edges)}}
pub fn language(lang:usize)->usize{[1,3,4,2,0,5,6,7,8][lang.min(8)]}
pub fn action_label(index:usize,lang:usize)->&'static str{use crate::locales as l;match index{0=>l::TITLE[lang],1=>l::REGISTER_ACTION[lang],_=>l::REGISTER_ACTION[lang]}}
pub fn ui(){
 let mut list=STATE.with(|s|s.list.clone());let lang=list.language;let tr=language(lang);
 let category=CString::new(alloc::format!("{}##wishlist-bindings",if cfg!(feature="keyboard-input"){crate::locales::BINDINGS_TITLE[lang]}else{crate::locales::PAD_BINDINGS_TITLE[lang]})).unwrap();
 if imgui::tree_node(&category){
 let devices=if cfg!(feature="keyboard-input"){2}else{1};
 for d in 0..devices{imgui::text(if d==0{crate::locales::BINDINGS_CONTROLLER[lang]}else{crate::locales::BINDINGS_KEYBOARD[lang]});for i in 0..COUNT{let target=(d*COUNT+i+1) as u8;unsafe{forge::sys::imgui::igPushID_Int(target as i32);}
 let title=CString::new(if i==1{alloc::format!("{} / {}",crate::locales::REGISTER[lang],crate::locales::QUICK_REMOVE[lang])}else{action_label(i,lang).into()}).unwrap();if imgui::button(&title,ImVec2{x:0.,y:0.}){unsafe{CAPTURE=Capture{target,..Capture::EMPTY};GUARD=KeyboardGuard::default();}CAPTURING.store(true,Ordering::Release);RAW.store(0,Ordering::Release);}
 imgui::same_line(0.,-1.);imgui::text(&if d==0{label(list.pad_bindings[i])}else{key_label(list.key_bindings[i])});unsafe{forge::sys::imgui::igPopID();}
 }}imgui::tree_pop();}
 let title=c"Key binding##wishlist";
 if capturing()&&!unsafe{forge::sys::imgui::igIsPopupOpen_Str(title.as_ptr(),forge::sys::imgui::ImGuiPopupFlags::None)}{imgui::open_popup(title,forge::sys::imgui::ImGuiPopupFlags::None);}
 center_capture();if imgui::begin_popup_modal(title,None,forge::sys::imgui::ImGuiWindowFlags::AlwaysAutoResize|forge::sys::imgui::ImGuiWindowFlags::NoNavInputs){capture_instructions(tr);capture_text(tr,None);let c=unsafe{CAPTURE};
 if c.has_binding(){let confirm=CString::new(crate::locales::CONFIRM[lang]).unwrap();if imgui::button(&confirm,ImVec2{x:0.,y:0.}){let index=(c.target as usize-1)%COUNT;if c.target<=COUNT as u8{list.pad_bindings[index]=c.candidate;}else{list.key_bindings[index]=c.candidate;}crate::ui::apply_preferences(&list);finish();imgui::close_current_popup();}imgui::same_line(0.,-1.);}
 let cancel=CString::new(crate::locales::BACK[lang]).unwrap();if imgui::button(&cancel,ImVec2{x:0.,y:0.}){finish();imgui::close_current_popup();}imgui::end_popup();}
}
fn finish(){unsafe{CAPTURE=Capture::EMPTY;#[cfg(feature="keyboard-input")]{let mut edges=Edges::new();edges.block();KEY_EDGES=edges;}}CAPTURING.store(false,Ordering::Release);}
fn capture_text(language:usize,live:Option<u32>){
    let binding=["등록할 조합", "New binding", "登録する組み合わせ", "待绑定组合", "待綁定組合", "Neue Belegung", "Nouvelle combinaison", "Nuova combinazione", "Nueva combinación"];
    let capture=unsafe{CAPTURE};
    let show=|value|if value==0{"-".into()}else if capture.target<=COUNT as u8{label(value)}else{key_label(value)};
    if let Some(raw)=live{
        let labels=["현재 입력","Live input","現在の入力","当前输入","目前輸入","Aktuelle Eingabe","Entrée actuelle","Input attuale","Entrada actual"];
        imgui::text(&alloc::format!("{}: {}",labels[language],show(raw)));
    }else if !capture.has_binding(){return}
    imgui::text(&alloc::format!("{}: {}",binding[language],show(capture.candidate)));
}
fn center_capture(){
    let viewport=unsafe{forge::sys::imgui::igGetMainViewport()};
    let display=imgui::get_io().DisplaySize;
    let (pos,size)=if viewport.is_null(){(ImVec2{x:0.,y:0.},display)}else{unsafe{((*viewport).Pos,(*viewport).Size)}};
    imgui::set_next_window_pos(ImVec2{x:pos.x+size.x*0.5,y:pos.y+size.y*0.5},forge::sys::imgui::ImGuiCond::Always,ImVec2{x:0.5,y:0.5});
}
fn capture_instructions(language:usize){
    let text=["Insert를 눌러 창을 닫고 조합을 입력한 뒤 다시 Insert를 눌러 키 바인딩을 완료하세요.","Press Insert to close the window, enter your combination, then press Insert again to complete the key binding.","Insert でウィンドウを閉じ、組み合わせを入力した後、もう一度 Insert を押してキー割り当てを完了してください。","按 Insert 关闭窗口，输入组合键后，再按 Insert 完成按键绑定。","按 Insert 關閉視窗，輸入組合鍵後，再按 Insert 完成按鍵綁定。","Mit Insert das Fenster schließen, die Kombination eingeben und erneut Insert drücken, um die Tastenbelegung abzuschließen.","Appuyez sur Insert pour fermer la fenêtre, saisissez la combinaison, puis appuyez à nouveau sur Insert pour terminer l’affectation.","Premi Insert per chiudere la finestra, inserisci la combinazione, poi premi di nuovo Insert per completare l’assegnazione.","Pulsa Insert para cerrar la ventana, introduce la combinación y vuelve a pulsar Insert para completar la asignación."];
    let pad=["버튼을 다시 설정하려면 Insert 키를 누르세요. 현재 설정을 적용하려면 확인 버튼을 누르세요.","Press Insert to rebind the buttons. Click Confirm to apply the current binding.","ボタンを再設定するには Insert を押してください。現在の設定を適用するには確定ボタンを押してください。","按 Insert 重新设置按钮。点击确认应用当前设置。","按 Insert 重新設定按鈕。點擊確認套用目前設定。","Insert drücken, um die Tasten neu zu belegen. Zum Übernehmen auf Bestätigen klicken.","Appuyez sur Insert pour réaffecter les boutons. Cliquez sur Confirmer pour appliquer la combinaison actuelle.","Premi Insert per riassegnare i pulsanti. Premi Conferma per applicare la combinazione attuale.","Pulsa Insert para reasignar los botones. Pulsa Confirmar para aplicar la combinación actual."];
    let keyboard=["키를 다시 설정하려면 Insert 키를 누르세요. 현재 설정을 적용하려면 확인 버튼을 누르세요.","Press Insert to rebind the keys. Click Confirm to apply the current binding.","キーを再設定するには Insert を押してください。現在の設定を適用するには確定ボタンを押してください。","按 Insert 重新设置按键。点击确认应用当前设置。","按 Insert 重新設定按鍵。點擊確認套用目前設定。","Insert drücken, um die Tastaturbelegung zu ändern. Zum Übernehmen auf Bestätigen klicken.","Appuyez sur Insert pour réaffecter les touches. Cliquez sur Confirmer pour appliquer la combinaison actuelle.","Premi Insert per riassegnare i tasti. Premi Conferma per applicare la combinazione attuale.","Pulsa Insert para reasignar las teclas. Pulsa Confirmar para aplicar la combinación actual."];
    let capture=unsafe{CAPTURE};
    let message=if capture.has_binding(){if capture.target<=COUNT as u8{pad[language]}else{keyboard[language]}}else{text[language]};
    let width=(imgui::get_io().DisplaySize.x-48.).clamp(180.,560.);
    imgui::push_text_wrap_pos(imgui::get_cursor_pos().x+width);
    imgui::text(message);imgui::pop_text_wrap_pos();
}
/// Always visible during capture, including while Forge's settings are closed.
pub fn capture_overlay(language:usize){
    if unsafe{CAPTURE.target==0}||forge::graphics::Graphics::is_menu_visible(){return}
    let raw=sample_capture();
    use forge::sys::imgui::ImGuiWindowFlags as F;
    let pushed=crate::ui::push_binding_font();
    center_capture();
    if imgui::begin(c"Wishlist binding##capture-overlay",None,F::AlwaysAutoResize|F::NoInputs|F::NoSavedSettings){
        let pad=["원하는 버튼을 누르세요. Insert를 눌러 설정을 완료해주세요.","Press the desired buttons. Press Insert to finish setup.","設定したいボタンを押してください。Insert で設定を完了してください。","请按下所需按钮。按 Insert 完成设置。","請按下所需按鈕。按 Insert 完成設定。","Gewünschte Tasten drücken. Mit Insert die Einrichtung abschließen.","Appuyez sur les boutons souhaités. Appuyez sur Insert pour terminer.","Premi i pulsanti desiderati. Premi Insert per completare la configurazione.","Pulsa los botones deseados. Pulsa Insert para finalizar."];
        let keyboard=["원하는 키를 누르세요. Insert를 눌러 설정을 완료해주세요.","Press the desired keys. Press Insert to finish setup.","設定したいキーを押してください。Insert で設定を完了してください。","请按下所需按键。按 Insert 完成设置。","請按下所需按鍵。按 Insert 完成設定。","Gewünschte Tastaturtasten drücken. Mit Insert die Einrichtung abschließen.","Appuyez sur les touches souhaitées. Appuyez sur Insert pour terminer.","Premi i tasti desiderati. Premi Insert per completare la configurazione.","Pulsa las teclas deseadas. Pulsa Insert para finalizar."];
        imgui::push_text_wrap_pos(imgui::get_cursor_pos().x+(imgui::get_io().DisplaySize.x-48.).clamp(180.,560.));
        imgui::text(if unsafe{CAPTURE.target<=COUNT as u8}{pad[language]}else{keyboard[language]});
        imgui::pop_text_wrap_pos();
        capture_text(language,Some(raw));
    }imgui::end();
    if pushed{imgui::pop_font();}
}

// Input hooks can run before Forge refreshes its keyboard cache. Check again on
// the render thread, and discard queued pad events until both devices release.
static mut RENDER_GUARD:KeyboardGuard=KeyboardGuard::new();
pub fn pad_shortcuts_allowed()->bool{let keyboard=keyboard_active();unsafe{let mut guard=RENDER_GUARD;let blocked=keyboard||guard.waiting();guard.filter(RAW.load(Ordering::Acquire),keyboard);RENDER_GUARD=guard;!blocked}}
