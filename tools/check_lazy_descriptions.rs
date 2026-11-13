//! Exercise demand reads, bounded LRU, damaged resources and SDK handle lifetime.
extern crate alloc;
#[path="../src/text_blob.rs"]mod text_blob;
#[path="../src/language_pack.rs"]mod language_pack;
#[path="../src/description_file.rs"]mod description_file;
#[path="language_test_source.rs"]mod language_test_source;
use std::{sync::{Arc,Mutex,atomic::{AtomicUsize,Ordering}},fs::File,io::{Read,Seek,SeekFrom},ffi::{CStr,c_char,c_void}};
struct Stats{data:Vec<u8>,reads:Vec<(u64,usize)>,fail:bool}
struct Tracked{stats:Arc<Mutex<Stats>>,drops:Arc<AtomicUsize>}
impl language_pack::Source for Tracked{
 fn len(&self)->u64{self.stats.lock().unwrap().data.len() as u64}
 fn read(&mut self,offset:u64,bytes:&mut[u8])->Result<(),()>{let mut s=self.stats.lock().unwrap();s.reads.push((offset,bytes.len()));if s.fail{return Err(())}bytes.copy_from_slice(s.data.get(offset as usize..offset as usize+bytes.len()).ok_or(())?);Ok(())}
}
impl Drop for Tracked{fn drop(&mut self){self.drops.fetch_add(1,Ordering::SeqCst);}}
fn word(bytes:&[u8],at:usize)->usize{u32::from_le_bytes(bytes[at..at+4].try_into().unwrap()) as usize}
fn fixture()->(Vec<u8>,Arc<Mutex<Stats>>,Arc<AtomicUsize>){(std::fs::read("assets/languages/names.en.bin").unwrap(),Arc::new(Mutex::new(Stats{data:std::fs::read("assets/languages/text.en.bin").unwrap(),reads:vec![],fail:false})),Arc::new(AtomicUsize::new(0)))}
fn source(stats:&Arc<Mutex<Stats>>,drops:&Arc<AtomicUsize>)->Box<dyn language_pack::Source>{Box::new(Tracked{stats:stats.clone(),drops:drops.clone()})}
fn key(stats:&Arc<Mutex<Stats>>,row:usize)->(u8,u16,usize){let s=stats.lock().unwrap();let k=word(&s.data,16+row*8);((k>>16) as u8,k as u16,word(&s.data,20+row*8))}
#[test]fn prepare_reads_only_index_and_visible_text_is_cached_with_bounded_memory(){
 let(names,stats,drops)=fixture();let mut pack=language_pack::Pack::prepare(0,names,source(&stats,&drops)).unwrap();let (table,id,offset)=key(&stats,0);
 let prefix=16+word(&stats.lock().unwrap().data,12)*8;
 assert_eq!(stats.lock().unwrap().reads,vec![(0,prefix)]);let before=pack.bytes();let first=pack.text(true,table,id,0);assert!(!first.is_empty());
 assert_eq!(stats.lock().unwrap().reads[1],(offset as u64,8));assert_eq!(stats.lock().unwrap().reads[2],(offset as u64+8,first.len()));
 for _ in 0..1000{assert_eq!(pack.text(true,table,id,0),first);}assert_eq!(stats.lock().unwrap().reads.len(),3);
 assert!(pack.text(true,table,id,1).is_empty());assert_eq!(stats.lock().unwrap().reads.len(),3);
 // More unique descriptions than the cache limit must evict old entries.
 for row in 1..=language_pack::CACHE_ENTRIES{let(t,id,_)=key(&stats,row);assert!(!pack.text(true,t,id,0).is_empty());}
 let count=stats.lock().unwrap().reads.len();assert_eq!(pack.text(true,table,id,0),first);assert_eq!(stats.lock().unwrap().reads.len(),count+2);
 for row in 25..300{let(t,id,_)=key(&stats,row);pack.text(true,t,id,0);}
 assert!(pack.bytes()-before<=language_pack::CACHE_BYTES+4096);
 assert!(pack.text(true,255,65535,0).is_empty());let count=stats.lock().unwrap().reads.len();assert!(pack.text(true,255,65535,0).is_empty());assert_eq!(stats.lock().unwrap().reads.len(),count);
 drop(pack);assert_eq!(drops.load(Ordering::SeqCst),1);assert!(!first.is_empty(),"returned text must own its bytes");
}
#[test]fn corrupt_text_short_reads_and_oversized_lengths_are_omitted_without_frame_retry(){
 for fault in 0..5{
  let(names,stats,drops)=fixture();let mut pack=language_pack::Pack::prepare(0,names,source(&stats,&drops)).unwrap();let(t,id,at)=key(&stats,0);
  {let mut s=stats.lock().unwrap();match fault{
   0=>s.data[at+8]^=1,
   1=>s.data[at..at+4].copy_from_slice(&u32::MAX.to_le_bytes()),
   2=>s.fail=true,
   3=>{let n=word(&s.data,at);s.data[at+8]=255;let crc=language_pack::crc32(&s.data[at+8..at+8+n]);s.data[at+4..at+8].copy_from_slice(&crc.to_le_bytes());},
   _=>{let len=s.data.len();s.data[at..at+4].copy_from_slice(&(len as u32).to_le_bytes());}
  }}
  assert!(pack.text(true,t,id,0).is_empty());assert_eq!(pack.failed_reads,1);let reads=stats.lock().unwrap().reads.len();
  for _ in 0..100{assert!(pack.text(true,t,id,0).is_empty());}assert_eq!(stats.lock().unwrap().reads.len(),reads);drop(pack);assert_eq!(drops.load(Ordering::SeqCst),1);
 }
}
#[test]fn index_corruption_and_prepare_failure_close_sources_and_keep_old_pack(){
 let mut store=language_pack::Store::default();store.commit(language_test_source::prepare(4).unwrap());let old=store.text(false,1,2,4);
 for fault in 0..4{let(names,stats,drops)=fixture();{let mut s=stats.lock().unwrap();match fault{0=>s.data[0]^=1,1=>{s.data.pop();},2=>s.data[20..24].copy_from_slice(&u32::MAX.to_le_bytes()),_=>s.fail=true}}
  assert!(language_pack::Pack::prepare(0,names,source(&stats,&drops)).is_err());assert_eq!(drops.load(Ordering::SeqCst),1);assert_eq!(store.text(false,1,2,4),old);
 }
 let(names,stats,drops)=fixture();store.commit(language_pack::Pack::prepare(0,names,source(&stats,&drops)).unwrap());drop(store.commit(language_test_source::prepare(3).unwrap()));assert_eq!(drops.load(Ordering::SeqCst),1);
}

