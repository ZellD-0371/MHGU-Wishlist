//! Returned strings own their bytes; a language swap cannot leave dangling text.
#[cfg(target_arch="arm")]
pub fn get(table:u8,index:usize,lang:usize)->alloc::string::String{if index>u16::MAX as usize{return alloc::string::String::new()}crate::language::text(false,table,index as u16,lang)}
#[cfg(not(target_arch="arm"))]
pub fn get(table:u8,index:usize,lang:usize)->&'static str{if index>u16::MAX as usize{return ""}crate::text_blob::get(include_bytes!("../assets/ui-names.bin"),table,index as u16,lang)}
#[cfg(target_arch="arm")]
pub fn material_description(id:u16,lang:usize)->alloc::string::String{get(31,id as usize,lang)}
#[cfg(not(target_arch="arm"))]
pub fn material_description(id:u16,lang:usize)->&'static str{get(31,id as usize,lang)}
