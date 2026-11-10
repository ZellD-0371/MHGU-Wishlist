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
