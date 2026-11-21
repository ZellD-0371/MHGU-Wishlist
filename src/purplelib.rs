//! Purple-lib: the settings under Options > Mod Options > Wishlist.
//!
//! Optional. Purple-lib calls `purplelib_on_register` (found by name in the exported symbols)
//! once from its first update; without Purple-lib nothing here runs and the Forge menu stays
//! the only way to change the settings. Changes apply at once through the same path as the
//! Forge menu, and are saved to the same INI as its Save button after the Options window
//! closes. Records and API follow Purple-lib's purplelib.h, API version 4 (Switch layout:
//! 4-byte pointers). Key bindings stay in the Forge menu (they need a capture dialog).
use core::{ffi::{c_char,c_void},sync::atomic::{AtomicBool,Ordering}};
use forge::log::prelude::{info,warn};
use crate::STATE;

const API_VERSION:u32=4;
const ROW_LABELS:u32=0;const ROW_NUMBERS:u32=1;
const NO_WRAP:u32=1;

#[repr(C)]
pub struct Row{
    name:*const c_char,help:*const c_char,labels:*const *const c_char,label_count:u32,kind:u32,
    value_count:u32,first:i32,step:i32,suffix:*const c_char,flags:u32,
}
#[repr(C)]
pub struct Options{
    size:u32,context:*mut c_void,
    title:Option<unsafe extern "C" fn(*mut c_void,u32)->*const c_char>,
    row_count:u32,
    row:Option<unsafe extern "C" fn(*mut c_void,u32,u32,*mut Row)->bool>,
    get:Option<unsafe extern "C" fn(*mut c_void,u32)->u32>,
    set:Option<unsafe extern "C" fn(*mut c_void,u32,u32)>,
    closed:Option<unsafe extern "C" fn(*mut c_void)>,
    value_text:Option<unsafe extern "C" fn(*mut c_void,u32,u32,u32)->*const c_char>,
    action:Option<unsafe extern "C" fn(*mut c_void,u32)>,
}
#[repr(C)]
pub struct Help{
    size:u32,context:*mut c_void,category:u32,page_count:u32,
    title:Option<unsafe extern "C" fn(*mut c_void,u32)->*const c_char>,
    page:Option<unsafe extern "C" fn(*mut c_void,u32,u32)->*const c_char>,
    open:Option<unsafe extern "C" fn(*mut c_void)>,
}
#[repr(C)]
pub struct Api{
    version:u32,
    add_options:Option<unsafe extern "C" fn(*const Options)->bool>,
    add_help:Option<unsafe extern "C" fn(*const Help)->bool>,
    hold_input:Option<unsafe extern "C" fn()>,
    options_shown:Option<unsafe extern "C" fn(*const Options)->bool>,
    options_row:Option<unsafe extern "C" fn(*const Options)->i32>,
    add_set_help:Option<unsafe extern "C" fn(*const Options,*const Help)->bool>,
    help_shown:Option<unsafe extern "C" fn(*const Options)->bool>,
}
#[cfg(target_arch="arm")]
const _:()={
    assert!(core::mem::size_of::<Options>()==40);assert!(core::mem::size_of::<Row>()==40);
    assert!(core::mem::size_of::<Help>()==28);assert!(core::mem::size_of::<Api>()==32);
};

/// A string the game may keep a pointer to: Purple-lib's rule is that it lives forever.
struct Str(*const c_char);
unsafe impl Sync for Str {}
macro_rules! s{($t:literal)=>{Str(concat!($t,"\0").as_ptr() as *const c_char)}}
struct Labels<const N:usize>([*const c_char;N]);
unsafe impl<const N:usize> Sync for Labels<N> {}
macro_rules! labels{($($t:literal),*)=>{Labels([$(concat!($t,"\0").as_ptr() as *const c_char),*])}}

