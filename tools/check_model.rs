#[path="../src/binding_model.rs"]mod binding_model;
#[path="../src/sorting.rs"]mod sorting;
#[path="../src/model.rs"]mod model;
#[path="../src/generated.rs"]mod generated;
#[path="../src/locales.rs"]mod locales;
#[path="../src/smithy.rs"]mod smithy;
#[path="../src/navigation.rs"]mod navigation;
#[path="../src/equipment.rs"]mod equipment;
#[path="../src/interaction.rs"]mod interaction;
#[path="../src/session.rs"]mod session;
#[path="../src/profiles.rs"]mod profiles;
use model::*;
#[path="../src/hook_guard.rs"]mod hook_guard;
#[test]fn shared_destructor_keeps_existing_forge_hook_chain(){
 let original=[0xe92d4010,0xe1a04000,0xe59f0084,0xe79f0000];
 assert!(hook_guard::matches(original,original,0x76da30));
 assert!(hook_guard::matches([0xe51ff004,0x03363d50,original[2],original[3]],original,0x76da30));
 assert!(!hook_guard::matches([0xe51ff004,0,original[2],original[3]],original,0x76da30));
 assert!(!hook_guard::matches([0xe51ff004,0x03363d51,original[2],original[3]],original,0x76da30));
 assert!(!hook_guard::matches([0xe51ff004,0x03363d50,0,original[3]],original,0x76da30));
 assert!(!hook_guard::matches([0xe92d4010,0,original[2],original[3]],original,0x76da30));
}
#[test]fn material_focus_preserves_equipment_cursor_and_returns(){
 let mut f=interaction::Focus::default();f.enter(0);assert!(!f.material);
 f.enter(4);assert!(f.material);assert_eq!(f.index,0);f.step(4,-1);assert_eq!(f.index,3);f.step(4,1);assert_eq!(f.index,0);assert!(f.back());assert!(!f.back());
 assert!(!f.yes);f.yes=true;assert!(f.yes);
}
#[test]fn opening_sound_setting_accepts_zero_and_old_files(){
 let old=List::decode("version=1\nlanguage=4\n").unwrap();assert_eq!(old.open_sound,1);
 for id in [0,1,7,123]{let mut list=old.clone();list.open_sound=id;assert_eq!(List::decode(&list.encode()).unwrap().open_sound,id);}
}
#[test]fn navigation_wraps_within_current_page(){
 for page in 0..6{assert_eq!(navigation::move_slot(page*8,-1,0),page*8+7);assert_eq!(navigation::move_slot(page*8+7,1,0),page*8);}
 assert_eq!(navigation::move_slot(7,0,-1),47);assert_eq!(navigation::move_slot(47,0,1),7);
}
#[test]fn saved_zenny_migrates_and_equipment_skills_exist(){
 let old="version=1\nlanguage=4\nslot0=2,4,0,0,0,318,2,1,1,0,0,0,0\n";
 let mut l=List::decode(old).unwrap();assert_eq!(l.costs[0],0);l.costs[0]=120000;let restored=List::decode(&l.encode()).unwrap();assert_eq!(restored.costs[0],120000);assert_eq!(l.slots[0].unwrap().recipe,restored.slots[0].unwrap().recipe);
 assert!(equipment::DETAILS.iter().any(|d|d.kind==1&&!d.skills.is_empty()));
 assert!(equipment::DETAILS.iter().any(|d|d.kind==25&&!d.skills.is_empty()));
 for d in equipment::DETAILS{assert!((1..=25).contains(&d.kind));for (id,_) in d.skills{for lang in 0..9{assert_ne!(equipment::skill(*id,lang),"?");}}}
}
fn native_row(r:Recipe)->smithy::SelectedRow{smithy::SelectedRow{list_kind:if r.kind==25{6}else{r.kind as u16},id:r.id,level_index:r.level.saturating_sub(1),group:r.group,points:r.points as u8,materials:r.materials}}
#[test]fn smithy_decorations_use_list_kind_and_equipment_id(){
 let recipes:Vec<_>=generated::RECIPES.iter().copied().filter(|r|r.kind==25).collect();assert!(!recipes.is_empty());
 for r in &recipes{let row=native_row(*r);assert_eq!(row.pane(),10);assert_eq!(row.resolve(&recipes,&[]),Some(*r));}
}
#[test]fn smithy_upgrade_category_is_not_a_decoration_id(){
 let recipes:Vec<_>=generated::RECIPES.iter().copied().filter(|r|r.kind>=7&&r.kind<=21&&r.level>0).collect();
 let mut categories=0;for r in &recipes{let row=native_row(*r);assert_eq!(row.resolve(&recipes,&[]),Some(*r),"{row:?}");if r.group>0{categories+=1;assert_eq!(row.pane(),11)}}assert!(categories>0);
}
#[test]fn smithy_rejects_mismatched_category_and_unrelated_kind(){
 let r=*generated::RECIPES.iter().find(|r|r.kind==7&&r.group>0&&r.level>0).unwrap();
 let mut row=native_row(r);row.group+=1;assert!(row.resolve(&[r],&[]).is_none());row=native_row(r);row.points+=1;assert!(row.resolve(&[r],&[]).is_none());row=native_row(r);row.list_kind=65535;assert!(row.resolve(&[r],&[]).is_none());
}
#[test]fn armor_upgrade_all_series_and_five_parts(){
 let mut series=std::collections::BTreeMap::<u16,Vec<Recipe>>::new();
 for r in generated::ARMOR_UPGRADES{assert_eq!(r.kind,0);assert!(Recipe{kind:1,..*r}.valid());series.entry(r.id).or_default().push(*r);}
 assert!(!series.is_empty());
 for recipes in series.values(){for r in recipes{for kind in 1..=5{
  let expected=Recipe{kind,..*r};let row=native_row(expected);
  assert_eq!(row.resolve(&[],recipes),Some(expected),"{row:?}");
 }}}
}
#[test]fn armor_upgrade_reported_series_446_and_part_identity(){
 let mut list=List::default();
 for kind in 1..=5{
  let row=smithy::SelectedRow{list_kind:kind,id:446,level_index:4,group:198,points:6,materials:[Material{id:1216,count:1},Material::default(),Material::default(),Material::default()]};
  let r=row.resolve(generated::RECIPES,generated::ARMOR_UPGRADES).expect("reported upgrade must resolve");
  assert_eq!(r.kind,kind as u8);assert_eq!(r.level,5);assert_eq!(row.pane(),11);
  list.insert(kind as usize-1,r).unwrap();
 }
 assert_eq!(List::decode(&list.encode()).unwrap().encode(),list.encode());
 let mut unrelated=native_row(list.slots[0].unwrap().recipe);unrelated.list_kind=7;
 assert!(unrelated.resolve(&[],generated::ARMOR_UPGRADES).is_none());
}
fn recipe()->Recipe{Recipe{kind:2,id:4,materials:[Material{id:318,count:2},Material{id:1,count:1},Material::default(),Material::default()],..Recipe::default()}}
#[test]fn notification_transitions_and_rearming(){
 let r=recipe();let mut l=List::default();l.insert(0,r).unwrap();let mut stock=[0;4096];
 assert!(l.poll(&stock,&[]).is_empty());stock[318]=2;stock[1]=1;
 assert_eq!(l.poll(&stock,&[]),vec![0]);assert!(l.poll(&stock,&[]).is_empty());
 stock[318]=1;assert!(l.poll(&stock,&[]).is_empty());stock[318]=2;assert_eq!(l.poll(&stock,&[]),vec![0]);
 // A loaded/just-registered ready item is a baseline, not a new pickup event.
 l.slots[0].as_mut().unwrap().ready=None;assert!(l.poll(&stock,&[]).is_empty());
}
#[test]fn duplicate_materials_and_category_consumption(){
 let mut r=recipe();r.materials[1]=Material{id:318,count:2};r.group=4;r.points=4;
 let p=&[(318,2,[4,0,0])];let mut s=[0;4096];s[318]=4;assert!(!r.ready(&s,p));s[318]=6;assert!(r.ready(&s,p));
 s[318]=3;r.group=0;assert!(!r.ready(&s,p));
}
#[test]fn duplicate_detection_slots_and_totals(){
 let mut l=List::default();let r=recipe();l.insert(0,r).unwrap();assert!(l.insert(1,r).is_err());
 let mut second=r;second.id=5;l.insert(47,second).unwrap();assert_eq!(l.totals().iter().find(|m|m.id==318).unwrap().count,4);
 assert!(l.insert(48,r).is_err());l.slots[0]=None;assert!(l.insert(1,r).is_ok());
}
#[test]fn persistence_and_corruption(){
 let mut l=List::default();l.language=8;l.insert(3,recipe()).unwrap();let text=l.encode();let next=List::decode(&text).unwrap();assert_eq!(next.language,8);assert_eq!(next.slots[3].unwrap().recipe,recipe());assert!(next.slots[3].unwrap().ready.is_none());
 for invalid in ["language=4","version=2","version=1\nlanguage=9","version=1\nslot48=2,4,0,0,0,318,2,1,1,0,0,0,0","version=1\nslot1=999,4,0,0,0,318,2,1,1,0,0,0,0","version=1\nslot1=2,4,0,0,0,4095,2,1,1,0,0,0,0"]{assert!(List::decode(invalid).is_err(),"accepted {invalid}");}
}
#[test]fn full_list_roundtrip_and_overflow(){
 let mut l=List::default();for i in 0..48{let mut r=recipe();r.id=i as u16+1;r.materials=[Material{id:318,count:999};4];l.insert(i,r).unwrap();}
 assert_eq!(l.totals()[0].count,48*4*999);let text=l.encode();assert!(text.len()<8192);assert_eq!(List::decode(&text).unwrap().encode(),text);
}
#[test]fn generated_data_and_nine_languages(){
 let mut kinds=std::collections::BTreeSet::new();for r in generated::RECIPES{assert!(r.valid(),"invalid recipe {r:?}");kinds.insert(r.kind);}
 for kind in [1,2,3,4,5,7,8,9,10,11,13,14,15,16,17,18,19,20,21,22,23,24,25]{assert!(kinds.contains(&kind));}
 assert_eq!(locales::CODES,["en","zh-Hans","zh-Hant","ja","ko","de","fr","it","es"]);
 for label in [locales::TITLE,locales::REGISTER,locales::NOTICE,locales::DELETE_QUESTION,locales::REPLACE_QUESTION,locales::SAVE_FAILED]{assert_eq!(label.len(),9);assert!(label.iter().all(|s|!s.is_empty()));}
 for table in [generated::ITEMDATA,generated::ARMORSERIESDATA,generated::OTWEAPONDATA,generated::OTARMORDATA]{assert!(table.iter().all(|row|row.len()==9));}
 // Verify known GU IDs independent of the XX database numbering.
 assert_eq!(generated::ITEMDATA[318*2][0],"Iron Ore");assert_eq!(generated::ITEMDATA[318*2][4],"철광석");
 assert_eq!(generated::ITEMDATA[318*2][3],"鉄鉱石");
 let saber=generated::WEAPON07MSGDATA.iter().find(|row|row[0]=="Petrified Saber").unwrap();
 assert_eq!(saber[3],"ベルダーサーベル");
 let breaker=generated::WEAPON02MSGDATA.iter().find(|row|row[0]=="Hidden Breaker").unwrap();
 assert_eq!(breaker[3],"ヒドゥンブレイカー");
}

