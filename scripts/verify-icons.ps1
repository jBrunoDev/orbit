$ErrorActionPreference = 'Stop'
$iconPath = Join-Path $PSScriptRoot '..\src-tauri\icons\icon.ico'
$bytes = [IO.File]::ReadAllBytes((Resolve-Path $iconPath))
if ($bytes.Length -lt 6) { throw "ICO inválido: cabeçalho incompleto ($iconPath)" }
$count = [BitConverter]::ToUInt16($bytes, 4)
$found = @()
for ($i = 0; $i -lt $count; $i++) {
  $offset = 6 + (16 * $i)
  if ($offset + 16 -gt $bytes.Length) { throw "ICO inválido: entrada $i truncada" }
  $width = if ($bytes[$offset] -eq 0) { 256 } else { [int]$bytes[$offset] }
  $height = if ($bytes[$offset + 1] -eq 0) { 256 } else { [int]$bytes[$offset + 1] }
  $found += "$width`x$height"
}
$required = @('16x16','24x24','32x32','48x48','64x64','128x128','256x256')
$missing = $required | Where-Object { $_ -notin $found }
if ($missing) { throw "ICO incompleto. Encontrados: $($found -join ', '). Faltando: $($missing -join ', ')" }
Write-Output "OK: $iconPath contém $($found -join ', ')"