const ROW_COUNT:usize=4;
/// One language's text for the page. The language row lists the mod's languages in
/// List::language order (the Forge menu's), named in the page's language.
struct Text{names:Labels<ROW_COUNT>,helps:Labels<ROW_COUNT>,on_off:Labels<2>,languages:Labels<9>,pages:Labels<2>}
/// The page's text in the mod's language order (List::language).
static TEXT:[Text;9]=[
    // en
    Text{
        names:labels!("Button layout","Language","UI scale","Logging"),
        helps:labels!("Controller style of the button icons.","The language of the wishlist window.","Size of the wishlist window.","Write diagnostic records to the log."),
        on_off:labels!("Off","On"),
        languages:labels!("English","Chinese (S)","Chinese (T)","Japanese","Korean","German","French","Italian","Spanish"),
        pages:labels!("Press R3 at the Smithy to add equipment,\r\nL3+R3 to open the list. In the village\r\nand hall it is also on page 3 of the\r\ngame menu.","Changes show at once. They are saved to\r\nwishlist.ini when you close the window.\r\nKey bindings are in the Forge menu."),
    },
    // zh-Hans
    Text{
        names:labels!("按键布局","语言","界面大小","日志记录"),
        helps:labels!("按键图标使用的手柄样式。","愿望清单窗口的语言。","愿望清单窗口的大小。","将诊断记录写入日志。"),
        on_off:labels!("关","开"),
        languages:labels!("英语","简体中文","繁体中文","日语","韩语","德语","法语","意大利语","西班牙语"),
        pages:labels!("在加工商处按R3登记装备，按住L3再按\r\nR3打开清单。在村庄和集会所也可从\r\n游戏菜单第3页打开。","更改会立即生效，关闭窗口时保存到\r\nwishlist.ini。按键设置请在Forge\r\n菜单中更改。"),
    },
    // zh-Hant
    Text{
        names:labels!("按鍵配置","語言","介面大小","日誌記錄"),
        helps:labels!("按鍵圖示使用的手把樣式。","願望清單視窗的語言。","願望清單視窗的大小。","將診斷記錄寫入日誌。"),
        on_off:labels!("關","開"),
        languages:labels!("英文","簡體中文","繁體中文","日文","韓文","德文","法文","義大利文","西班牙文"),
        pages:labels!("在加工商處按R3登記裝備，按住L3再按\r\nR3開啟清單。在村莊和集會所也可從\r\n遊戲選單第3頁開啟。","變更會立即生效，關閉視窗時儲存到\r\nwishlist.ini。按鍵設定請在Forge\r\n選單中變更。"),
    },
    // ja
    Text{
        names:labels!("ボタン配置","言語","UIサイズ","ログ記録"),
        helps:labels!("ボタンアイコンのコントローラー種類です。","ウィッシュリスト画面の言語です。","ウィッシュリスト画面の大きさです。","調査用の診断記録をログに残します。"),
        on_off:labels!("オフ","オン"),
        languages:labels!("英語","中国語（簡体）","中国語（繁体）","日本語","韓国語","ドイツ語","フランス語","イタリア語","スペイン語"),
        pages:labels!("加工屋でR3を押すと装備を登録し、L3を\r\n押しながらR3でリストを開きます。村と\r\n集会所ではゲームメニューの3ページ\r\nからも開けます。","変更はすぐに反映されます。ウィンドウを\r\n閉じるとwishlist.iniに保存されます。\r\nキー設定はForgeメニューで変更します。"),
    },
    // ko
    Text{
        names:labels!("버튼 배치","언어","UI 크기","로그 기록"),
        helps:labels!("버튼 아이콘을 이 컨트롤러 모양으로 표시합니다.","찜 목록 창에 쓸 언어입니다.","찜 목록 창의 크기입니다.","문제 조사용 진단 기록을 로그에 남깁니다."),
        on_off:labels!("끄기","켜기"),
        languages:labels!("영어","중국어(간체)","중국어(번체)","일본어","한국어","독일어","프랑스어","이탈리아어","스페인어"),
        pages:labels!("가공점에서 R3로 장비를 찜하고, L3를\r\n누른 채 R3로 목록을 엽니다. 마을과\r\n집회소에서는 게임 메뉴 3페이지에서도\r\n열 수 있습니다.","변경 내용은 바로 반영됩니다. 창을 닫으면\r\nwishlist.ini에 저장됩니다. 키 설정은\r\nForge 메뉴에서 바꿉니다."),
    },
    // de
    Text{
        names:labels!("Tastenlayout","Sprache","UI-Größe","Protokollierung"),
        helps:labels!("Controller-Stil der Tastensymbole.","Sprache des Wunschlistenfensters.","Größe des Wunschlistenfensters.","Schreibt Diagnosedaten ins Protokoll."),
        on_off:labels!("Aus","An"),
        languages:labels!("Englisch","Chinesisch (V)","Chinesisch (T)","Japanisch","Koreanisch","Deutsch","Französisch","Italienisch","Spanisch"),
        pages:labels!("R3 in der Schmiede merkt Ausrüstung vor,\r\nL3+R3 öffnet die Liste. Im Dorf und in\r\nder Halle auch auf Seite 3 des Menüs.","Änderungen gelten sofort und werden beim\r\nSchließen in wishlist.ini gespeichert.\r\nTasten stellst du im Forge-Menü ein."),
    },
    // fr
    Text{
        names:labels!("Disposition","Langue","Taille de l'interface","Journalisation"),
        helps:labels!("Style de manette des icônes de boutons.","Langue de la fenêtre de la liste.","Taille de la fenêtre de la liste.","Écrit des données de diagnostic dans le journal."),
        on_off:labels!("Non","Oui"),
        languages:labels!("Anglais","Chinois (S)","Chinois (T)","Japonais","Coréen","Allemand","Français","Italien","Espagnol"),
        pages:labels!("R3 à la forge ajoute l'équipement, L3+R3\r\nouvre la liste. Au village et au hall,\r\naussi en page 3 du menu du jeu.","Les changements s'appliquent aussitôt et\r\nsont enregistrés dans wishlist.ini à la\r\nfermeture. Touches : menu Forge."),
    },
    // it
    Text{
        names:labels!("Layout pulsanti","Lingua","Scala interfaccia","Registrazione log"),
        helps:labels!("Stile del controller per le icone dei pulsanti.","Lingua della finestra della lista.","Dimensione della finestra della lista.","Scrive dati diagnostici nel log."),
        on_off:labels!("No","Sì"),
        languages:labels!("Inglese","Cinese (S)","Cinese (T)","Giapponese","Coreano","Tedesco","Francese","Italiano","Spagnolo"),
        pages:labels!("R3 dal fabbro aggiunge l'equipaggiamento,\r\nL3+R3 apre la lista. Al villaggio e nella\r\nsala anche a pagina 3 del menu di gioco.","Le modifiche valgono subito e vengono\r\nsalvate in wishlist.ini alla chiusura.\r\nI tasti si cambiano nel menu Forge."),
    },
    // es
    Text{
        names:labels!("Diseño de botones","Idioma","Escala de interfaz","Registro"),
        helps:labels!("Estilo de mando de los iconos de botones.","Idioma de la ventana de la lista.","Tamaño de la ventana de la lista.","Escribe datos de diagnóstico en el registro."),
        on_off:labels!("No","Sí"),
        languages:labels!("Inglés","Chino (S)","Chino (T)","Japonés","Coreano","Alemán","Francés","Italiano","Español"),
        pages:labels!("R3 en la herrería añade el equipo, L3+R3\r\nabre la lista. En la aldea y el salón,\r\ntambién en la página 3 del menú.","Los cambios se aplican al instante y se\r\nguardan en wishlist.ini al cerrar.\r\nLas teclas se cambian en el menú Forge."),
    },
];
/// Purple-lib passes the game's language (MT Framework order: 0 Japanese, 1 English,
/// 2 French, 3 Spanish, 4 German, 5 Italian, 6 Korean, 7 Chinese (T), 8 Chinese (S);
/// 6 under the Korean patch). The game's font has those glyphs, so the page follows the
/// game, not the mod's own language. Unknown numbers fall back to English.
fn text(language:u32)->&'static Text{
    const MOD_LANGUAGE:[usize;9]=[3,0,6,8,5,7,4,2,1];
    &TEXT[MOD_LANGUAGE.get(language as usize).copied().unwrap_or(0)]
}

