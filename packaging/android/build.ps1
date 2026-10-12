# Wordpadx Android Tablet Build Script (PowerShell)
param (
    [switch]$SkipWeb = $false,
    [switch]$Release = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Resolve-Path "$ScriptDir\..\.."
$WebDir = "$RootDir\apps\wordcraft-web"
$DistWeb = "$RootDir\dist\web"
$AndroidDir = "$RootDir\mobile\android"
$AndroidAssets = "$AndroidDir\app\src\main\assets\www"

Write-Host "==> Building Wordpadx for Android Tablet..." -ForegroundColor Cyan

if (-not $SkipWeb) {
    Write-Host "--> Building WebAssembly bundle with Trunk..." -ForegroundColor Yellow
    Push-Location $WebDir
    try {
        trunk build --release
    } finally {
        Pop-Location
    }
}

Write-Host "--> Copying web bundle to Android assets..." -ForegroundColor Yellow
if (Test-Path $AndroidAssets) {
    Remove-Item -Recurse -Force $AndroidAssets
}
New-Item -ItemType Directory -Force -Path $AndroidAssets | Out-Null
Copy-Item -Recurse -Force "$DistWeb\*" $AndroidAssets

Write-Host "--> Compiling Android APK with Gradle..." -ForegroundColor Yellow
Push-Location $AndroidDir
try {
    $GradleCmd = if (Test-Path ".\gradlew.bat") { ".\gradlew.bat" } else { "gradle" }
    $BuildTask = if ($Release) { "assembleRelease" } else { "assembleDebug" }
    & $GradleCmd $BuildTask
} finally {
    Pop-Location
}

$ApkFolder = if ($Release) { "$AndroidDir\app\build\outputs\apk\release" } else { "$AndroidDir\app\build\outputs\apk\debug" }
Write-Host "==> Android Tablet build complete!" -ForegroundColor Green
Write-Host "    APKs located at: $ApkFolder"
