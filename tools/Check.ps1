param([switch]$UIRegression, [string]$UiPython = "python")
$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
 & python tools/prepare_low_memory.py --check
 if ($LASTEXITCODE -ne 0) { throw 'LowMemory atlas is stale or invalid' }
 & python tools/prepare_hd_atlas.py --check
 if ($LASTEXITCODE -ne 0) { throw 'HD cropped atlas is stale or invalid' }
 & python tools/prepare_names.py
 if ($LASTEXITCODE -ne 0) { throw 'Name resource generation failed' }
 & python tools/prepare_languages.py --check
 if ($LASTEXITCODE -ne 0) { throw 'Language resources are stale or invalid' }
 New-Item -ItemType Directory -Force target/host-check | Out-Null
 & rustc +nightly --edition 2024 --test tools/check_model.rs -Awarnings -o target/host-check/check_model.exe
 if ($LASTEXITCODE -ne 0) { throw 'Could not compile model checks' }
 & ./target/host-check/check_model.exe
 if ($LASTEXITCODE -ne 0) { throw 'Model checks failed' }
 & rustc +nightly --edition 2024 --test tools/check_language_switch.rs -Awarnings -o target/host-check/check_language_switch.exe
 if ($LASTEXITCODE -ne 0) { throw 'Could not compile language-switch checks' }
 & ./target/host-check/check_language_switch.exe
 if ($LASTEXITCODE -ne 0) { throw 'Language-switch checks failed' }
 & rustc +nightly --edition 2024 --test tools/check_lazy_descriptions.rs -Awarnings -o target/host-check/check_lazy_descriptions.exe
 if ($LASTEXITCODE -ne 0) { throw 'Could not compile demand-description checks' }
 & ./target/host-check/check_lazy_descriptions.exe
 if ($LASTEXITCODE -ne 0) { throw 'Demand-description checks failed' }
 & rustc +nightly --edition 2024 --test -O tools/check_ui_caches.rs -Awarnings -o target/host-check/check_ui_caches.exe
 if ($LASTEXITCODE -ne 0) { throw 'Could not compile UI cache checks' }
 & ./target/host-check/check_ui_caches.exe
 if ($LASTEXITCODE -ne 0) { throw 'UI cache checks failed' }
 & rustc +nightly --edition 2024 --test -O tools/check_font_validation.rs -Awarnings -o target/host-check/check_font_validation.exe
 if ($LASTEXITCODE -ne 0) { throw 'Could not compile streamed font validation checks' }
 & ./target/host-check/check_font_validation.exe
 if ($LASTEXITCODE -ne 0) { throw 'Streamed font validation checks failed' }
 & python tools/check_badges.py
 if ($LASTEXITCODE -ne 0) { throw 'Native badge replay checks failed' }
 & cargo +nightly check --offline
 if ($LASTEXITCODE -ne 0) { throw 'ARM32 checks failed' }
 if ($UIRegression) {
  & $UiPython -c 'import PIL, fontTools'
  if ($LASTEXITCODE -ne 0) { throw 'UI checks require Pillow and fontTools. Set -UiPython to a Python interpreter with those packages.' }
  & $UiPython tools/check_language_font.py
  if ($LASTEXITCODE -ne 0) { throw 'Font glyph/metric checks failed' }
  & $UiPython tools/render_ui_capture.py --matrix
  if ($LASTEXITCODE -ne 0) { throw 'UI regression failed. See target/ui-steel-proof/regression-report.json and the command output.' }
 }
} finally { Pop-Location }
