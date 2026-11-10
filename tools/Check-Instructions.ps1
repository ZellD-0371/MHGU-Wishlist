$ErrorActionPreference = 'Stop'
$taskObjdump = Get-Command arm-none-eabi-objdump -ErrorAction Stop
$taskElf = Join-Path $PSScriptRoot '../target/armv7a-none-eabi/release/libwishlist.so'
$taskDisassembly = & $taskObjdump.Source -d $taskElf
if ($LASTEXITCODE -ne 0) { throw 'Could not inspect plugin instructions' }
# The user's Recomp rejects packed unsigned saturating arithmetic in plugins.
$taskUnsupported = $taskDisassembly | Select-String -Pattern '\b(uqadd8|uqadd16|uqsub8|uqsub16|uqasx|uqsax)\b'
if ($taskUnsupported) {
    $taskUnsupported | ForEach-Object { Write-Host $_.Line }
    throw 'Plugin contains packed saturation instructions unsupported by the tested Recomp'
}
Write-Host 'Recomp packed saturation instruction check passed.'

# Compiling alone does not guarantee the runtime will load the NRO.
$taskNro = [IO.File]::ReadAllBytes((Join-Path $PSScriptRoot '../wishlist.nro'))
$taskRequired = [int64]$taskNro.Length + [BitConverter]::ToUInt32($taskNro, 0x38) + 0x10000
if ($taskRequired -gt 0xC00000) { throw "NRO exceeds the Recomp plugin memory region ($taskRequired bytes)" }
Write-Host "Recomp plugin load size check passed: $taskRequired / 12582912 bytes."