#[test]fn button_layout_roundtrip_and_invalid_layout(){for layout in 0..3{let mut l=List::default();l.layout=layout;assert_eq!(List::decode(&l.encode()).unwrap().layout,layout);}assert!(List::decode("version=1\nlayout=3").is_err());assert_eq!(List::decode("version=1").unwrap().layout,0);}
#[test]fn weapon_details_have_sharpness_affinity_and_elements(){
 let melee:Vec<_>=equipment::DETAILS.iter().filter(|d|d.kind>=7&&d.kind<=18).collect();assert!(melee.iter().any(|d|d.affinity>0));assert!(melee.iter().any(|d|d.affinity<0));assert!(melee.iter().any(|d|d.elements.iter().any(|e|e.0==3&&e.1>0)));assert!(melee.iter().any(|d|d.sharp[0]>0&&d.sharp[2]>0));
 assert!(equipment::DETAILS.iter().all(|d|d.slots>=0&&d.slots<=3));
}

#[test]fn character_migration_is_once_and_settings_are_shared(){
 let mut legacy=List::default();legacy.insert(2,recipe()).unwrap();legacy.costs[2]=4500;legacy.language=3;legacy.layout=2;
 let first=profiles::load(1,false,List::default(),&legacy,None);assert_eq!(first.claim,Some(1));assert!(first.write);assert_eq!(first.list.slots[2].unwrap().recipe,recipe());assert_eq!(first.list.costs[2],4500);
 let other=profiles::load(0,false,List::default(),&legacy,Some(1));assert!(other.list.slots.iter().all(Option::is_none));assert_eq!(other.claim,None);assert_eq!(other.list.language,3);assert_eq!(other.list.layout,2);
 let cleared=profiles::load(1,true,List::default(),&legacy,Some(1));assert!(cleared.list.slots.iter().all(Option::is_none));assert!(!cleared.write);
 let mut personal=List::default();personal.insert(4,recipe()).unwrap();let restored=profiles::load(2,true,personal,&legacy,Some(1));assert!(restored.list.slots[4].is_some());assert!(restored.list.slots[2].is_none());
}

#[test]fn preferences_restore_across_restart_and_character_switch_without_changing_entries(){
 let defaults=List::default();assert_eq!((defaults.language,defaults.ui_scale),(0,100));
 let old=List::decode("version=1\nlanguage=4\nlayout=2\n").unwrap();assert_eq!((old.language,old.layout,old.ui_scale),(4,2,100));
 for scale in [50,60,70,80,90,100]{
  let mut legacy=List::default();legacy.insert(2,recipe()).unwrap();legacy.costs[2]=4500;
  let mut settings=List::default();settings.language=7;settings.layout=1;settings.ui_scale=scale;
  legacy.copy_preferences(&settings);let disk=List::decode(&legacy.encode()).unwrap();
  assert_eq!((disk.language,disk.layout,disk.ui_scale),(7,1,scale));assert_eq!(disk.slots[2].unwrap().recipe,recipe());assert_eq!(disk.costs[2],4500);
  let mut personal=List::default();personal.insert(8,Recipe{id:42,..recipe()}).unwrap();personal.costs[8]=300;
  let loaded=profiles::load(2,true,personal,&disk,Some(0)).list;
  assert_eq!((loaded.language,loaded.layout,loaded.ui_scale),(7,1,scale));assert_eq!(loaded.slots[8].unwrap().recipe.id,42);assert_eq!(loaded.costs[8],300);assert!(loaded.slots[2].is_none());
 }
 for invalid in ["49","101","0","200","nan"]{assert!(List::decode(&format!("version=1\nui_scale={invalid}\n")).is_err());}
}