// Same order as the Forge menu. The layout names are the same in every language.
const LAYOUT:usize=0;const LANGUAGE:usize=1;const SCALE:usize=2;const LOGGING:usize=3;
static LAYOUTS:Labels<3>=labels!("Nintendo","Xbox","PlayStation");
static PERCENT:Str=s!("%");
// UI scale: 50-100% in steps of 10, as in the Forge menu.
const SCALE_FIRST:i32=50;const SCALE_STEP:i32=10;const SCALE_COUNT:u32=6;

/// Set when the Options window closes; the next update saves once no language change is pending.
static SAVE:AtomicBool=AtomicBool::new(false);

/// The mod's own name in every language, so it reads the same as in mod lists.
unsafe extern "C" fn title(_:*mut c_void,_:u32)->*const c_char{c"Wishlist".as_ptr()}

unsafe extern "C" fn row(_:*mut c_void,language:u32,index:u32,out:*mut Row)->bool{
    let index=index as usize;
    let Some(out)=(unsafe{out.as_mut()}) else{return false};
    if index>=ROW_COUNT{return false}
    let t=text(language);
    out.name=t.names.0[index];out.help=t.helps.0[index];
    match index{
        LAYOUT=>{out.kind=ROW_LABELS;out.labels=LAYOUTS.0.as_ptr();out.label_count=3;}
        LANGUAGE=>{out.kind=ROW_LABELS;out.labels=t.languages.0.as_ptr();out.label_count=9;}
        SCALE=>{out.kind=ROW_NUMBERS;out.value_count=SCALE_COUNT;out.first=SCALE_FIRST;out.step=SCALE_STEP;out.suffix=PERCENT.0;out.flags=NO_WRAP;}
        _=>{out.kind=ROW_LABELS;out.labels=t.on_off.0.as_ptr();out.label_count=2;}
    }
    true
}

