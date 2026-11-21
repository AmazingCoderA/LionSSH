$ErrorActionPreference = "Stop"
$Repo = "OWNER/LionSSH"
$Bin = "lionssh"
$Version = if ($args.Length -gt 0) { $args[0] } else { "latest" }
$Target = "x86_64-pc-windows-msvc"
if ($Version -eq "latest") {
  $Url = "https://github.com/$Repo/releases/latest/download/$Bin-$Target.zip"
} else {
  $Url = "https://github.com/$Repo/releases/download/$Version/$Bin-$Version-$Target.zip"
}
$Tmp = Join-Path $env:TEMP ("lionssh-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null
$Zip = Join-Path $Tmp "pkg.zip"
Write-Host "Downloading $Url"
Invoke-WebRequest -Uri $Url -OutFile $Zip
Expand-Archive -Path $Zip -DestinationPath $Tmp -Force
$Dir = Get-ChildItem $Tmp -Directory | Where-Object { $_.Name -like "$Bin-*" } | Select-Object -First 1
$Dest = Join-Path $env:USERPROFILE ".lionssh\bin"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null
Copy-Item (Join-Path $Dir.FullName "$Bin.exe") (Join-Path $Dest "$Bin.exe") -Force
Write-Host "Installed to $Dest\$Bin.exe"
Write-Host "Add to PATH if needed: $Dest"