#[test]fn uniform_viewport_fits_resolutions_aspect_ratios_and_scale_options(){
 for (width,height) in [(640.,360.),(1280.,720.),(1920.,1080.),(3840.,2160.),(1024.,768.),(1920.,1200.),(2560.,1080.),(3440.,1440.),(720.,1280.)]{
  for percent in [50,60,70,80,90,100]{let v=presentation::viewport(width,height,percent).unwrap();
   assert!(v.scale>0.&&v.ox>=-0.01&&v.oy>=-0.01);assert!(v.ox+1280.*v.scale<=width+0.01&&v.oy+720.*v.scale<=height+0.01);
   assert!((1280.*v.scale/(720.*v.scale)-16./9.).abs()<0.001);
   assert!((2.*v.ox+1280.*v.scale-width).abs()<0.01);assert!((2.*v.oy+720.*v.scale-height).abs()<0.01);
  }
 }
 for (w,h) in [(0.,720.),(-1.,720.),(1280.,0.),(f32::NAN,720.),(1280.,f32::INFINITY)]{assert!(presentation::viewport(w,h,100).is_none());}
}

#[path="../src/text_blob.rs"]mod text_blob;
#[path="../src/language_pack.rs"]mod language_pack;
#[path="language_test_source.rs"]mod language_test_source;
#[path="../src/name_data.rs"]mod name_data;
mod material_text_source{include!("../src/item_text_generated.rs");}
#[test]fn external_names_preserve_all_source_rows_and_nine_languages(){
 let data=include_bytes!("../assets/ui-names.bin");assert!(text_blob::valid(data));
 let tables=[generated::ITEMDATA,generated::ARMORSERIESDATA,generated::OTWEAPONDATA,generated::OTARMORDATA,
 generated::WEAPON00MSGDATA,generated::WEAPON01MSGDATA,generated::WEAPON02MSGDATA,generated::WEAPON03MSGDATA,
 generated::WEAPON04MSGDATA,generated::WEAPON06MSGDATA,generated::WEAPON07MSGDATA,generated::WEAPON08MSGDATA,
 generated::WEAPON09MSGDATA,generated::WEAPON10MSGDATA,generated::WEAPON11MSGDATA,generated::WEAPON12MSGDATA,
 generated::WEAPON13MSGDATA,generated::WEAPON14MSGDATA];
 for (table,rows) in tables.iter().enumerate(){for (index,row) in rows.iter().enumerate(){for lang in 0..9{
  assert_eq!(name_data::get(table as u8+1,index,lang),row[lang],"table {} row {} lang {}",table,index,lang);
 }}}
 for (index,row) in material_text_source::ITEM_TEXT.iter().enumerate(){for lang in 0..9{
  assert_eq!(name_data::material_description(index as u16,lang),row.1[lang]);
 }}
 assert_eq!(name_data::get(255,0,0),"");assert_eq!(name_data::get(1,usize::MAX,0),"");assert_eq!(name_data::get(1,0,9),"");
}
#[test]fn native_sharpness_matches_reported_weapons(){
 let breaker=equipment::get(Recipe{kind:9,id:51,level:4,..Recipe::default()}).unwrap();
 assert_eq!(breaker.sharp,[100,70,30,120,70,10,0]);
 let plume=equipment::get(Recipe{kind:14,id:19,level:1,..Recipe::default()}).unwrap();assert_eq!(plume.sharp,[70,30,40,10,0,0,0]);
 for d in equipment::DETAILS{assert!(d.sharp.iter().map(|v|*v as u32).sum::<u32>()<=450);if [11,13,17].contains(&d.kind){assert_eq!(d.sharp,[0;7]);}}
}
#[test]fn native_category_names_icons_and_descriptions_all_languages(){
 assert!(equipment::CATEGORIES.iter().all(|c|c.0<107&&c.1<17&&c.2.iter().all(|n|!n.is_empty())));
 let category=&equipment::CATEGORIES[182];assert_eq!((category.0,category.1),(13,10));assert_eq!(category.2[0],"Nargacuga");assert_eq!(category.2[3],"ナルガクルガ素材");assert!(category.2[4].contains("소재"));
 let data=include_bytes!("../assets/ui-text.bin");assert!(text_blob::valid(data));
 for kind in [1,2,3,4,5,7,9,14,22,23,24]{for lang in 0..9{assert!(!text_blob::get(data,kind,1,lang).is_empty());}}
 assert!(text_blob::get(data,2,1,4).contains("가죽"));assert!(text_blob::get(data,2,1,3).contains("胴"));
 assert_eq!(text_blob::get(data,255,65535,0),"");assert_eq!(text_blob::get(data,2,1,9),"");
 let mut bad=data.to_vec();bad[16..20].copy_from_slice(&u32::MAX.to_le_bytes());assert!(!text_blob::valid(&bad));
 assert!(!text_blob::valid(&data[..20]));bad=data.to_vec();*bad.last_mut().unwrap()=255;assert!(!text_blob::valid(&bad));
}

#[path="../src/presentation.rs"]mod presentation;
#[test]fn aggregate_material_dependencies_follow_registered_entries(){
 let mut list=List::default();let first=Recipe{materials:[Material{id:318,count:2},Material{id:318,count:1},Material{id:319,count:0},Material::default()],..recipe()};
 list.insert(0,first).unwrap();list.insert(9,Recipe{id:5,materials:[Material{id:318,count:4},Material{id:540,count:2},Material::default(),Material::default()],..first}).unwrap();
 assert_eq!(list.totals().iter().find(|m|m.id==318).unwrap().count,7);
 assert!(list.totals().iter().all(|m|m.count>0));assert_eq!(list.requiring_material(318),vec![0,9]);assert_eq!(list.requiring_material(540),vec![9]);assert!(list.requiring_material(319).is_empty());assert!(list.requiring_material(999).is_empty());
 list.slots[0]=None;assert_eq!(list.requiring_material(318),vec![9]);list.slots[9]=None;assert!(list.requiring_material(318).is_empty());
}
#[test]fn aggregate_cursor_pages_partial_pages(){
 let mut f=interaction::AggregateFocus::default();f.material_step(10,-1,0);assert_eq!(f.material,7);f.material_step(10,0,1);assert_eq!(f.material,9);f.material_step(10,1,0);assert_eq!(f.material,8);
 f.material_step(10,1,0);assert_eq!(f.material,9);f.material_step(0,1,1);assert_eq!(f.material,0);
 for count in 1..=192{for index in 0..count+8{for d in -1..=1{for page in -1..=1{assert!(navigation::move_item(index,count,d,page)<count);}}}}
}
#[test]fn notice_appears_immediately_holds_and_fades_without_motion(){
 assert_eq!(presentation::notice_opacity(0.),1.);assert_eq!(presentation::notice_opacity(presentation::NOTICE_HOLD),1.);
 assert!((presentation::notice_opacity(presentation::NOTICE_HOLD+presentation::NOTICE_FADE*0.5)-0.5).abs()<0.001);
 assert_eq!(presentation::notice_opacity(presentation::NOTICE_DURATION),0.);
 let mut last=1.;for step in 0..=100{let opacity=presentation::notice_opacity(presentation::NOTICE_DURATION*step as f32/100.);assert!(opacity<=last);assert!((0.0..=1.0).contains(&opacity));last=opacity;}
 assert!(presentation::NOTICE_HOLD>=3.);
}
#[path="../src/game_menu.rs"]mod game_menu;
#[test]fn native_main_menu_detection_rejects_submenus_in_both_locations(){
 for vtable in [game_menu::QUEST_VTABLE,game_menu::LOBBY_VTABLE]{
  assert!(game_menu::is_main_menu(vtable,2,0,true));assert!(game_menu::is_main_menu(vtable,3,0,true));
  for phase in [0,1,4]{assert!(!game_menu::is_main_menu(vtable,phase,0,true));}
  for substate in [1,2,3,4,u32::MAX]{for phase in [2,3]{assert!(!game_menu::is_main_menu(vtable,phase,substate,true));}}
  assert!(!game_menu::is_main_menu(vtable,3,0,false));
 }
 assert!(!game_menu::is_main_menu(0,3,0,true));assert!(!game_menu::is_main_menu(0x177afc4,3,0,true));
}
#[path="../src/sound_cues.rs"]mod sound_cues;
#[test]fn native_effects_are_selected_by_action(){
 use sound_cues::Cue::*;
 assert_eq!([Open,Close,Cursor,Page,Tab,Inspect,Registered].map(|c|c.id()),[1,2,4,7,27,6,31]);
}

