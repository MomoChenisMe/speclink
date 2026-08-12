# Speclink CLI 安裝腳本（Windows；cli-distribution spec「安裝腳本一行安裝對應平台
# CLI」，design D3）。行為契約與同目錄的 install.sh 一致：偵測平台、取得對應的
# GitHub Release 壓縮檔、驗證 SHA-256 後安裝。
#
# 用法：
#   irm https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.ps1 | iex
#   pwsh -File scripts/install.ps1 -DryRun
#
# 環境變數：
#   SPECLINK_INSTALL_VERSION  釘選版本（如 v0.1.0）；未設定時查詢最新 Release
#   SPECLINK_INSTALL_DIR      安裝目錄；預設 %LOCALAPPDATA%\Speclink\bin
#   SPECLINK_INSTALL_REPO     來源 repo；預設 MomoChenisMe/speclink
#
# 一律先驗 checksum 再落檔：驗證在暫存目錄完成，不符即中止，安裝目錄不會出現半份
# 或損毀的 binary。

[CmdletBinding()]
param(
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

function Get-EnvOrDefault {
    param([string]$Name, [string]$Default)
    $value = [Environment]::GetEnvironmentVariable($Name)
    if ([string]::IsNullOrWhiteSpace($value)) { return $Default }
    return $value
}

$Repo = Get-EnvOrDefault 'SPECLINK_INSTALL_REPO' 'MomoChenisMe/speclink'
$BinName = 'speclink'

# 使用者層級目錄，不需要系統管理員權限；LOCALAPPDATA 缺席時退回使用者家目錄。
$defaultDir = if ($env:LOCALAPPDATA) {
    Join-Path $env:LOCALAPPDATA 'Speclink\bin'
} else {
    Join-Path $HOME 'Speclink\bin'
}
$InstallDir = Get-EnvOrDefault 'SPECLINK_INSTALL_DIR' $defaultDir

# --- 平台偵測 ---
#
# release 管線只建置 Windows x64；其他架構明說不支援，而非裝下去才爆。
$arch = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture
switch ($arch) {
    'X64' { $Target = 'x86_64-pc-windows-msvc' }
    default {
        Write-Error "Windows 上不支援的架構：$arch（release 目前只建置 x64）"
        exit 1
    }
}

$Version = Get-EnvOrDefault 'SPECLINK_INSTALL_VERSION' ''

# --- dry-run ---
#
# 不連網、不寫檔；未釘選版本時只標示 latest 並在網址保留佔位，不去查 API。
if ($DryRun) {
    if ([string]::IsNullOrWhiteSpace($Version)) {
        $versionLabel = 'latest（安裝時查詢 GitHub API）'
        $versionInUrl = '<version>'
    } else {
        $versionLabel = $Version
        $versionInUrl = $Version
    }
    $asset = "$BinName-$versionInUrl-$Target.zip"
    Write-Output "平台：      $Target"
    Write-Output "版本：      $versionLabel"
    Write-Output "資產：      $asset"
    Write-Output "下載網址：  https://github.com/$Repo/releases/download/$versionInUrl/$asset"
    Write-Output "安裝目錄：  $InstallDir"
    exit 0
}

if ([string]::IsNullOrWhiteSpace($Version)) {
    $api = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $release = Invoke-RestMethod -Uri $api -Headers @{ 'User-Agent' = 'speclink-install' }
    } catch {
        Write-Error "查詢最新 Release 失敗：$api"
        exit 1
    }
    $Version = $release.tag_name
    if ([string]::IsNullOrWhiteSpace($Version)) {
        Write-Error "無法從 $api 的回應解析出 tag_name"
        exit 1
    }
}

$Asset = "$BinName-$Version-$Target.zip"
$Base = "https://github.com/$Repo/releases/download/$Version"

# --- 下載與驗證 ---
#
# 全程在暫存目錄進行；finally 涵蓋正常結束與中止，不留殘檔。
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("speclink-install-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp -Force | Out-Null

try {
    Write-Output "下載 $Asset（$Version）…"
    $archive = Join-Path $tmp $Asset
    $sums = Join-Path $tmp 'SHA256SUMS.txt'
    Invoke-WebRequest -Uri "$Base/$Asset" -OutFile $archive -UseBasicParsing
    Invoke-WebRequest -Uri "$Base/SHA256SUMS.txt" -OutFile $sums -UseBasicParsing

    $expected = $null
    foreach ($line in Get-Content $sums) {
        $parts = $line -split '\s+', 2
        if ($parts.Count -eq 2 -and $parts[1].Trim() -eq $Asset) {
            $expected = $parts[0].Trim()
            break
        }
    }
    if (-not $expected) {
        Write-Error "SHA256SUMS.txt 中找不到 $Asset 的條目"
        exit 1
    }

    $actual = (Get-FileHash -Path $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected.ToLowerInvariant()) {
        Write-Error "checksum 不符（預期 $expected，實得 $actual），已中止安裝"
        exit 1
    }

    # --- 安裝 ---
    #
    # 驗證通過才碰安裝目錄——失敗路徑不會在此留下任何檔案。
    $extract = Join-Path $tmp 'extract'
    Expand-Archive -Path $archive -DestinationPath $extract -Force
    $source = Join-Path $extract "$BinName.exe"
    if (-not (Test-Path $source)) {
        Write-Error "壓縮檔中找不到 $BinName.exe"
        exit 1
    }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -Path $source -Destination (Join-Path $InstallDir "$BinName.exe") -Force

    Write-Output "已安裝 $BinName $Version 至 $InstallDir\$BinName.exe"

    $pathEntries = ($env:PATH -split ';') | ForEach-Object { $_.TrimEnd('\') }
    if ($pathEntries -notcontains $InstallDir.TrimEnd('\')) {
        Write-Output "提醒：$InstallDir 不在 PATH 中，請將它加入使用者環境變數後重開終端機"
    }
} finally {
    Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
