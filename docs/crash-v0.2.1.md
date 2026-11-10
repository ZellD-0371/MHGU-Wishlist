# v0.2.1 UI 크래시 분석

- 로그: `crash-reports/v0.2.1-imgui-mirror-limit.log`.
- 최초 관련 메시지: `[forge] too many ImGui objects for plugins`.
- 최종 예외: `c0000005`, 주소 `0x34`, guest `r0=0`, LR `03201964`.
- 해당 v0.2.1 ELF의 `Canvas::rect` 주소 `0x1960`이 `ImDrawList_AddRectFilled`를 호출합니다. LR `0x1964`와 일치합니다. 첫 인자인 draw-list가 0인 상태입니다.

## 원인

실행기 `runtime/forge_imgui_thunks.inc`의 `mf_ImDrawList`는 `_VtxWritePtr` (guest +48, host +64)을 `MF_OBJ / ImDrawVert`로 분류했습니다. 이 포인터는 독립 객체가 아니라 가변 꼭짓점 버퍼 안의 쓰기 커서입니다. 그리기 호출 뒤 동기화할 때마다 변경된 포인터를 새 객체로 등록하여 8,192개 mirror 한도를 소비합니다. 기존 아이콘의 사각형 단위 그리기가 이를 빠르게 악화시킵니다. 한도 초과 뒤 foreground draw-list 조회가 0을 반환했고, 방어 없이 그리기 호출로 넘겨 크래시가 발생했습니다.

## 반영한 변경

- Wishlist: HD 아이콘을 하나의 RGBA 텍스처에 올려 아이콘당 이미지 한 번으로 표시합니다. 픽셀마다 사각형을 생성하지 않습니다.
- Wishlist: 프레임마다 유효한 draw-list를 한 번 얻어 공유하고 0이면 그리기를 생략합니다. 아무 패널도 필요 없는 상태와 타이틀에서는 조회도 하지 않습니다.
- Wishlist: 실제 표시 글꼴 크기를 정수 픽셀로 제한하여 불필요한 크기별 글꼴 캐시 증가를 줄입니다.
- 실행기 소스: `tools/imgui_thunks.py`에서 이 커서를 `MF_DATA`로 분류하고 생성된 `forge_imgui_thunks.inc`를 갱신했습니다. 전체 수정은 6줄 추가, 1줄 삭제입니다.

## 검증 / 적용 범위

- 기존 ELF의 예외 주소와 호출 인자 대조 완료.
- 생성기를 다시 실행하여 `_VtxWritePtr` 한 곳이 `MF_DATA`로 바뀌는 것을 확인했습니다.
- 모드 자체의 검사 및 release 패키징 결과는 `VALIDATION.md`에 기록합니다.
- 실행기의 동일한 3개 연속 descriptor를 바이너리에서도 유일하게 확인했습니다. 읽기 전용 PE 데이터 영역인지 검증한 뒤 `_VtxWritePtr`의 kind/type **2바이트만** 수정했습니다. 실행 코드와 ABI는 그대로입니다.
- 적용 파일: `Kone/MHGU/mhgurecomp.exe`. 원본: `MHGU-Wishlist/target/runtime-backup/mhgurecomp.before-imgui-cursor-fix.exe`. 원본·수정본 SHA-256과 변경 위치는 같은 폴더의 `imgui-cursor-fix.json`에 보존합니다.
- 별도 복사본에서 적용 → 수정 상태 인식 → 원본 복원을 실행하고 바이트 전체가 일치함을 검사했습니다. 현재 실제 실행기에도 수정본이 적용되었습니다. 전체 재빌드는 하지 않았습니다.
- `tools/fix_recomp_imgui.py`는 일치하는 실행기에서만 적용하며, 기존 백업을 덮어쓰지 않습니다. 패키지는 EXE를 포함하지 않습니다.
- 장시간 게임 실행에서 완전한 재발 방지 여부는 사용자 재확인이 필요합니다.

원본 복구 (작업 공간 루트에서 게임 종료 후):

```powershell
python MHGU-Wishlist/tools/fix_recomp_imgui.py Kone/MHGU/mhgurecomp.exe --restore --backup-dir MHGU-Wishlist/target/runtime-backup
```