#[path="../src/notifications.rs"]mod notifications;
fn notification_recipes()->[Recipe;4]{core::array::from_fn(|i|Recipe{id:i as u16+1,..recipe()})}
fn tick_queue(q:&mut notifications::Queue,count:usize){for _ in 0..count{q.advance(0.1);}}
#[test]fn notifications_show_two_keep_waiting_time_and_promote_fifo(){
 let recipes=notification_recipes();let mut q=notifications::Queue::default();for r in recipes{q.push(r);}q.push(recipes[0]);
 assert_eq!((q.visible_count(),q.waiting_count()),(2,2));assert_eq!(q.visible().map(|(_,n)|n.recipe).collect::<Vec<_>>(),recipes[..2]);
 tick_queue(&mut q,50);assert_eq!(q.waiting_count(),2);assert!(q.visible().all(|(_,n)|n.seconds>0.49));
 tick_queue(&mut q,5);assert_eq!((q.visible_count(),q.waiting_count()),(2,0));
 assert_eq!(q.visible().map(|(_,n)|n.recipe).collect::<Vec<_>>(),recipes[2..]);assert!(q.visible().all(|(_,n)|(n.seconds-presentation::NOTICE_DURATION).abs()<0.001));
 tick_queue(&mut q,55);assert!(q.is_empty());
 assert!(notifications::top(0)+notifications::HEIGHT+notifications::GAP<=notifications::top(1));assert!(notifications::top(1)+notifications::HEIGHT<570.);
}
#[test]fn notification_lifetimes_are_independent_when_second_arrives_later(){
 let r=notification_recipes();let mut q=notifications::Queue::default();q.push(r[0]);tick_queue(&mut q,20);q.push(r[1]);q.push(r[2]);
 tick_queue(&mut q,35);let v:Vec<_>=q.visible().map(|(_,n)|(n.recipe,n.seconds)).collect();
 assert_eq!(v[0].0,r[1]);assert!((v[0].1-2.).abs()<0.001);assert_eq!(v[1].0,r[2]);assert!((v[1].1-presentation::NOTICE_DURATION).abs()<0.001);
 tick_queue(&mut q,20);assert_eq!(q.visible().next().unwrap().1.recipe,r[2]);assert_eq!(q.visible_count(),1);
 q.advance(f32::NAN);q.advance(-5.);assert!(q.visible().all(|(_,n)|n.seconds.is_finite()));
}
#[test]fn notification_demo_scenarios_use_the_same_queue_and_do_not_change_registrations(){
 let recipes=notification_recipes();
 let mut simultaneous=notifications::Demo::new(recipes,0);assert_eq!((simultaneous.queue.visible_count(),simultaneous.queue.waiting_count()),(2,0));
 let mut staggered=notifications::Demo::new(recipes,1);assert_eq!(staggered.queue.visible_count(),1);for _ in 0..20{staggered.advance(0.1);}assert_eq!(staggered.queue.visible_count(),2);
 let mut queued=notifications::Demo::new(recipes,2);assert_eq!((queued.queue.visible_count(),queued.queue.waiting_count()),(2,2));for _ in 0..55{queued.advance(0.1);}assert_eq!(queued.queue.waiting_count(),0);assert_eq!(queued.queue.visible_count(),2);
 simultaneous.advance(0.1);simultaneous.restart();assert!(simultaneous.queue.visible().all(|(_,n)|n.seconds==presentation::NOTICE_DURATION));
 assert_eq!(simultaneous.recipes,recipes);queued.queue.clear();assert!(queued.queue.is_empty());
}

