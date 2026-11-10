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
 & cargo +nightly check --offline
 if ($LASTEXITCODE -ne 0) { throw 'ARM32 checks failed' }
} finally { Pop-Location }
