extern crate alloc;
#[path="../src/binding_model.rs"]mod binding_model;
#[path="../src/sorting.rs"]mod sorting;
#[path="../src/model.rs"]mod model;
#[path="../src/generated.rs"]mod generated;
#[path="../src/equipment.rs"]mod equipment;
#[path="../src/presentation.rs"]mod presentation;
#[path="../src/material_cache.rs"]mod material_cache;
#[path="../src/text_cache.rs"]mod text_cache;
use model::*;
use text_cache::*;
#[test]fn indexed_equipment_lookup_matches_original_for_all_entries_and_missing_levels(){
 for d in equipment::DETAILS{
  for level in [0,d.level,32,255]{let r=Recipe{kind:d.kind,id:d.id,level,..Default::default()};let old=equipment::DETAILS.iter().find(|v|v.kind==r.kind&&v.id==r.id&&(v.level==0||v.level==r.level.max(1)));
   assert_eq!(equipment::get(r).map(|p|p as *const _),old.map(|p|p as *const _),"kind={} id={} level={}",r.kind,r.id,level);
  }
 }
 for kind in 0..=255{for id in [0,65535]{assert!(equipment::get(Recipe{kind,id,..Default::default()}).is_none());}}
}
fn check_materials(cache:&mut material_cache::Cache,list:&List,stock:&[u32;4096],known:bool){
 cache.refresh(list,stock,known,generated::PROVISIONS);assert_eq!(cache.totals(),list.ordered_totals(stock,known));
 for(i,e)in list.slots.iter().enumerate(){if let Some(e)=e{assert_eq!(cache.ready(i,e.recipe),known.then(||e.recipe.ready(stock,generated::PROVISIONS)));}}
 for m in cache.totals(){let old=list.requiring_material(m.id).into_iter().fold(0u64,|bits,i|bits|(1u64<<i));assert_eq!(cache.related(m.id),old);}
}
#[test]fn material_cache_reuses_results_and_invalidates_inventory_recipe_sort_and_availability(){
 let mut list=List::default();for (slot,r) in generated::RECIPES.iter().filter(|r|r.valid()).step_by(317).take(48).enumerate(){list.register(slot,*r,1).unwrap();}
 let mut stock=[99;4096];let mut cache=material_cache::Cache::default();check_materials(&mut cache,&list,&stock,true);
 let before=list.encode_character();let pointer=cache.totals().as_ptr();for _ in 0..1000{assert!(!cache.refresh(&list,&stock,true,generated::PROVISIONS));assert_eq!(pointer,cache.totals().as_ptr());}
 assert_eq!(before,list.encode_character(),"display cache must not change saved order");
 stock.fill(0);cache.inventory_changed();check_materials(&mut cache,&list,&stock,true);assert!(!cache.refresh(&list,&stock,true,generated::PROVISIONS));
 check_materials(&mut cache,&list,&stock,false);check_materials(&mut cache,&list,&stock,true);
 let removed=list.slots[7].unwrap().recipe;list.remove_slot(7);check_materials(&mut cache,&list,&stock,true);assert!(list.slots[7].is_none());assert_eq!(cache.ready(7,removed),None);
 list.sort_display(2,3,&stock,true,generated::PROVISIONS).unwrap();check_materials(&mut cache,&list,&stock,true);
 let r=*generated::RECIPES.iter().find(|r|r.valid()&&list.registered(**r).is_none()).unwrap();list.register(47,r,123).unwrap();check_materials(&mut cache,&list,&stock,true);
 let encoded=list.encode_character();let loaded=List::decode(&encoded).unwrap();check_materials(&mut cache,&loaded,&stock,true);
 cache=Default::default();check_materials(&mut cache,&List::default(),&stock,true);assert!(cache.totals().is_empty());assert_eq!(cache.related(1),0);
}
#[test]fn metrics_cache_is_font_pixel_text_sensitive_and_evicts_with_limits(){
 let mut cache=MetricsCache::default();let key=MetricKey{font:1,pixels:25};let value=Metrics{width:32.,ink:Some([1.,31.,2.,21.])};let calls=std::cell::Cell::new(0);
 for _ in 0..1000{assert_eq!(cache.get(key,"日本語 한국어 abc",||{calls.set(calls.get()+1);value}),value);}assert_eq!(calls.get(),1);
 for key in [MetricKey{font:2,..key},MetricKey{pixels:30,..key}]{cache.get(key,"日本語 한국어 abc",||{calls.set(calls.get()+1);value});}assert_eq!(calls.get(),3);
 for i in 0..METRIC_LIMIT{cache.get(key,&format!("{i}"),||value);}cache.get(key,"日本語 한국어 abc",||{calls.set(calls.get()+1);value});assert_eq!(calls.get(),4);
 let huge="x".repeat(TEXT_BUDGET+1);for _ in 0..2{cache.get(key,&huge,||{calls.set(calls.get()+1);value});}assert_eq!(calls.get(),6);
 // Byte budget eviction must also occur before the entry-count limit is reached.
 let mut cache=MetricsCache::default();let large="a".repeat(TEXT_BUDGET);cache.get(key,&large,||value);cache.get(key,"b",||value);let mut rebuilt=false;cache.get(key,&large,||{rebuilt=true;value});assert!(rebuilt);
}
#[test]fn paragraph_cache_reuses_layout_and_invalidates_for_scale_geometry_font_or_text(){
 let mut cache=LayoutsCache::default();let key=LayoutKey{font:1,scale:1f32.to_bits(),width:320f32.to_bits(),height:60f32.to_bits(),size:25f32.to_bits()};let builds=std::cell::Cell::new(0);
 let build=||{builds.set(builds.get()+1);Layout{size:25.,lines:vec!["line 1".into(),"line 2".into()]}};
 for _ in 0..1000{cache.with(key,"description",build,|l|assert_eq!(l.lines.len(),2));}assert_eq!(builds.get(),1);
 for key in [LayoutKey{font:2,..key},LayoutKey{scale:2f32.to_bits(),..key},LayoutKey{width:300f32.to_bits(),..key},LayoutKey{height:70f32.to_bits(),..key},LayoutKey{size:24f32.to_bits(),..key}]{cache.with(key,"description",build,|_|());}assert_eq!(builds.get(),6);
 cache.with(key,"different language",build,|_|());assert_eq!(builds.get(),7);
 for i in 0..LAYOUT_LIMIT{cache.with(key,&format!("text {i}"),build,|_|());}let previous=builds.get();cache.with(key,"description",build,|_|());assert_eq!(builds.get(),previous+1);
 // Drawing a paragraph may safely request font metrics from the independent cache.
 text_cache::paragraph(key,"nested",build,|_|{text_cache::metrics(MetricKey{font:1,pixels:25},"line",Metrics::default);});text_cache::clear();
}
#[test]fn recently_used_metrics_survive_eviction_without_rebuilding(){
 let mut cache=MetricsCache::default();let key=MetricKey{font:1,pixels:25};
 for i in 0..METRIC_LIMIT{cache.get(key,&i.to_string(),Metrics::default);}
 cache.get(key,"0",||panic!("cache miss"));cache.get(key,"new",Metrics::default);
 cache.get(key,"0",||panic!("recently used entry was evicted"));let mut rebuilt=false;cache.get(key,"1",||{rebuilt=true;Metrics::default()});assert!(rebuilt);
}
