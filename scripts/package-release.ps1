[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [string]$OutputDir = "release"
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

if (-not $SkipBuild) {
    Write-Host "Installing frontend dependencies..."
    Push-Location (Join-Path $repoRoot "frontend")
    try {
        & npm ci
        if ($LASTEXITCODE -ne 0) {
            throw "npm ci failed with exit code $LASTEXITCODE"
        }

        Write-Host "Building frontend..."
        & npm run build
        if ($LASTEXITCODE -ne 0) {
            throw "npm run build failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }

    Write-Host "Building backend..."
    & cargo build --manifest-path backend/Cargo.toml --release --locked
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }
}

$exe = Join-Path $repoRoot "backend/target/release/listquiz.exe"
$dist = Join-Path $repoRoot "frontend/dist"
if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
    throw "Missing $exe. Run without -SkipBuild first."
}
if (-not (Test-Path -LiteralPath (Join-Path $dist "index.html") -PathType Leaf)) {
    throw "Missing $dist/index.html. Run without -SkipBuild first."
}

$outputRoot = Join-Path $repoRoot $OutputDir
$stage = Join-Path $outputRoot "ListQuiz"
if (Test-Path -LiteralPath $stage) {
    Remove-Item -LiteralPath $stage -Recurse -Force
}
New-Item -ItemType Directory -Path (Join-Path $stage "frontend") -Force | Out-Null

Copy-Item -LiteralPath $exe -Destination (Join-Path $stage "listquiz.exe")
Copy-Item -LiteralPath $dist -Destination (Join-Path $stage "frontend/dist") -Recurse
$startBat = [System.IO.File]::ReadAllText((Join-Path $repoRoot "packaging/start.bat"))
$startBat = $startBat -replace "`r?`n", "`r`n"
[System.IO.File]::WriteAllText(
    (Join-Path $stage "start.bat"),
    $startBat,
    [System.Text.Encoding]::ASCII
)
Copy-Item -LiteralPath (Join-Path $repoRoot "packaging/README.txt") -Destination $stage
Copy-Item -LiteralPath (Join-Path $repoRoot "LICENSE") -Destination $stage

$zip = Join-Path $outputRoot "listquiz-windows-x64.zip"
if (Test-Path -LiteralPath $zip) {
    Remove-Item -LiteralPath $zip -Force
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory(
    $stage,
    $zip,
    [System.IO.Compression.CompressionLevel]::Optimal,
    $true
)

$hash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash
$checksum = "$zip.sha256"
Set-Content -LiteralPath $checksum -Value "$hash  listquiz-windows-x64.zip" -Encoding Ascii

Write-Host ""
Write-Host "Portable release created:"
Write-Host "  $zip"
Write-Host "  $checksum"
