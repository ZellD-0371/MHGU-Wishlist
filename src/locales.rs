pub const CODES:[&str;9]=["en","zh-Hans","zh-Hant","ja","ko","de","fr","it","es"];
pub const LANGUAGES:[&str;9]=["English","简体中文","繁體中文","日本語","한국어","Deutsch","Français","Italiano","Español"];
pub const LANGUAGE:[&str;9]=["Language","语言","語言","言語","언어","Sprache","Langue","Lingua","Idioma"];
// Every UI string has exactly nine entries in the requested stable language order.
pub type Label=[&'static str;9];
pub const TITLE:Label=["Wishlist","愿望清单","願望清單","ウィッシュリスト","찜 목록","Wunschliste","Liste de souhaits","Lista desideri","Lista de deseos"];
pub const REGISTER:Label=["Add to wishlist","加入愿望清单","加入願望清單","リストに登録","찜 등록","Zur Wunschliste","Ajouter à la liste","Aggiungi alla lista","Añadir a la lista"];
pub const REGISTER_ACTION:Label=["Add to wishlist","加入愿望清单","加入願望清單","ウィッシュリストに登録","찜 목록에 등록","Zur Wunschliste hinzufügen","Ajouter à la liste de souhaits","Aggiungi alla lista desideri","Añadir a la lista de deseos"];
pub const REGISTERED_SLOT:Label=["(Already registered)","（已登记）","（已登錄）","（登録済みです）","(이미 등록되어 있습니다)","(Bereits eingetragen)","(Déjà enregistré)","(Già registrato)","(Ya registrado)"];
pub const AUTO_SAVE:Label=["Settings are saved automatically.","设置会自动保存。","設定會自動儲存。","設定は自動保存されます。","설정은 자동으로 저장됩니다.","Einstellungen werden automatisch gespeichert.","Les réglages sont enregistrés automatiquement.","Le impostazioni vengono salvate automaticamente.","Los ajustes se guardan automáticamente."];
pub const NOTICE_TEST:Label=["Notification test","通知测试","通知測試","通知テスト","소재 충족 알림 테스트","Benachrichtigungstest","Test des notifications","Test notifiche","Prueba de notificaciones"];
pub const TEST_CASE:Label=["Test case","测试场景","測試情境","テスト項目","테스트 종류","Testfall","Scénario de test","Tipo di test","Caso de prueba"];
pub const NOTICE_SCENARIOS:[Label;3]=[
 ["2 at once","同时2个","同時2個","2件同時","2개 동시 표시","2 gleichzeitig","2 simultanées","2 simultanee","2 simultáneas"],
 ["2 with a 2-second interval","2个，间隔2秒","2個，間隔2秒","2秒間隔で2件","2초 간격으로 2개","2 im Abstand von 2 Sekunden","2 espacées de 2 secondes","2 a distanza di 2 secondi","2 con intervalo de 2 segundos"],
 ["4 at once (2 visible + 2 queued)","同时4个（显示2个、等待2个）","同時4個（顯示2個、等待2個）","4件同時（表示2件・待機2件）","4개 동시 (표시 2개 + 대기 2개)","4 gleichzeitig (2 sichtbar + 2 wartend)","4 simultanées (2 visibles + 2 en attente)","4 simultanee (2 visibili + 2 in coda)","4 simultáneas (2 visibles + 2 en cola)"]
];
pub const NOTICE_TEST_HELP:Label=["Sample notifications only. Inventory and wishlist stay unchanged.\nA: replay. D-pad left/right: change test. B: close.","仅显示示例通知，不改变素材或清单。\nA重播，十字键左右切换，B关闭。","僅顯示範例通知，不變更素材或清單。\nA重播，方向鍵左右切換，B關閉。","サンプル通知のみ。所持品・登録内容は変更しません。\nAで再生、十字キー左右で切替、Bで閉じます。","예제 알림만 표시하며 소재와 찜 목록은 변경하지 않습니다.\nA 다시 재생 / 십자키 좌우 테스트 변경 / B 닫기","Nur Beispielmeldungen. Inventar und Liste bleiben erhalten.\nA: erneut. Steuerkreuz links/rechts: Test ändern. B: schließen.","Exemples uniquement. Inventaire et liste inchangés.\nA : rejouer. Gauche/droite : autre test. B : fermer.","Solo notifiche di esempio. Inventario e lista invariati.\nA: ripeti. Sinistra/destra: cambia test. B: chiudi.","Solo avisos de ejemplo. Inventario y lista sin cambios.\nA: repetir. Izquierda/derecha: cambiar prueba. B: cerrar."];
pub const VISIBLE:Label=["Visible","显示","顯示","表示","표시 중","Sichtbar","Visibles","Visibili","Visibles"];
pub const QUEUED:Label=["Queued","等待","等待","待機","대기","Wartend","En attente","In coda","En cola"];
pub const TEST_FINISHED:Label=["Complete — press A to replay.","已完成，按A重播。","已完成，按A重播。","完了。Aで再生します。","테스트 완료 — A로 다시 재생합니다.","Fertig — A zum Wiederholen.","Terminé — A pour rejouer.","Completato — A per ripetere.","Completado — A para repetir."];
pub const REPLAY:Label=["Replay","重播","重播","再生","다시 재생","Wiederholen","Rejouer","Ripeti","Repetir"];
pub const REGISTER_TITLE:Label=["Register equipment","登记装备","登錄裝備","装備の登録","찜 목록 등록 화면","Ausrüstung eintragen","Enregistrer un équipement","Registra equipaggiamento","Registrar equipo"];
pub const TARGET:Label=["Equipment to register","要登记的装备","要登錄的裝備","登録する装備","등록할 장비","Einzutragende Ausrüstung","Équipement à enregistrer","Equipaggiamento da registrare","Equipo que registrar"];
pub const LIST:Label=["Registered equipment","已登记装备","已登錄裝備","登録リスト","등록 목록","Eingetragene Ausrüstung","Équipements enregistrés","Equipaggiamento registrato","Equipo registrado"];
pub const TOTAL:Label=["Total materials","所需素材合计","所需素材合計","合計必要素材","합계 필요 소재","Materialbedarf gesamt","Total des matériaux","Materiali totali","Materiales totales"];
pub const MATERIALS:Label=["Required materials","所需素材","所需素材","必要素材","필요 소재","Benötigte Materialien","Matériaux requis","Materiali necessari","Materiales necesarios"];
pub const EMPTY:Label=["Empty","未登记","未登錄","未登録","미등록","Leer","Vide","Vuoto","Vacío"];
pub const READY:Label=["Materials ready","素材已齐备","素材已齊備","素材が揃いました","소재 충족","Materialien vorhanden","Matériaux réunis","Materiali disponibili","Materiales reunidos"];
pub const WAITING:Label=["Gathering materials","素材收集中","素材收集中","素材収集中","소재 수집 중","Materialien sammeln","Collecte en cours","Raccolta materiali","Reuniendo materiales"];
pub const NOTICE:Label=["All required materials have been gathered.","所需素材已齐备。","所需素材已齊備。","必要素材が揃いました。","필요 소재가 갖추어졌습니다.","Alle benötigten Materialien sind vorhanden.","Tous les matériaux nécessaires sont réunis.","Tutti i materiali necessari sono disponibili.","Has reunido todos los materiales necesarios."];
pub const CONFIRM:Label=["Confirm","确定","確定","決定","결정","Bestätigen","Confirmer","Conferma","Confirmar"];
pub const BACK:Label=["Back","返回","返回","戻る","돌아가기","Zurück","Retour","Indietro","Volver"];
pub const DELETE:Label=["Remove","删除","刪除","登録解除","등록 삭제","Entfernen","Supprimer","Rimuovi","Eliminar"];
pub const DELETE_QUESTION:Label=["Delete the registered contents.\nContinue?","删除登记内容。\n是否继续？","刪除登錄內容。\n是否繼續？","登録内容を削除します。\nよろしいですか？","등록 내용을 삭제합니다.\n그렇게 하시겠습니까?","Registrierten Eintrag löschen.\nFortfahren?","Supprimer le contenu enregistré.\nContinuer ?","Eliminare il contenuto registrato.\nContinuare?","Se eliminará el registro.\n¿Continuar?"];
pub const REPLACE_QUESTION:Label=["Overwrite registered contents.\nContinue?","将覆盖登记内容。\n是否继续？","將覆蓋登錄內容。\n是否繼續？","登録内容を上書きします。\nよろしいですか？","등록 내용을 덮어씁니다.\n그렇게 하시겠습니까?","Eintrag wird überschrieben.\nFortfahren?","Écraser le contenu enregistré.\nContinuer ?","Sovrascrivere il contenuto.\nContinuare?","Se sobrescribirá el registro.\n¿Continuar?"];
pub const YES:Label=["Yes","是","是","はい","예","Ja","Oui","Sì","Sí"];
pub const NO:Label=["No","否","否","いいえ","아니요","Nein","Non","No","No"];
pub const NO_EQUIPMENT:Label=["No equipment","无装备","無裝備","装備なし","장비 없음","Keine Ausrüstung","Aucun équipement","Nessun equipaggiamento","Sin equipo"];
pub const CAPTURE_SOUND:Label=["Start sound capture","开始捕获声音","開始擷取音效","効果音の記録開始","소리 캡처 시작","Tonaufnahme starten","Capturer les sons","Cattura suoni","Capturar sonidos"];
pub const STOP_SOUND:Label=["Stop sound capture","停止捕获","停止擷取","記録停止","소리 캡처 중지","Aufnahme stoppen","Arrêter la capture","Ferma cattura","Detener captura"];
pub const DUPLICATE:Label=["This recipe is already registered.","此配方已登记。","此配方已登錄。","このレシピは登録済みです。","이미 등록한 장비·레시피입니다.","Dieses Rezept ist bereits eingetragen.","Cette recette est déjà enregistrée.","Questa ricetta è già registrata.","Esta receta ya está registrada."];
pub const SAVED:Label=["Registered and saved.","已登记并保存。","已登錄並儲存。","登録して保存しました。","찜 목록에 등록하고 저장했습니다.","Eingetragen und gespeichert.","Enregistré et sauvegardé.","Registrato e salvato.","Registrado y guardado."];
pub const SAVE_FAILED:Label=["Save failed. Your change was not applied. Try again.","保存失败，未应用更改。请重试。","儲存失敗，未套用變更。請重試。","保存に失敗しました。変更は適用されません。","저장하지 못해 변경을 적용하지 않았습니다. 다시 시도하세요.","Speichern fehlgeschlagen. Änderung nicht übernommen.","Échec de sauvegarde. Modification non appliquée.","Salvataggio fallito. Modifica non applicata.","Error al guardar. No se aplicó el cambio."];
pub const PICK:Label=["Select a registration slot.","请选择登记目标。","請選擇登錄目標。","登録先を選択してください。","등록 대상을 선택하십시오.","Wähle einen Eintragsplatz.","Sélectionnez un emplacement d’inscription.","Seleziona lo spazio di registrazione.","Selecciona el espacio de registro."];
pub const NO_TARGET:Label=["Select equipment at the smithy to register it.","请在加工店选择装备。","請在加工店選擇裝備。","加工屋で装備を選択してください。","가공점에서 등록할 장비를 선택하세요.","Ausrüstung beim Schmied auswählen.","Sélectionnez un équipement à la forge.","Seleziona un equipaggiamento alla forgia.","Selecciona un equipo en la forja."];
pub const GROUP:Label=["Category upgrade materials","类别强化素材","類別強化素材","カテゴリ強化素材","카테고리 강화 소재","Kategorie-Aufwertungsmaterial","Matériaux de catégorie","Materiali potenziamento categoria","Materiales de mejora de categoría"];
pub const SHARED:Label=["Check the registered materials.","查看已登记的素材。","查看已登錄的素材。","登録されている素材を確認します。","등록되어 있는 소재를 확인합니다.","Registrierte Materialien prüfen.","Consultez les matériaux enregistrés.","Controlla i materiali registrati.","Consulta los materiales registrados."];
pub const FORGE:Label=["Forge","生产","生產","生産","생산","Herstellen","Forge","Forgia","Forjar"];
pub const UPGRADE:Label=["Upgrade","强化","強化","強化","강화","Verbessern","Amélioration","Migliora","Mejorar"];
pub const PREVIEW:Label=["Inspect UI panels","检查界面面板","檢查介面面板","UIパネル確認","UI 패널 검수","UI-Panels prüfen","Inspecter les panneaux","Controlla pannelli UI","Revisar paneles UI"];
pub const DETAILS:Label=["Equipment details","装备详情","裝備詳情","装備詳細","장비 세부 정보","Ausrüstungsdetails","Détails équipement","Dettagli equipaggiamento","Detalles del equipo"];
pub const SKILLS:Label=["Equipment skills","装备技能","裝備技能","装備スキル","장비 스킬","Ausrüstungsfähigkeiten","Talents équipement","Abilità equipaggiamento","Habilidades del equipo"];
pub const COST:Label=["Required zenny","所需金钱","所需金錢","必要金額","필요 금액","Benötigte Zenny","Zenny requis","Zenny necessari","Zenny necesarios"];
pub const CLOSE:Label=["Close","关闭","關閉","閉じる","닫기","Schließen","Fermer","Chiudi","Cerrar"];
pub const DONE:Label=["Registered.","已登记。","已登錄。","登録しました。","등록했습니다.","Eingetragen.","Enregistré.","Registrato.","Registrado."];
pub const RARITY:Label=["Rarity","稀有度","稀有度","レア度","희귀도","Seltenheit","Rareté","Rarità","Rareza"];
pub const ATTACK:Label=["Attack","攻击力","攻擊力","攻撃力","공격력","Angriff","Attaque","Attacco","Ataque"];
pub const DEFENSE:Label=["Base defense","基础防御力","基礎防禦力","基本防御力","기본 방어력","Basis-Verteidigung","Défense de base","Difesa base","Defensa base"];
pub const SLOTS:Label=["Slots","孔位","孔位","スロット","슬롯","Slots","Fentes","Slot","Ranuras"];
pub const RESIST:Label=["Fire / Water / Thunder / Ice / Dragon","火 / 水 / 雷 / 冰 / 龙","火 / 水 / 雷 / 冰 / 龍","火 / 水 / 雷 / 氷 / 龍","불 / 물 / 번개 / 얼음 / 용","Feuer / Wasser / Donner / Eis / Drache","Feu / Eau / Foudre / Glace / Dragon","Fuoco / Acqua / Tuono / Ghiaccio / Drago","Fuego / Agua / Rayo / Hielo / Dragón"];
pub const UNKNOWN:Label=["Inventory unavailable","无法读取持有素材","無法讀取持有素材","所持素材を取得できません","소지 소재를 확인할 수 없습니다","Inventar nicht verfügbar","Inventaire indisponible","Inventario non disponibile","Inventario no disponible"];

pub const BUTTON_LAYOUT:Label=["Button layout","按键布局","按鍵配置","ボタン配置","버튼 배치","Tastenbelegung","Disposition des touches","Layout pulsanti","Disposición de botones"];
pub const SHARPNESS:Label=["Sharpness","斩味","斬味","斬れ味","예리도","Schärfe","Tranchant","Affilatura","Afilado"];
pub const AFFINITY:Label=["Affinity","会心率","會心率","会心率","회심률","Affinität","Affinité","Affinità","Afinidad"];

pub const SOUND_IDS:Label=["Open: 1 | Close: 2 | Cursor: 4 | Inspect: 6\nPage: 7 | Tabs: 27 | Registered: 31","打开: 1 | 关闭: 2 | 光标: 4 | 查看: 6\n翻页: 7 | 标签页: 27 | 登记完成: 31","開啟: 1 | 關閉: 2 | 游標: 4 | 查看: 6\n翻頁: 7 | 分頁: 27 | 登錄完成: 31","開く: 1 | 戻る: 2 | カーソル: 4 | 確認: 6\nページ: 7 | タブ: 27 | 登録完了: 31","열기: 1 | 닫기: 2 | 커서: 4 | 상세: 6\n페이지: 7 | 탭: 27 | 등록 완료: 31","Öffnen: 1 | Zurück: 2 | Cursor: 4 | Details: 6\nSeite: 7 | Tabs: 27 | Eingetragen: 31","Ouvrir : 1 | Retour : 2 | Curseur : 4 | Détails : 6\nPage : 7 | Onglets : 27 | Inscription : 31","Apri: 1 | Indietro: 2 | Cursore: 4 | Dettagli: 6\nPagina: 7 | Schede: 27 | Registrato: 31","Abrir: 1 | Volver: 2 | Cursor: 4 | Detalles: 6\nPágina: 7 | Pestañas: 27 | Registro: 31"];
pub const ENTER_GAME:Label=["Load a character to open Wishlist.","载入角色后可打开愿望清单。","載入角色後可開啟願望清單。","キャラクターを読み込んでから開いてください。","캐릭터로 게임을 시작한 뒤 열 수 있습니다.","Zum Öffnen einen Charakter laden.","Chargez un personnage pour ouvrir la liste.","Carica un personaggio per aprire la lista.","Carga un personaje para abrir la lista."];

pub const PICK_ENTRY:Label=["Select the registered entry to view.","请选择要查看的登记内容。","請選擇要查看的登錄內容。","確認したい登録内容を選択してください。","확인하고 싶은 등록 내용을 선택하십시오.","Wähle den Eintrag, den du ansehen möchtest.","Sélectionnez le contenu enregistré à consulter.","Seleziona la voce registrata da visualizzare.","Selecciona el registro que quieras consultar."];

pub const INSPECT_ENTRY:Label=["View the registered contents.","查看登记内容。","查看登錄內容。","登録内容を確認します。","등록 내용을 확인합니다.","Registrierten Eintrag ansehen.","Consultez le contenu enregistré.","Visualizza il contenuto registrato.","Consulta el contenido registrado."];

pub const HOTKEYS:Label=["R3 / F5: Register / Remove | `: Wishlist\nL3 + R3: Wishlist at smithy or + menu","R3 / F5: 登记 / 删除 | `: 愿望清单\nL3 + R3: 加工店或 + 菜单","R3 / F5: 登錄 / 刪除 | `: 願望清單\nL3 + R3: 加工店或 + 選單","R3 / F5: 登録 / 解除 | `: リスト\nL3 + R3: 加工屋または + メニュー","R3 / F5: 찜 등록 / 삭제 | `: 찜 목록\nL3 + R3: 가공점 또는 + 메뉴에서 찜 목록 열기","R3 / F5: Eintragen / Entfernen | `: Wunschliste\nL3 + R3: Schmiede oder +-Menü","R3 / F5 : Inscrire / Retirer | F6 : Liste\nL3 + R3 : Forge ou menu +","R3 / F5: Registra / Rimuovi | `: Lista\nL3 + R3: Forgia o menu +","R3 / F5: Registrar / Quitar | `: Lista\nL3 + R3: Herrería o menú +"];

pub const QUICK_REMOVE:Label=["Remove wishlist","取消愿望清单","取消願望清單","登録解除","찜 삭제","Wunsch entfernen","Retirer de la liste","Rimuovi dalla lista","Quitar de la lista"];
