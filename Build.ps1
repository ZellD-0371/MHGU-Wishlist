param([switch]$SkipPackage, [switch]$Offline, [switch]$LowMemory, [string]$ResourcePython = "python")
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
$taskOldPath = $env:PATH
$taskOldOffline = $env:CARGO_NET_OFFLINE
$taskOldLinker = $env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER
try {
    & python './tools/prepare_names.py'
    if ($LASTEXITCODE -ne 0) { throw 'Name resource generation failed' }
    & $ResourcePython './tools/prepare_languages.py'
    if ($LASTEXITCODE -ne 0) { throw 'Language resource generation failed (fontTools is required when regenerating fonts).' }
    & $ResourcePython './tools/prepare_low_memory.py'
    if ($LASTEXITCODE -ne 0) { throw 'LowMemory resource generation failed (Pillow is required when regenerating icons).' }
    & python './tools/prepare_hd_atlas.py'
    if ($LASTEXITCODE -ne 0) { throw 'HD atlas cropping failed' }
    if ($Offline) { $env:CARGO_NET_OFFLINE = 'true' }
    $taskArmRoot = $env:DEVKITARM
    if (-not $taskArmRoot -and $env:DEVKITPRO) { $taskArmRoot = Join-Path $env:DEVKITPRO 'devkitARM' }
    if ($taskArmRoot) { $env:PATH = (Join-Path $taskArmRoot 'bin') + [IO.Path]::PathSeparator + $env:PATH }
    $taskCompiler = Get-Command arm-none-eabi-gcc -ErrorAction SilentlyContinue
    if (-not $taskCompiler) { throw 'Install devkitARM and set DEVKITARM, or add arm-none-eabi-gcc to PATH.' }
    $env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER = $taskCompiler.Source
    & cargo +nightly forge build --release
    if ($LASTEXITCODE -ne 0) { throw 'Forge build failed' }
    & (Join-Path $PSScriptRoot 'tools/Check-Instructions.ps1')
    if (-not $SkipPackage) {
        $taskPython = Get-Command python -ErrorAction SilentlyContinue
        if (-not $taskPython) { $taskPython = Get-Command python3 -ErrorAction SilentlyContinue }
        if (-not $taskPython) { throw 'Python 3 is required for packaging. Use -SkipPackage to build only.' }
        # Both releases share the NRO. A normal build leaves both ZIPs in dist.
        if (-not $LowMemory) {
            & $taskPython.Source './tools/package.py'
            if ($LASTEXITCODE -ne 0) { throw 'Standard packaging failed' }
        }
        & $taskPython.Source './tools/package.py' --low-memory
        if ($LASTEXITCODE -ne 0) { throw 'LowMemory packaging failed' }
    }
} finally {
    $env:PATH = $taskOldPath
    $env:CARGO_NET_OFFLINE = $taskOldOffline
    $env:CARGO_TARGET_ARMV7A_NONE_EABI_LINKER = $taskOldLinker
    Pop-Location
}
