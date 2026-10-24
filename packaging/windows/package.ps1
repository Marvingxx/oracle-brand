<#
.SYNOPSIS
  Build and package Oracle Brand Kit and Oracle Mockup Studio for Windows (x64).

.DESCRIPTION
  Produces, in dist/release:
    oracle-brand-kit-<version>-windows-x64.msi       Brand Kit + the oracle-brand CLI
    oracle-mockup-studio-<version>-windows-x64.msi   Mockup Studio
    oracle-assets-<version>-windows-x64.msi          Oracle Assets (the library as an app)
    oracle-font-<version>-windows-x64.msi            Oracle Font (fonts and type tests)
  The binaries link the C runtime statically. Needs Rust (MSVC) and WiX v5 with the UI and Util
  extensions (see oracle-vector/packaging/windows/package.ps1).

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File packaging/windows/package.ps1
#>
param([switch] $SkipBuild)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path

function Invoke-Native([string] $What, [scriptblock] $Block) {
  Write-Output "==> $What"
  & $Block
  if ($LASTEXITCODE -ne 0) { throw "$What failed with exit code $LASTEXITCODE" }
}

$Version = $null
$inPkg = $false
foreach ($line in Get-Content (Join-Path $Root 'Cargo.toml')) {
  if ($line -match '^\s*\[') { $inPkg = ($line.Trim() -eq '[workspace.package]'); continue }
  if ($inPkg -and $line -match '^\s*version\s*=\s*"([^"]+)"') { $Version = $Matches[1]; break }
}
if (-not $Version) { throw 'could not read [workspace.package] version from Cargo.toml' }

$Target = 'x86_64-pc-windows-msvc'
$Dist = Join-Path $Root 'dist\release'
$TargetDir = Join-Path $Root 'target'
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

if (-not $SkipBuild) {
  [Environment]::SetEnvironmentVariable('CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS', '-C target-feature=+crt-static')
  Invoke-Native "cargo build ($Target)" { cargo build --release -p oracle-brand-kit -p oracle-mockup-studio -p oracle-assets -p oracle-font -p oracle-brand --target $Target }
}
$Bin = Join-Path $TargetDir "$Target\release"