#[path="../src/badge_model.rs"]mod badge_model;
#[test]fn shortages_sort_before_sufficient_materials_and_keep_stable_ids(){
 let mut list=List::default();list.insert(0,Recipe{kind:2,id:1,materials:[Material{id:10,count:2},Material{id:20,count:5},Material{id:30,count:1},Material{id:40,count:3}],..Recipe::default()}).unwrap();
 let mut stock=[0;4096];stock[10]=2;stock[30]=5;
 assert_eq!(list.ordered_totals(&stock,true).iter().map(|m|m.id).collect::<Vec<_>>(),vec![20,40,10,30]);
 assert_eq!(list.ordered_totals(&stock,false).iter().map(|m|m.id).collect::<Vec<_>>(),vec![10,20,30,40]);
 stock[20]=5;assert_eq!(list.ordered_totals(&stock,true)[0].id,40);
}
#[test]fn production_completion_removes_only_same_equipment_and_clears_cost(){
 let mut list=List::default();let r=Recipe{kind:2,id:7,materials:[Material{id:1,count:1},Material::default(),Material::default(),Material::default()],..Recipe::default()};
 list.insert(0,r).unwrap();list.costs[0]=500;list.insert(1,Recipe{kind:1,..r}).unwrap();list.insert(2,Recipe{level:2,..r}).unwrap();list.insert(3,Recipe{materials:[Material{id:2,count:1},Material::default(),Material::default(),Material::default()],..r}).unwrap();
 assert!(list.remove_completed(2,7,0));assert!(list.slots[0].is_none());assert_eq!(list.costs[0],0);assert!(list.slots[3].is_none());assert!(list.slots[1].is_some());assert!(list.slots[2].is_some());assert!(!list.remove_completed(2,7,0));
 let restored=List::decode(&list.encode()).unwrap();assert!(restored.slots[0].is_none());assert!(restored.slots[2].is_some());
}
#[test]fn native_grid_badges_use_equipment_identity_next_upgrade_and_live_removal(){
 let mut list=List::default();let r=Recipe{kind:9,id:51,level:5,materials:[Material{id:1,count:1},Material::default(),Material::default(),Material::default()],..Recipe::default()};list.insert(0,r).unwrap();
 let gear=badge_model::Gear::from_native(9|(3<<5),51);assert_eq!(gear.registered(&list),Some(0));
 assert_eq!(badge_model::Gear::from_native(9|(2<<5),51).registered(&list),None);assert_eq!(badge_model::Gear::from_native(14|(3<<5),51).registered(&list),None);assert_eq!(badge_model::Gear::from_native(9|(3<<5),52).registered(&list),None);
 list.remove_completed(9,51,5);assert_eq!(gear.registered(&list),None);list.insert(0,Recipe{level:0,..r}).unwrap();assert_eq!(gear.registered(&list),Some(0));
}
#[test]fn removed_equipment_does_not_leave_queued_notifications(){
 let mut list=List::default();let r=*generated::RECIPES.iter().find(|r|r.level==0).unwrap();list.insert(0,r).unwrap();let mut queue=notifications::Queue::default();queue.push(r);queue.retain_registered(&list);assert_eq!(queue.visible_count(),1);list.remove_completed(r.kind,r.id,0);queue.retain_registered(&list);assert!(queue.is_empty());
}
#[test]fn equipment_icons_match_native_rarity_instead_of_a_shared_blue_tint(){
 let braces=Recipe{kind:3,id:3,..Recipe::default()};
 assert_eq!(equipment::get(braces).unwrap().rarity,1);assert_eq!(equipment::icon_tint(braces),0xffe6e6e6);
 let breaker=Recipe{kind:9,id:51,level:4,..Recipe::default()};
 assert_eq!(equipment::get(breaker).unwrap().rarity,2);assert_eq!(equipment::icon_tint(breaker),0xfff292b0);
 for kind in [1,2,3,4,5,7,8,9,10,11,13,14,15,16,17,18,19,20,21,22,23,24]{
  let r=Recipe{kind,id:1,..Recipe::default()};if let Some(d)=equipment::get(r){assert_eq!(equipment::icon_tint(r),presentation::rarity(d.rarity));}
 }
}
#[path="../src/hud_layout.rs"]mod hud_layout;
#[test]fn native_hud_uses_runtime_picture_and_reference_width(){
 use hud_layout::{HudSize as H,right_top as map};
 let hud=|w,h,u|Some(H{render_width:w,render_height:h,ui_width:u});
 let p=map(2560.,1600.,hud(2560,1600,1280)).unwrap();assert_eq!((p.scale,p.ox,p.oy),(2.,0.,0.));
 let p=map(2560.,1600.,hud(1920,1080,1280)).unwrap();assert_eq!((p.scale,p.ox,p.oy),(2.,0.,80.));
 let a=map(3440.,1440.,hud(3440,1440,1280)).unwrap();let b=map(3440.,1440.,hud(3440,1440,1720)).unwrap();assert_eq!(a.ox,440.);assert_eq!(b.ox,880.);
 let p=map(1024.,768.,hud(1280,800,1280)).unwrap();assert!((p.scale-0.8).abs()<0.001);assert_eq!(p.oy,64.);
 assert!(map(0.,720.,None).is_none());
 for values in [(0,720,1280),(1280,0,1280),(1280,720,0),(u32::MAX,720,1280)]{let p=map(1920.,1080.,hud(values.0,values.1,values.2)).unwrap();assert_eq!((p.scale,p.ox,p.oy),(1.5,0.,0.));}
}

#[test]fn custom_bindings_round_trip_and_invalid_values_fall_back(){
 let mut list=List::default();list.pad_bindings[0]=(1<<6)|(1<<20);list.key_bindings[0]=0x50000|14;
 let mut copy=List::decode(&list.encode()).unwrap();assert_eq!(copy.pad_bindings,list.pad_bindings);assert_eq!(copy.key_bindings,list.key_bindings);
 copy.copy_preferences(&List::default());assert_eq!(copy.pad_bindings,binding_model::DEFAULT_PAD);
 let bad=List::decode("version=1\npad_binding0=4294967295\nkey_binding0=73\nkey_binding1=9999999\n").unwrap();assert_eq!(bad.pad_bindings,binding_model::DEFAULT_PAD);assert_eq!(bad.key_bindings,binding_model::DEFAULT_KEYS);
}
#[test]fn binding_chords_are_specific_and_only_fire_on_new_press(){
 use binding_model::*;let mut e=Edges::default();let both=active(48,&DEFAULT_PAD);assert_eq!(both,1);assert_eq!(events(e.step(both)),2);assert_eq!(e.step(both),0);e.step(0);assert_eq!(events(e.step(both)),2);
 e.block();assert_eq!(e.step(both),0);e.step(0);assert_eq!(e.step(both),1);
}
#[test]fn keyboard_translation_requires_neutral_before_controller_rearms(){
 let mut g=binding_model::KeyboardGuard::default();assert_eq!(g.filter(32,true),0);assert_eq!(g.filter(32,false),0);assert_eq!(g.filter(0,false),0);assert_eq!(g.filter(32,false),32);
}
#[test]fn captures_can_be_replaced_and_need_a_complete_keyboard_key(){
 use binding_model::*;let mut c=Capture{target:1,..Capture::EMPTY};c.sample(0);c.sample(16);c.sample(48);c.sample(16);assert_eq!(c.candidate,48);c.sample(0);c.sample(64);assert_eq!(c.candidate,64);
 let mut k=Capture{target:COUNT as u8+1,..Capture::EMPTY};k.sample(0);k.sample(1<<16);assert!(!k.has_binding());k.sample((1<<16)|14);assert!(k.has_binding());k.sample(1<<16);k.sample((1<<16)|15);assert_eq!(k.candidate,(1<<16)|15);
}

#[test]fn resetting_preferences_preserves_registrations_and_saved_snapshot(){
 let mut live=List::default();live.insert(0,generated::RECIPES[0]).unwrap();live.costs[0]=123;
 live.language=4;live.layout=2;live.ui_scale=60;live.pad_bindings[0]=64;live.key_bindings[0]=14;
 let saved=live.encode();let entry=live.slots[0].unwrap().recipe;live.reset_preferences();
 assert_eq!(live.language,0);assert_eq!(live.layout,0);assert_eq!(live.ui_scale,100);assert_eq!(live.pad_bindings,binding_model::DEFAULT_PAD);assert_eq!(live.key_bindings,binding_model::DEFAULT_KEYS);
 assert_eq!(live.slots[0].unwrap().recipe,entry);assert_eq!(live.costs[0],123);
 let disk=List::decode(&saved).unwrap();assert_eq!(disk.language,4);assert_eq!(disk.pad_bindings[0],64);
 let mut other=List::default();other.copy_preferences(&live);assert_eq!(other.key_bindings,live.key_bindings);
}

#[test]fn character_autosave_contains_entries_but_no_unsaved_preferences(){
 let mut list=List::default();list.language=4;list.pad_bindings[0]=64;list.insert(0,generated::RECIPES[0]).unwrap();list.costs[0]=123;
 let text=list.encode_character();assert!(!text.contains("language="));assert!(!text.contains("binding"));assert!(!text.contains("layout="));
 let disk=List::decode(&text).unwrap();assert_eq!(disk.slots[0].unwrap().recipe,list.slots[0].unwrap().recipe);assert_eq!(disk.costs[0],123);
}

