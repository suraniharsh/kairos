$ErrorActionPreference = "Stop"

$REPO    = "suraniharsh/kairos"
$BIN     = "kairos"
$TARGET  = "x86_64-pc-windows-msvc"

# Resolve latest version
$release = Invoke-RestMethod "https://api.github.com/repos/$REPO/releases/latest"
$VERSION = $release.tag_name

$FILENAME = "$BIN-$VERSION-$TARGET.zip"
$URL      = "https://github.com/$REPO/releases/download/$VERSION/$FILENAME"

Write-Host "Installing kairos $VERSION ($TARGET)..."

# Download to temp dir
$TMP = Join-Path $env:TEMP "kairos-install-$([System.IO.Path]::GetRandomFileName())"
New-Item -ItemType Directory -Force -Path $TMP | Out-Null

try {
    Invoke-WebRequest $URL -OutFile "$TMP\$FILENAME" -UseBasicParsing
    Expand-Archive "$TMP\$FILENAME" -DestinationPath $TMP -Force

    $EXE = Get-ChildItem $TMP -Recurse -Filter "$BIN.exe" | Select-Object -First 1
    if (-not $EXE) {
        Write-Error "Binary not found in archive"; exit 1
    }

    $INSTALL_DIR = "$env:USERPROFILE\.local\bin"
    New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
    Copy-Item $EXE.FullName "$INSTALL_DIR\$BIN.exe" -Force

    Write-Host "Installed: $INSTALL_DIR\$BIN.exe"

    # Add to user PATH if missing
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$INSTALL_DIR*") {
        [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$INSTALL_DIR", "User")
        Write-Host "Added $INSTALL_DIR to PATH (restart your terminal)"
    }
} finally {
    Remove-Item $TMP -Recurse -Force -ErrorAction SilentlyContinue
}
