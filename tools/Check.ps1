param([switch]$UIRegression, [string]$UiPython = "python")
$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
 & python tools/prepare_names.py
 if ($LASTEXITCODE -ne 0) { throw 'Name resource generation failed' }
 New-Item -ItemType Directory -Force target/host-check | Out-Null
 & rustc +nightly --edition 2024 --test tools/check_model.rs -Awarnings -o target/host-check/check_model.exe
 if ($LASTEXITCODE -ne 0) { throw 'Could not compile model checks' }
 & ./target/host-check/check_model.exe
 if ($LASTEXITCODE -ne 0) { throw 'Model checks failed' }
 & python tools/check_badges.py
 if ($LASTEXITCODE -ne 0) { throw 'Native badge replay checks failed' }
 & cargo +nightly check --offline
 if ($LASTEXITCODE -ne 0) { throw 'ARM32 checks failed' }
 if ($UIRegression) {
  & $UiPython -c 'import PIL, fontTools'
  if ($LASTEXITCODE -ne 0) { throw 'UI checks require Pillow and fontTools. Set -UiPython to a Python interpreter with those packages.' }
  & $UiPython tools/render_ui_capture.py --matrix
  if ($LASTEXITCODE -ne 0) { throw 'UI regression failed. See target/ui-steel-proof/regression-report.json and the command output.' }
 }
} finally { Pop-Location }