#[test]fn queued_mapped_pad_press_is_blocked_through_keyboard_and_pad_release(){
 let mut guard=binding_model::KeyboardGuard::new();
 for (raw,keyboard,blocked) in [(32,true,true),(32,false,true),(0,false,true),(0,false,false),(32,false,false)]{
  assert_eq!(keyboard||guard.waiting(),blocked);guard.filter(raw,keyboard);
 }
}

fn sort_recipe(id:u16,kind:u8,material:u16)->Recipe{Recipe{kind,id,materials:[Material{id:material,count:2},Material::default(),Material::default(),Material::default()],..Recipe::default()}}
fn display_ids(list:&List)->Vec<u16>{list.slots.iter().flatten().map(|e|e.recipe.id).collect()}
#[test]fn sort_keeps_material_priority_and_uses_category_only_inside_the_preferred_status(){
 let mut l=List::default();for (i,r) in [sort_recipe(1,9,1),sort_recipe(2,2,2),sort_recipe(3,9,2),sort_recipe(4,2,1),sort_recipe(5,9,2)].into_iter().enumerate(){l.register(i,r,(i+1) as u32*100).unwrap();}
 let mut stock=[0;4096];stock[2]=2;
 l.sort_display(1,1,&stock,true,&[]).unwrap();assert_eq!(display_ids(&l),vec![3,5,2,1,4]);assert_eq!(l.costs[..5],[300,500,200,100,400]);
 let saved=l.encode_character();let mut reload=List::decode(&saved).unwrap();assert_eq!(display_ids(&reload),vec![3,5,2,1,4]);
 reload.sort_display(2,1,&stock,true,&[]).unwrap();assert_eq!(display_ids(&reload),vec![1,4,2,3,5]);
 reload.sort_display(0,5,&stock,false,&[]).unwrap();assert_eq!(display_ids(&reload),vec![1,2,3,4,5]);assert_eq!(reload.costs[..5],[100,200,300,400,500]);
}
#[test]fn sorted_display_stays_frozen_as_stock_changes_and_shared_materials_are_not_reserved(){
 let mut l=List::default();for (i,r) in [sort_recipe(1,2,1),sort_recipe(2,9,1),sort_recipe(3,2,2)].into_iter().enumerate(){l.register(i,r,0).unwrap();}
 let mut stock=[0;4096];stock[1]=3;l.sort_display(1,1,&stock,true,&[]).unwrap();assert_eq!(display_ids(&l),vec![2,1,3]);
 stock[1]=0;stock[2]=3;l.poll(&stock,&[]);assert_eq!(display_ids(&l),vec![2,1,3]);
 let restored=List::decode(&l.encode_character()).unwrap();assert_eq!(display_ids(&restored),vec![2,1,3]);
}
#[test]fn removal_keeps_gaps_and_registration_and_overwrite_use_the_selected_slot(){
 let mut l=List::default();for i in 0..3{l.register(i,sort_recipe(i as u16+1,9,1),100+i as u32).unwrap();}
 l.remove_slot(1);assert_eq!(display_ids(&l),vec![1,3]);assert!(l.slots[1].is_none());assert_eq!(l.slots[2].unwrap().recipe.id,3);assert_eq!(l.registration[..3],[1,0,2]);
 assert_eq!(l.register(1,sort_recipe(4,9,1),400).unwrap(),1);assert_eq!(display_ids(&l),vec![1,4,3]);
 l.sort_display(0,0,&[0;4096],false,&[]).unwrap();assert_eq!(display_ids(&l),vec![1,3,4]);
 assert_eq!(l.register(0,sort_recipe(5,2,1),500).unwrap(),0);assert_eq!(display_ids(&l),vec![5,3,4]);assert_eq!(l.registration[..3],[3,1,2]);assert_eq!(l.costs[..3],[500,102,400]);assert!(l.slots[3..].iter().all(Option::is_none));
 assert!(l.register(1,sort_recipe(5,2,1),0).is_err());assert_eq!(display_ids(&l),vec![5,3,4]);
}
#[test]fn every_secondary_category_promotes_matching_material_status_only(){
 for (category,kind) in [(1,21),(2,5),(3,25),(4,22),(5,24)]{
  let mut l=List::default();l.register(0,sort_recipe(1,if category==1{2}else{9},1),0).unwrap();l.register(1,sort_recipe(2,kind,1),0).unwrap();
  l.sort_display(2,category,&[0;4096],true,&[]).unwrap();assert_eq!(display_ids(&l),vec![2,1]);
 }
}
#[test]fn sort_unknown_stock_and_invalid_options_do_not_change_list_and_category_points_count(){
 let mut l=List::default();let mut r=sort_recipe(1,9,1);r.group=8;r.points=6;l.register(0,r,0).unwrap();l.register(1,sort_recipe(2,9,2),0).unwrap();let before=l.encode_character();
 assert!(l.sort_display(1,0,&[0;4096],false,&[]).is_err());assert_eq!(l.encode_character(),before);
 assert!(l.sort_display(3,0,&[0;4096],true,&[]).is_err());assert_eq!(l.encode_character(),before);
 let mut stock=[0;4096];stock[1]=2;stock[2]=2;l.sort_display(1,0,&stock,true,&[(3,2,[8,0,0])]).unwrap();assert_eq!(display_ids(&l),vec![2,1]);
 stock[3]=3;l.sort_display(1,0,&stock,true,&[(3,2,[8,0,0])]).unwrap();assert_eq!(display_ids(&l),vec![1,2]);
}
#[test]fn legacy_registration_order_migrates_and_invalid_rank_data_is_rejected(){
 let mut l=List::default();l.insert(8,sort_recipe(1,9,1)).unwrap();l.insert(2,sort_recipe(2,9,1)).unwrap();
 let legacy=l.encode_character().lines().filter(|line|!line.starts_with("registration")).collect::<Vec<_>>().join("\n");
 let mut loaded=List::decode(&legacy).unwrap();loaded.compact();assert_eq!(display_ids(&loaded),vec![2,1]);loaded.sort_display(0,0,&[0;4096],false,&[]).unwrap();assert_eq!(display_ids(&loaded),vec![2,1]);
 let mut text=l.encode_character();text=text.replace("registration8=1","registration8=2");assert!(List::decode(&text).is_err());
}
#[test]fn sort_dialog_is_two_stage_transaction_and_reopens_at_defaults(){
 use sorting::{Dialog,Action};let mut d=Dialog::default();assert_eq!(d.input(0,true,false),Action::Apply);assert!(!d.secondary);
 d.input(1,false,false);assert_eq!(d.input(0,true,false),Action::None);assert!(d.secondary);d.input(-1,false,false);assert_eq!(d.category,5);
 assert_eq!(d.input(0,false,true),Action::None);assert!(!d.secondary);assert_eq!(d.category,0);assert_eq!(d.input(0,false,true),Action::Cancel);
 let fresh=Dialog::default();assert_eq!((fresh.criterion,fresh.category,fresh.secondary),(0,0,false));
}

