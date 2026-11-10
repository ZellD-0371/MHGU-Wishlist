# MHGU Wishlist

Monster Hunter Generations Ultimate의 가공점에 찜 목록을 추가하는 Forge 모드입니다. 만들거나 강화할 장비를 등록하고, 필요한 소재와 소지 수량을 확인할 수 있습니다. 소재가 모두 모이면 알림을 표시하며, 해당 장비의 생산·강화가 완료되면 등록을 자동으로 삭제합니다.

## 지원 환경

- MHGU 글로벌 버전 1.4.0
- Forge 플러그인을 지원하는 MHGU Recomp 혹은 Forge 1.5.1이 설치된 에뮬레이터

## 주요 기능

- 무기·방어구 생산, 장식품 생산, 무기·방어구 강화, 동반자 무기·방어구 생산 지원
- 가공점의 등록 장비 아이콘에 금색 찜 표시 및 바로 삭제 기능
- 장비별 필요 소재·필요 금액·카테고리 강화 소재 확인
- 전체 등록 장비의 합계 필요 소재와 해당 소재가 필요한 장비 확인
- 생산·강화 성공 시 해당 등록 자동 삭제
- Nintendo·Xbox·PlayStation 버튼 아이콘 선택
- 화면 비율을 유지하는 UI 및 50~100% UI 크기 설정

## 설치

1. 게임을 종료합니다.
2. `Wishlist_v1.0.0.zip`을 압축 해제합니다.
3. 압축 안의 `Wishlist` 폴더를 Recomp 실행 파일 옆의 `mods` 폴더에 넣습니다.
4. 게임을 실행합니다. 모드 관리에서 별도로 비활성화한 경우 활성화합니다.

```text
mods/
└─ Wishlist/
   └─ romfs/
      └─ nativeNX/
         └─ plugins/
            ├─ wishlist.nro
            └─ wishlist/
               ├─ ui-text.bin
               ├─ wishlist-atlas.rgba
               └─ NotoSansCJKkr-Regular.otf
```

## 사용 방법

### 목록 확인

가공점에서는 L3를 누른 채 R3를 눌러 목록을 엽니다. 마을·퀘스트 필드에서는 + 버튼으로 여는 게임 메뉴가 표시 중일 때 같은 단축키를 사용할 수 있습니다. 키보드에서는 백틱 키(숫자 1 왼쪽)로 캐릭터 진입 후 목록을 열 수 있습니다.

| 입력 — Nintendo 기준 | 기능 |
| --- | --- |
| 가공점 R3 / F5 | 선택 장비 등록 / 등록된 장비 삭제 확인 |
| 가공점 또는 게임 메뉴에서 L3 + R3 | 찜 목록 열기 |
| 백틱 키 ( ` ) | 찜 목록 열기 |
| 위·아래 | 현재 페이지의 항목 선택 |
| 십자키 좌·우 / PageUp·PageDown | 페이지 이동 |
| L·R / Tab | 등록 목록·합계 필요 소재 탭 전환 |
| ZL·ZR / Q·E | 장비 세부 정보 페이지 전환 |
| A / Enter | 결정 / 소재 상세 진입 |
| B / Esc | 취소·뒤로·닫기 |
| 등록 목록에서 X / Delete | 등록 삭제 확인 |

버튼 배치를 바꾸면 표시 아이콘이 달라집니다. 실제 컨트롤러 입력 매핑은 Recomp 설정을 따릅니다. 타이틀·캐릭터 선택 화면에서는 찜 목록을 열 수 없습니다.

## 설정

Insert로 Forge 설정을 열고 `Plugin UI > Wishlist`에서 변경합니다.

| 항목 | 내용 |
| --- | --- |
| Button layout | Nintendo / Xbox / PlayStation |
| Language | 지원 언어 9개 중 선택 |
| UI scale | 50 / 60 / 70 / 80 / 90 / 100% |

지원 언어: `en`, `zh-Hans`, `zh-Hant`, `ja`, `ko`, `de`, `fr`, `it`, `es`.

## 저장 데이터

찜 목록과 설정은 게임의 현재 저장 영역에 별도 INI 파일로 보관합니다.

| 파일 | 내용 |
| --- | --- |
| `wishlist.ini` | 공통 설정 및 이전 버전 목록의 호환 정보 |
| `wishlist_character_1.ini` | 첫 번째 캐릭터의 찜 목록 |
| `wishlist_character_2.ini` | 두 번째 캐릭터의 찜 목록 |
| `wishlist_character_3.ini` | 세 번째 캐릭터의 찜 목록 |

Windows Recomp 기본 경로는 `%APPDATA%\MHGURecomp\save\save_0\`입니다. 실행 환경에서 저장 경로를 따로 지정했다면 해당 경로를 사용합니다. 모드를 업데이트해도 이 파일들은 교체하지 않습니다.

## 소스 빌드

필요 도구: Rust nightly와 `rust-src`, Forge Cargo 빌드 도구(`cargo forge`), devkitARM, Python 3.11 이상. Forge SDK는 `Cargo.toml`에 지정된 커밋을 사용합니다.

```powershell
./tools/Check.ps1
$env:DEVKITARM = 'D:/Utils/devkitPro/devkitARM'
./Build.ps1
```

의존성이 이미 준비되어 있다면 `./Build.ps1 -Offline`으로 빌드할 수 있습니다. 결과는 `wishlist.nro`, `dist/Wishlist`, `Wishlist_v1.0.0.zip`에 생성됩니다.

일반 배포에서는 소리 캡처·UI 패널 검수·알림 테스트를 숨깁니다. 개발 중 다시 표시하려면 다음과 같이 빌드합니다.

```powershell
cargo +nightly forge build --release --features debug-ui
```

오프라인 UI 배치 검사는 `tools/render_ui_capture.py`를 사용합니다. 추가 Python 의존성은 Pillow와 fontTools이며, 출력은 `target/ui-steel-proof`에 생성됩니다. 이 검사는 게임 안의 실제 조작 검사를 대신하지 않습니다.

## 저장소 구성

`src`, `.cargo`, `link`, `conf`, `assets`, `tools`는 소스 빌드·검사에 필요한 파일입니다. `Cargo.lock`과 생성된 Rust 데이터도 빌드 입력이므로 함께 보관합니다.

빌드 결과물, 배포 ZIP, 로컬 테스트 게임·세이브, 로그, 브라우저 프로필과 미리보기 캐시는 `.gitignore`로 제외합니다. `Run-Test.cmd`는 이 작업 공간의 별도 테스트 실행용이며 배포 설치에는 필요하지 않습니다.