/// The preferences shown: a language change waiting for the next frame counts as made.
fn preferences()->(usize,usize,u16,bool){
    STATE.with(|s|{let l=s.pending_preferences.as_ref().unwrap_or(&s.list);(l.layout,l.language,l.ui_scale,l.diagnostics)})
}

unsafe extern "C" fn get(_:*mut c_void,index:u32)->u32{
    let (layout,language,scale,logging)=preferences();
    match index as usize{
        LAYOUT=>layout.min(2) as u32,
        LANGUAGE=>language.min(8) as u32,
        // Nearest step, so any value from the INI still shows.
        SCALE=>(((i32::from(scale)-SCALE_FIRST)*2+SCALE_STEP)/(2*SCALE_STEP)).clamp(0,SCALE_COUNT as i32-1) as u32,
        LOGGING=>u32::from(logging),
        _=>0,
    }
}

unsafe extern "C" fn set(_:*mut c_void,index:u32,value:u32){
    let index=index as usize;
    if index>=ROW_COUNT{return}
    let mut next=STATE.with(|s|s.pending_preferences.clone().unwrap_or_else(||s.list.clone()));
    match index{
        LAYOUT=>{if value>=3{return}next.layout=value as usize;}
        LANGUAGE=>{if value>=9{return}next.language=value as usize;}
        SCALE=>{if value>=SCALE_COUNT{return}next.ui_scale=(SCALE_FIRST+value as i32*SCALE_STEP) as u16;}
        _=>next.diagnostics=value!=0,
    }
    // The Forge menu's path: a language change loads its text before the next frame.
    crate::ui::apply_preferences(&next);
}

unsafe extern "C" fn closed(_:*mut c_void){SAVE.store(true,Ordering::Release);}

/// Called each update: save once the Options window closed and the language is in place.
pub fn update(){
    if !SAVE.load(Ordering::Acquire)||STATE.with(|s|s.pending_preferences.is_some()){return}
    SAVE.store(false,Ordering::Release);
    if !crate::ui::save_preferences(){warn!("[wishlist] Mod Options: settings could not be saved");}
}

struct Record<T>(T);
unsafe impl<T> Sync for Record<T> {}
static OPTIONS:Record<Options>=Record(Options{
    size:core::mem::size_of::<Options>() as u32,context:core::ptr::null_mut(),title:Some(title),
    row_count:ROW_COUNT as u32,row:Some(row),get:Some(get),set:Some(set),closed:Some(closed),value_text:None,action:None,
});

unsafe extern "C" fn help_page(_:*mut c_void,language:u32,index:u32)->*const c_char{
    text(language).pages.0.get(index as usize).copied().unwrap_or(core::ptr::null())
}
static SET_HELP:Record<Help>=Record(Help{
    size:core::mem::size_of::<Help>() as u32,context:core::ptr::null_mut(),category:0,
    page_count:2,title:Some(title),page:Some(help_page),open:None,
});

#[unsafe(no_mangle)]
pub unsafe extern "C" fn purplelib_on_register(api:*const Api){
    let Some(api)=(unsafe{api.as_ref()}) else{return};
    if api.version<API_VERSION{warn!("[wishlist] Purple-lib is too old: no Mod Options page");return}
    let Some(add)=api.add_options else{return};
    if unsafe{add(&OPTIONS.0)}{
        if let Some(help)=api.add_set_help{unsafe{help(&OPTIONS.0,&SET_HELP.0);}}
        info!("[wishlist] settings under Options > Mod Options");
    }else{
        warn!("[wishlist] Purple-lib refused the Mod Options page");
    }
}
