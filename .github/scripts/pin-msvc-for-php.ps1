#!/usr/bin/env pwsh
# Pin the MSVC toolset used for linking to one no newer than the PHP core's linker.
#
# PHP on Windows refuses to load an extension DLL that was linked with a newer
# linker than the PHP core itself. php.net pins
# its build toolset per PHP release line while the GitHub runner images roll
# forward, so we read the required version from the installed PHP core, select
# (and if needed install) a matching side-by-side toolset, and export its build
# environment for subsequent steps.
#
# Expects PHP to be on PATH already.

$ErrorActionPreference = 'Stop'

function Get-PeLinkerVersion([string] $Path) {
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    $peOffset = [System.BitConverter]::ToInt32($bytes, 0x3C)
    # PE signature (4 bytes) + COFF file header (20 bytes), then the optional
    # header starts with Magic (2 bytes) followed by the linker version bytes.
    $optionalHeader = $peOffset + 4 + 20
    [version]::new($bytes[$optionalHeader + 2], $bytes[$optionalHeader + 3])
}

# The loader compares an extension against the core DLL the running PHP actually
# uses, which differs by thread safety: php8ts.dll for ZTS, php8.dll for NTS.
$phpDir = Split-Path (Get-Command php).Source
$zts = [bool]((& php -v) -match 'ZTS')
$corePattern = if ($zts) { '^php\d+ts\.dll$' } else { '^php\d+\.dll$' }
$coreDll = Get-ChildItem "$phpDir/php*.dll" |
    Where-Object Name -match $corePattern |
    Select-Object -First 1
if (-not $coreDll) {
    throw "No PHP core DLL matching '$corePattern' found in $phpDir"
}
$coreVersion = Get-PeLinkerVersion $coreDll.FullName
Write-Host "PHP core $($coreDll.Name) is linked with $coreVersion"

$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vs = & $vswhere -latest -products * -property installationPath

# Newest installed side-by-side toolset that is not newer than the core.
$toolset = Get-ChildItem "$vs\VC\Tools\MSVC" -Directory |
    ForEach-Object { [version]$_.Name } |
    Where-Object { $_.Major -eq $coreVersion.Major -and $_.Minor -le $coreVersion.Minor } |
    Sort-Object -Descending |
    Select-Object -First 1

if (-not $toolset) {
    # Install the exact toolset the core was linked with.
    $component = if ($coreVersion.Minor -le 29) {
        'Microsoft.VisualStudio.Component.VC.14.29.16.11.x86.x64'
    } else {
        "Microsoft.VisualStudio.Component.VC.14.$($coreVersion.Minor).17.$($coreVersion.Minor - 30).x86.x64"
    }
    Write-Host "No suitable toolset installed; adding $component"
    $setup = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\setup.exe"
    Start-Process $setup -Wait -ArgumentList `
        "modify --installPath `"$vs`" --add $component --quiet --norestart --nocache"
    $toolset = [version]::new($coreVersion.Major, $coreVersion.Minor)
}

$pin = "$($toolset.Major).$($toolset.Minor)"
Write-Host "Pinning MSVC toolset $pin (PHP core: $coreVersion)"

cmd /c "`"$vs\VC\Auxiliary\Build\vcvarsall.bat`" x64 -vcvars_ver=$pin >nul && set" |
    ForEach-Object {
        $name, $value = $_ -split '=', 2
        if (-not $name -or $name -match '^(GITHUB_|ACTIONS_|RUNNER_)') { return }
        if ([Environment]::GetEnvironmentVariable($name) -cne $value) {
            "$name=$value" | Add-Content $env:GITHUB_ENV
        }
    }

$toolsetDir = Get-ChildItem "$vs\VC\Tools\MSVC" -Directory |
    Where-Object Name -like "$pin.*" |
    Sort-Object Name -Descending |
    Select-Object -First 1
$linker = "$($toolsetDir.FullName)\bin\Hostx64\x64\link.exe"
if (-not (Test-Path $linker)) {
    throw "Pinned linker not found at $linker"
}
Write-Host "Using linker $linker"
"CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=$linker" | Add-Content $env:GITHUB_ENV