// Run the production ARM SDK wrapper against real host files. ABI signatures
// match storage.rs; the ARM cargo check also verifies the actual target layout.
static SDK:Mutex<Option<File>>=Mutex::new(None);
static CLOSES:AtomicUsize=AtomicUsize::new(0);
#[unsafe(export_name="_ZN2nn2fs8OpenFileEPNS0_10FileHandleEPKci")]
unsafe extern "C" fn sdk_open(handle:*mut u32,path:*const c_char,mode:i32)->u32{assert_eq!(mode,1);let path=unsafe{CStr::from_ptr(path)}.to_str().unwrap();let Ok(file)=File::open(path)else{return 1};let mut slot=SDK.lock().unwrap();assert!(slot.is_none());*slot=Some(file);unsafe{*handle=1};0}
#[unsafe(export_name="_ZN2nn2fs11GetFileSizeEPxNS0_10FileHandleE")]
unsafe extern "C" fn sdk_size(bytes:*mut i64,handle:u32)->u32{assert_eq!(handle,1);unsafe{*bytes=SDK.lock().unwrap().as_ref().unwrap().metadata().unwrap().len() as i64};0}
#[unsafe(export_name="_ZN2nn2fs8ReadFileENS0_10FileHandleExPvj")]
unsafe extern "C" fn sdk_read(handle:u32,offset:i64,bytes:*mut c_void,length:usize)->u32{assert_eq!(handle,1);let mut slot=SDK.lock().unwrap();let file=slot.as_mut().unwrap();if file.seek(SeekFrom::Start(offset as u64)).is_err(){return 1}if file.read_exact(unsafe{std::slice::from_raw_parts_mut(bytes.cast(),length)}).is_ok(){0}else{1}}
#[unsafe(export_name="_ZN2nn2fs9CloseFileENS0_10FileHandleE")]
unsafe extern "C" fn sdk_close(handle:u32){assert_eq!(handle,1);assert!(SDK.lock().unwrap().take().is_some());CLOSES.fetch_add(1,Ordering::SeqCst);}
#[test]fn actual_sdk_reader_fetches_exact_ranges_and_closes_on_drop(){
 use language_pack::Source;
 assert!(description_file::File::open(c"assets/languages/missing.bin").is_err());
 let mut file=description_file::File::open(c"assets/languages/text.en.bin").unwrap();let mut bytes=[0;16];file.read(0,&mut bytes).unwrap();assert_eq!(&bytes[..4],b"WLD1");assert!(file.read(file.len(),&mut bytes).is_err());
 drop(file);assert_eq!(CLOSES.load(Ordering::SeqCst),1);
 let names=std::fs::read("assets/languages/names.en.bin").unwrap();let file=description_file::File::open(c"assets/languages/text.en.bin").unwrap();let mut pack=language_pack::Pack::prepare(0,names,Box::new(file)).unwrap();let expected=text_blob::get(include_bytes!("../assets/ui-names.bin"),31,1,0);assert_eq!(pack.text(true,31,1,0),expected);drop(pack);assert_eq!(CLOSES.load(Ordering::SeqCst),2);
}