#[test]fn deleting_from_a_sorted_list_preserves_both_orders_and_reuses_bounded_ranks(){
 let mut l=List::default();for i in 0..48{l.register(i,sort_recipe(i as u16+1,if i%2==0{2}else{9},1),i as u32).unwrap();}
 l.sort_display(2,1,&[0;4096],true,&[]).unwrap();assert_eq!(l.slots[0].unwrap().recipe.id,2);
 l.remove_slot(1);assert!(l.slots[1].is_none());assert_eq!(l.slots[2].unwrap().recipe.id,6);assert_eq!(l.slots[47].unwrap().recipe.id,47);
 for id in 100..200{l.register(0,sort_recipe(id,9,1),id as u32).unwrap();assert_eq!(l.registration.iter().copied().max(),Some(47));}
 let disk=List::decode(&l.encode_character()).unwrap();assert_eq!(display_ids(&disk),display_ids(&l));assert_eq!(disk.registration,l.registration);
 l.sort_display(0,0,&[0;4096],false,&[]).unwrap();let ids=display_ids(&l);assert!(ids.windows(2).all(|pair|pair[0]<pair[1]));assert_eq!(ids.last(),Some(&199));
}

#[test]fn registration_keeps_selected_empty_or_occupied_slot_across_save_and_reload(){
 let mut l=List::default();l.register(0,sort_recipe(1,9,1),100).unwrap();
 assert_eq!(l.register(7,sort_recipe(2,25,1),200).unwrap(),7);assert!(l.slots[1..7].iter().all(Option::is_none));
 let mut loaded=List::decode(&l.encode_character()).unwrap();assert!(loaded.slots[1..7].iter().all(Option::is_none));assert_eq!(loaded.slots[7].unwrap().recipe.id,2);assert_eq!(loaded.costs[7],200);
 loaded.register(0,sort_recipe(3,2,1),300).unwrap();assert_eq!(loaded.slots[0].unwrap().recipe.id,3);assert_eq!(loaded.slots[7].unwrap().recipe.id,2);
 loaded.sort_display(0,0,&[0;4096],false,&[]).unwrap();assert_eq!(display_ids(&loaded),vec![2,3]);assert_eq!(loaded.costs[..2],[200,300]);assert!(loaded.slots[2..].iter().all(Option::is_none));
}
#[test]fn no_missing_decorations_still_promotes_all_other_missing_equipment(){
 let mut l=List::default();for (i,r) in [sort_recipe(1,2,1),sort_recipe(2,25,2),sort_recipe(3,9,2),sort_recipe(4,25,2),sort_recipe(5,22,1)].into_iter().enumerate(){l.register(i,r,0).unwrap();}
 let mut stock=[0;4096];stock[2]=2;
 l.sort_display(2,3,&stock,true,&[]).unwrap();assert_eq!(display_ids(&l),vec![1,5,2,3,4]);
 l.sort_display(1,3,&stock,true,&[]).unwrap();assert_eq!(display_ids(&l),vec![2,4,3,1,5]);
 l.sort_display(2,0,&stock,true,&[]).unwrap();assert_eq!(display_ids(&l),vec![1,5,2,3,4]);
 l.register(5,sort_recipe(6,25,1),0).unwrap();l.sort_display(2,3,&stock,true,&[]).unwrap();assert_eq!(display_ids(&l),vec![6,1,5,2,3,4]);
 // Once no entries have the preferred material state, category must not reorder them.
 stock[1]=2;l.sort_display(2,3,&stock,true,&[]).unwrap();assert_eq!(display_ids(&l),vec![1,2,3,4,5,6]);
 stock[1]=0;stock[2]=0;l.sort_display(1,3,&stock,true,&[]).unwrap();assert_eq!(display_ids(&l),vec![1,2,3,4,5,6]);
}

#[test]fn manual_and_completed_removals_preserve_slots_until_explicit_sort(){
 for completed in [false,true]{
  let mut list=List::default();for (slot,id) in [(0,1),(7,2),(8,3),(47,4)]{list.register(slot,sort_recipe(id,9,1),id as u32*100).unwrap();}
  if completed{assert!(list.remove_completed(9,2,0));}else{list.remove_slot(7);}
  assert!(list.slots[7].is_none());assert_eq!(list.costs[7],0);assert_eq!(list.registration[7],0);
  assert_eq!(list.slots[8].unwrap().recipe.id,3);assert_eq!(list.costs[8],300);assert_eq!(list.slots[47].unwrap().recipe.id,4);assert_eq!(list.costs[47],400);
  let mut loaded=List::decode(&list.encode_character()).unwrap();assert!(loaded.slots[7].is_none());assert_eq!(loaded.slots[8].unwrap().recipe.id,3);assert_eq!(loaded.slots[47].unwrap().recipe.id,4);
  loaded.sort_display(0,0,&[0;4096],false,&[]).unwrap();assert_eq!(display_ids(&loaded),vec![1,3,4]);assert_eq!(loaded.costs[..3],[100,300,400]);assert!(loaded.slots[3..].iter().all(Option::is_none));
 }
}

fn language_files(language:usize,kind:usize)->Vec<u8>{let code=locales::CODES[language];std::fs::read(format!("assets/languages/{}.{}.bin",if kind==0{"names"}else{"text"},code)).unwrap()}
#[test]fn compact_name_indexes_keep_only_nonempty_ids_in_all_nine_languages(){
 let original=&include_bytes!("../assets/ui-names.bin")[..];let old_rows=u32::from_le_bytes(original[8..12].try_into().unwrap()) as usize;
 for lang in 0..9{let data=language_files(lang,0);let count=u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;let mut expected=0;
  for i in 0..old_rows{let key=u32::from_le_bytes(original[12+i*40..16+i*40].try_into().unwrap());if key>>16!=31&&!text_blob::get(original,(key>>16) as u8,key as u16,lang).is_empty(){expected+=1;}}
  assert_eq!(count,expected);for row in data[16..16+count*8].chunks_exact(8){let at=u32::from_le_bytes(row[4..8].try_into().unwrap()) as usize;assert_ne!(data[at],0,"empty name rows should not be stored");}
 }
}
#[test]fn single_language_packs_preserve_every_name_and_description_in_all_nine_languages(){
 for lang in 0..9{let mut pack=language_test_source::prepare(lang).unwrap();
  for (description,source) in [(false,&include_bytes!("../assets/ui-names.bin")[..]),(true,&include_bytes!("../assets/ui-text.bin")[..])]{
   if !description{let single=language_files(lang,0);assert_eq!(text_blob::single_language(&single),Some(lang));assert!(text_blob::valid(&single));}
   let rows=u32::from_le_bytes(source[8..12].try_into().unwrap()) as usize;
   for i in 0..rows{let key=u32::from_le_bytes(source[12+i*40..16+i*40].try_into().unwrap());assert_eq!(pack.text(description,(key>>16) as u8,key as u16,lang),text_blob::get(source,(key>>16) as u8,key as u16,lang));}
   assert_eq!(pack.text(description,1,0,(lang+1)%9),"");
  }
  assert!(pack.bytes()<2*1024*1024); // one language only, not the original ~10 MiB corpus
 }
}
#[test]fn resource_staging_failure_keeps_previous_language_and_owned_text_valid(){
 let mut store=language_pack::Store::default();store.commit(language_test_source::prepare(0).unwrap());
 let owned=store.text(false,1,2,0);assert_eq!(owned,"Book of Combos 1");
 for failure in 0..4{
  let mut names=language_files(if failure==2{0}else{4},0);let mut text=language_files(4,1);
  match failure{0=>names.clear(),1=>text.clear(),3=>text.truncate(20),_=>()}
  let attempted=language_pack::Pack::prepare(4,names,Box::new(language_test_source::Memory(text)));
  assert!(attempted.is_err());assert_eq!(store.active.as_ref().unwrap().language,0);assert_eq!(store.text(false,1,2,0),owned);
 }
 for lang in (0..9).cycle().take(90){let next=language_test_source::prepare(lang).unwrap();drop(store.commit(next));assert_eq!(store.active.as_ref().unwrap().language,lang);}
 assert_eq!(owned,"Book of Combos 1");assert!(!language_pack::matches(99,0,b""));assert!(!language_pack::matches(0,9,b""));
 assert_eq!(language_pack::crc32(b"123456789"),0xcbf43926);
}
#[test]fn single_language_blob_rejects_wrong_language_offsets_and_truncation(){
 let good=language_files(0,0);for end in [0,4,12,15,20,good.len()-1]{assert!(text_blob::single_language(&good[..end]).is_none());}
 let mut bad=good.clone();bad[8..12].copy_from_slice(&9u32.to_le_bytes());assert!(text_blob::single_language(&bad).is_none());
 bad=good.clone();bad[20..24].copy_from_slice(&0u32.to_le_bytes());assert!(text_blob::single_language(&bad).is_none());
}