# The installer art: the app icon on white (banner) and on a paper panel with the Oracle red
# stripe (Welcome and Finish pages). 24-bit BMPs.
function New-InstallerArt([string] $IconPng, [string] $BannerOut, [string] $DialogOut) {
  Add-Type -AssemblyName System.Drawing
  $paper = [System.Drawing.ColorTranslator]::FromHtml('#f2f1ee')
  $accent = [System.Drawing.ColorTranslator]::FromHtml('#d31c3b')
  $icon = [System.Drawing.Image]::FromFile($IconPng)
  try {
    foreach ($spec in @(@($BannerOut, 493, 58), @($DialogOut, 493, 312))) {
      $bmp = New-Object System.Drawing.Bitmap $spec[1], $spec[2], ([System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
      $g = [System.Drawing.Graphics]::FromImage($bmp)
      try {
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.Clear([System.Drawing.Color]::White)
        if ($spec[2] -eq 58) {
          $g.DrawImage($icon, 493 - 46 - 8, 6, 46, 46)
        } else {
          $panel = New-Object System.Drawing.SolidBrush $paper
          $stripe = New-Object System.Drawing.SolidBrush $accent
          $g.FillRectangle($panel, 0, 0, 164, 312)
          $g.FillRectangle($stripe, 160, 0, 4, 312)
          $g.DrawImage($icon, 18, 64, 124, 124)
          $panel.Dispose(); $stripe.Dispose()
        }
      } finally { $g.Dispose() }
      $bmp.Save($spec[0], [System.Drawing.Imaging.ImageFormat]::Bmp)
      $bmp.Dispose()
    }
  } finally { $icon.Dispose() }
}

$Apps = @(
  @{ Id = 'oracle-brand-kit'; Name = 'Oracle Brand Kit'; Dir = 'kit'; Icon = 'brand-kit'; Cli = 'oracle-brand.exe'
     Upgrade = '560E04C1-AEF8-4D39-8935-334C3BDE375D'; Main = '69B68C93-F2C3-492A-8D47-79EE79C3BB18'; Desktop = '190A190C-807F-40D3-AAE2-7E0376064284' },
  @{ Id = 'oracle-mockup-studio'; Name = 'Oracle Mockup Studio'; Dir = 'mockup'; Icon = 'mockup-studio'; Cli = $null
     Upgrade = '79D984EA-F2D8-4600-BB2E-F2339DC70978'; Main = 'E368B8E2-6C6F-4C49-B4A5-3298B3A45E09'; Desktop = '21D167D0-009C-4B26-BC41-44AD315B8C4D' },
  @{ Id = 'oracle-assets'; Name = 'Oracle Assets'; Dir = 'oracle-assets'; Icon = 'oracle-assets'; Cli = $null
     Upgrade = '2B0EFE68-AD91-47E2-A18A-A898ACD2E0B3'; Main = '7CF77FD3-FF48-4DF5-8BA3-9DEB78BA2113'; Desktop = '554D0437-046A-4F4C-B521-A4C81F451A94' },
  @{ Id = 'oracle-font'; Name = 'Oracle Font'; Dir = 'oracle-font'; Icon = 'oracle-font'; Cli = $null
     Upgrade = 'FD52EB49-E7B0-42DD-B7AF-BA32499376B1'; Main = '867FA9F2-C25C-4BB7-A0F3-BF7AF8364112'; Desktop = '5B94359F-6996-4ED6-A483-4BFEBC355939' }
)
foreach ($a in $Apps) {
  $Stage = Join-Path $TargetDir "windows-package\$($a.Id)"
  Remove-Item -Recurse -Force $Stage -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $Stage | Out-Null
  Copy-Item (Join-Path $Bin "$($a.Id).exe") $Stage
  if ($a.Cli) { Copy-Item (Join-Path $Bin $a.Cli) $Stage }
  $Docs = Join-Path $Stage 'docs'
  New-Item -ItemType Directory -Force -Path $Docs | Out-Null
  Copy-Item (Join-Path $Root 'README.md') $Docs
  Copy-Item (Join-Path $Root 'style\assets\OFL-*.txt') $Docs
  $Banner = Join-Path $Stage 'installer-banner.bmp'
  $Dialog = Join-Path $Stage 'installer-dialog.bmp'
  $IconDir = Join-Path $Root "$($a.Dir)\assets"
  New-InstallerArt (Join-Path $IconDir "hicolor\256x256\apps\app.oracle.$($a.Icon).png") $Banner $Dialog
  $Msi = Join-Path $Dist "$($a.Id)-$Version-windows-x64.msi"
  $cliArg = @()
  if ($a.Cli) { $cliArg = @('-d', "CliExe=$($a.Cli)") }
  Invoke-Native "wix build $($a.Id)" {
    wix build (Join-Path $PSScriptRoot 'app.wxs') -arch x64 -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
      -d "ProductName=$($a.Name)" -d "AppId=$($a.Id)" -d "UpgradeCode=$($a.Upgrade)" -d "MainGuid=$($a.Main)" -d "DesktopGuid=$($a.Desktop)" `
      -d "Version=$Version" -d "BinDir=$Stage" -d "IconPath=$(Join-Path $IconDir "$($a.Icon).ico")" -d "DocsDir=$Docs" `
      -d "BannerBmp=$Banner" -d "DialogBmp=$Dialog" @cliArg -o $Msi
  }
  Remove-Item -Force -ErrorAction SilentlyContinue ([IO.Path]::ChangeExtension($Msi, '.wixpdb'))
  Get-Item $Msi | Format-Table Name, Length
}