#[path="../src/atlas.rs"]mod atlas;
mod atlas_art{include!("../src/icons_generated.rs");}
#[test]fn hd_and_low_memory_atlases_preserve_all_sprite_rectangles_and_reject_other_sizes(){
 let hd_bytes=std::fs::metadata("assets/wishlist-atlas.rgba").unwrap().len();let low_bytes=std::fs::metadata("assets/low-memory/wishlist-atlas.rgba").unwrap().len();
 let hd=atlas::layout(hd_bytes).unwrap();let low=atlas::layout(low_bytes).unwrap();
 assert!(!hd.low_memory&&low.low_memory);assert_eq!((hd.width,hd.height),(2048,1024));assert_eq!((low.width,low.height),(1024,352));assert_eq!(low.uv_width,hd.uv_width);assert_eq!(low.uv_height,704);
 for size in [0,1,low_bytes-1,low_bytes+1,hd_bytes-1,hd_bytes+1]{assert!(atlas::layout(size).is_none());}
 for sprite in atlas_art::ART{
  let x0=sprite.x as f32/low.uv_width as f32*low.width as f32;let y0=sprite.y as f32/low.uv_height as f32*low.height as f32;
  let x1=(sprite.x+sprite.w) as f32/low.uv_width as f32*low.width as f32;let y1=(sprite.y+sprite.h) as f32/low.uv_height as f32*low.height as f32;
  assert!((x0-sprite.x as f32*0.5).abs()<0.001&&(y0-sprite.y as f32*0.5).abs()<0.001);
  assert!(x1<=low.width as f32&&y1<=low.height as f32);assert!((x1-x0-sprite.w as f32*0.5).abs()<0.001&&(y1-y0-sprite.h as f32*0.5).abs()<0.001);
 }
 assert_eq!(atlas_art::ART.len(),159);assert_eq!(hd_bytes,8388608);assert_eq!(low_bytes,1441792);
 assert!(low_bytes*100/hd_bytes<=18);
}
#[test]fn cropped_hd_atlas_preserves_all_sprite_pixels_and_legacy_uv_coordinates(){
 let original=std::fs::read("assets/wishlist-atlas.rgba").unwrap();let cropped=std::fs::read("assets/hd/wishlist-atlas.rgba").unwrap();let hd=atlas::layout(cropped.len() as u64).unwrap();
 assert!(!hd.low_memory);assert_eq!((hd.width,hd.height,hd.uv_width,hd.uv_height),(2048,704,2048,704));assert_eq!(cropped.len(),5767168);assert_eq!(cropped,&original[..cropped.len()]);
 assert!(original[cropped.len()+3..].iter().step_by(4).all(|alpha|*alpha==0));
 for sprite in atlas_art::ART{assert!((sprite.x+sprite.w)<=hd.width&&(sprite.y+sprite.h)<=hd.height);
  for row in sprite.y..sprite.y+sprite.h{let at=(row as usize*2048+sprite.x as usize)*4;let end=at+sprite.w as usize*4;assert_eq!(cropped[at..end],original[at..end]);}
  for y in [sprite.y,sprite.y+sprite.h]{let normalized=y as f32/hd.uv_height as f32;assert!((normalized*hd.height as f32-y as f32).abs()<0.0001);}
 }
 for bad in [cropped.len()-1,cropped.len()+1]{assert!(atlas::layout(bad as u64).is_none());}
}

#[path="../src/diagnostics.rs"]mod diagnostics;
#[path="../src/inventory.rs"]mod inventory;
#[test]fn diagnostic_preference_is_opt_in_persisted_global_and_resettable(){
 assert!(!List::default().diagnostics);assert!(!List::decode("version=1\nlanguage=0\n").unwrap().diagnostics);
 let mut settings=List::default();settings.diagnostics=true;
 assert!(List::decode(&settings.encode()).unwrap().diagnostics);
 assert!(!List::decode(&settings.encode_character()).unwrap().diagnostics);
 let mut character=List::default();character.copy_preferences(&settings);assert!(character.diagnostics);
 character.reset_preferences();assert!(!character.diagnostics);
 assert!(List::decode("version=1\ndiagnostics=oops\n").is_err());
 assert_eq!(crate::locales::ENABLE_LOGGING[4],"로그 기록 활성화");assert!(crate::locales::ENABLE_LOGGING.iter().all(|s|!s.is_empty()));
}
#[test]fn diagnostic_logs_are_gated_rate_limited_and_reenabled_cleanly(){
 use diagnostics::*;
 set_enabled(false);let before=emitted();event(1,"disabled",[0;4]);assert_eq!(emitted(),before);assert!(!enabled());
 assert!(set_enabled(true));assert!(!set_enabled(true));assert!(enabled());
 assert!(due(false,0,0,0));assert!(!due(true,7,7,29999));assert!(due(true,7,7,30000));
 assert!(!due(true,7,8,999));assert!(due(true,7,8,1000));
 let before=emitted();event(1,"first",[0;4]);event(1,"repeat",[0;4]);assert_eq!(emitted(),before+1);tick(f32::NAN);tick(1.);event(1,"repeat",[0;4]);assert_eq!(emitted(),before+1);for _ in 0..29{tick(1.);}event(1,"periodic",[0;4]);assert_eq!(emitted(),before+2);
 assert!(set_enabled(false));assert!(!enabled());assert!(set_enabled(true));event(1,"new session",[0;4]);set_enabled(false);
}
#[test]fn stock_decoder_reports_unrelated_invalid_slot_and_recovers(){
 let mut stock=[0;4096];let mut reads=0;
 inventory::collect(&mut stock,|source,slot|{reads+=1;if slot==0{(7,if source==0{99}else{2})}else{(0,0)}}).unwrap();
 assert_eq!(reads,2350);assert_eq!(stock[7],101);
 let error=inventory::collect(&mut stock,|source,slot|if source==0&&slot==2299{(12,100)}else{(0,0)}).unwrap_err();
 assert_eq!(error,inventory::Failure{source:0,slot:2299,id:12,count:100});
 let error=inventory::collect(&mut stock,|source,slot|if source==1&&slot==49{(2992,1)}else{(0,0)}).unwrap_err();
 assert_eq!(error,inventory::Failure{source:1,slot:49,id:2992,count:1});
 inventory::collect(&mut stock,|_,_|(0,0)).unwrap();assert!(stock.iter().all(|n|*n==0));
}
