#Requires -Version 5.1
<#
.SYNOPSIS
    RevGame Engine Installer for Windows

.DESCRIPTION
    Installs dependencies and builds the Bevy game client for Windows.

.EXAMPLE
    .\install.ps1
#>

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

# State
$script:MissingDeps = @()
$script:BuildType = "graphics"

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-Err {
    param([string]$Message)
    Write-Host "[ERROR] " -ForegroundColor Red -NoNewline
    Write-Host $Message
}

function Write-Step {
    param([string]$Step, [string]$Message)
    Write-Host ""
    Write-Host "[$Step] " -ForegroundColor Cyan -NoNewline
    Write-Host $Message -ForegroundColor White
}

function Read-YesNo {
    param(
        [string]$Prompt,
        [bool]$Default = $true
    )

    $suffix = if ($Default) { "[Y/n]" } else { "[y/N]" }
    $response = Read-Host "$Prompt $suffix"

    if ([string]::IsNullOrWhiteSpace($response)) {
        return $Default
    }

    return $response -match '^[Yy]'
}

function Read-Choice {
    param(
        [string]$Prompt,
        [int]$Default,
        [string[]]$Options
    )

    Write-Host ""
    for ($i = 0; $i -lt $Options.Length; $i++) {
        Write-Host "  [$($i + 1)] $($Options[$i])"
    }
    Write-Host ""

    while ($true) {
        $response = Read-Host "$Prompt [$Default]"
        if ([string]::IsNullOrWhiteSpace($response)) {
            return $Default
        }

        $choice = 0
        if ([int]::TryParse($response, [ref]$choice)) {
            if ($choice -ge 1 -and $choice -le $Options.Length) {
                return $choice
            }
        }

        Write-Host "Please enter a number between 1 and $($Options.Length)"
    }
}

function Show-Banner {
    Write-Host ""
    Write-Host "================================================================================" -ForegroundColor White
    Write-Host "                       RevGame Engine Installer" -ForegroundColor White
    Write-Host "================================================================================" -ForegroundColor White
    Write-Host ""
    Write-Host "This script will install dependencies and build RevGame for Windows."
    Write-Host ""
}

function Test-Command {
    param(
        [string]$Name,
        [string]$Command
    )

    Write-Host "  ${Name}: " -NoNewline

    try {
        $result = & $Command --version 2>&1 | Select-Object -First 1
        Write-Host "FOUND " -ForegroundColor Green -NoNewline
        Write-Host "($result)"
        return $true
    }
    catch {
        Write-Host "NOT FOUND" -ForegroundColor Red
        return $false
    }
}

function Test-Dependencies {
    Write-Step "1/4" "Checking dependencies..."

    $script:MissingDeps = @()

    if (-not (Test-Command "Git" "git")) {
        $script:MissingDeps += "git"
    }

    if (-not (Test-Command "Rust" "rustc")) {
        $script:MissingDeps += "rust"
    }

    if (-not (Test-Command "Cargo" "cargo")) {
        if ("rust" -notin $script:MissingDeps) {
            $script:MissingDeps += "rust"
        }
    }

    # Check for Visual Studio Build Tools (required for Rust on Windows)
    Write-Host ""
    Write-Host "  Visual Studio Build Tools:"

    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($vsPath) {
            Write-Host "    C++ Build Tools: " -NoNewline
            Write-Host "FOUND" -ForegroundColor Green
        } else {
            Write-Host "    C++ Build Tools: " -NoNewline
            Write-Host "NOT FOUND" -ForegroundColor Red
            $script:MissingDeps += "vs-buildtools"
        }
    } else {
        # vswhere not found, check if cl.exe is in PATH
        try {
            $null = Get-Command cl -ErrorAction Stop
            Write-Host "    C++ Build Tools: " -NoNewline
            Write-Host "FOUND" -ForegroundColor Green
        }
        catch {
            Write-Host "    C++ Build Tools: " -NoNewline
            Write-Host "NOT FOUND" -ForegroundColor Red
            $script:MissingDeps += "vs-buildtools"
        }
    }

    Write-Host ""
    if ($script:MissingDeps.Count -eq 0) {
        Write-Info "All dependencies are installed!"
        return $true
    } else {
        Write-Warn "Missing dependencies: $($script:MissingDeps -join ', ')"
        return $false
    }
}

function Get-PackageManager {
    # Check for winget
    try {
        $null = Get-Command winget -ErrorAction Stop
        return "winget"
    }
    catch {}

    # Check for chocolatey
    try {
        $null = Get-Command choco -ErrorAction Stop
        return "choco"
    }
    catch {}

    return $null
}

function Install-Rust {
    Write-Info "Installing Rust via rustup..."

    $rustupInit = Join-Path $env:TEMP "rustup-init.exe"

    # Download rustup-init.exe
    Write-Info "Downloading rustup-init.exe..."
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit -UseBasicParsing

    # Run rustup-init
    Write-Info "Running rustup-init..."
    & $rustupInit -y

    # Refresh PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" +
                [System.Environment]::GetEnvironmentVariable("Path", "User")

    # Also add cargo bin directly
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoPath) {
        $env:Path = "$cargoPath;$env:Path"
    }

    # Verify installation
    try {
        $version = & rustc --version
        Write-Info "Rust installed successfully: $version"
    }
    catch {
        Write-Err "Rust installation failed. Please restart your terminal and try again."
        exit 1
    }

    # Clean up
    Remove-Item $rustupInit -ErrorAction SilentlyContinue
}

function Install-Git {
    $pkgManager = Get-PackageManager

    Write-Info "Installing Git..."

    switch ($pkgManager) {
        "winget" {
            winget install --id Git.Git -e --accept-source-agreements --accept-package-agreements
        }
        "choco" {
            choco install git -y
        }
        default {
            Write-Err "No package manager found. Please install Git manually from https://git-scm.com/download/win"
            exit 1
        }
    }

    # Refresh PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" +
                [System.Environment]::GetEnvironmentVariable("Path", "User")
}

function Install-VsBuildTools {
    Write-Info "Visual Studio Build Tools are required for Rust compilation."
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  1. Install Visual Studio Build Tools (recommended)"
    Write-Host "  2. Install Visual Studio Community (full IDE)"
    Write-Host ""
    Write-Host "You can download from: https://visualstudio.microsoft.com/downloads/"
    Write-Host "Select 'Desktop development with C++' workload during installation."
    Write-Host ""

    $pkgManager = Get-PackageManager

    if ($pkgManager -eq "winget") {
        if (Read-YesNo "Attempt to install via winget?" $true) {
            Write-Info "Installing Visual Studio Build Tools via winget..."
            winget install --id Microsoft.VisualStudio.2022.BuildTools -e --accept-source-agreements --accept-package-agreements --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

            Write-Info "Build Tools installation started. This may take a while..."
            Write-Warn "You may need to restart your terminal after installation completes."
        } else {
            Write-Warn "Please install Visual Studio Build Tools manually and re-run this script."
            exit 0
        }
    } elseif ($pkgManager -eq "choco") {
        if (Read-YesNo "Attempt to install via chocolatey?" $true) {
            Write-Info "Installing Visual Studio Build Tools via chocolatey..."
            choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended" -y
        } else {
            Write-Warn "Please install Visual Studio Build Tools manually and re-run this script."
            exit 0
        }
    } else {
        Write-Warn "Please install Visual Studio Build Tools manually and re-run this script."
        exit 0
    }
}

function Install-Dependencies {
    Write-Step "2/4" "Installing missing dependencies..."

    if ($script:MissingDeps.Count -eq 0) {
        Write-Info "No dependencies to install"
        return
    }

    Write-Host ""
    Write-Host "The following will be installed:"
    foreach ($dep in $script:MissingDeps) {
        switch ($dep) {
            "git" { Write-Host "  - Git (version control)" }
            "rust" { Write-Host "  - Rust toolchain (via rustup)" }
            "vs-buildtools" { Write-Host "  - Visual Studio Build Tools (C++ compiler)" }
        }
    }
    Write-Host ""

    if (-not (Read-YesNo "Proceed with installation?")) {
        Write-Warn "Installation cancelled"
        exit 0
    }

    Write-Host ""

    foreach ($dep in $script:MissingDeps) {
        switch ($dep) {
            "git" { Install-Git }
            "rust" { Install-Rust }
            "vs-buildtools" { Install-VsBuildTools }
        }
    }

    Write-Info "All dependencies installed!"
}

function Select-BuildType {
    Write-Step "3/4" "Build configuration"

    $choice = Read-Choice "Select build type:" 1 @(
        "Full build with graphics (Bevy) - for playing/testing"
        "Headless build - API testing only"
    )

    if ($choice -eq 1) {
        $script:BuildType = "graphics"
    } else {
        $script:BuildType = "headless"
    }

    Write-Info "Selected: $($script:BuildType) build"
}

function Build-RevGame {
    Write-Host ""
    Write-Info "Building RevGame ($($script:BuildType))..."
    Write-Host ""

    # Ensure we're in the right directory
    Set-Location $ScriptDir

    # Check Cargo.toml exists
    if (-not (Test-Path "Cargo.toml")) {
        Write-Err "Cargo.toml not found. Are you in the revgame directory?"
        exit 1
    }

    # Build
    if ($script:BuildType -eq "graphics") {
        & cargo build --release --features graphics
    } else {
        & cargo build --release
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Err "Build failed with exit code $LASTEXITCODE"
        exit 1
    }

    Write-Info "Build complete!"
}

function Test-Build {
    Write-Step "4/4" "Verification"

    Write-Host ""

    if ($script:BuildType -eq "graphics") {
        $binary = Join-Path $ScriptDir "target\release\revgame.exe"
        if (Test-Path $binary) {
            Write-Host "  Binary: " -NoNewline
            Write-Host $binary -ForegroundColor Green
            Write-Info "Verification passed!"
        } else {
            Write-Host "  Binary: " -NoNewline
            Write-Host "NOT FOUND" -ForegroundColor Red
            Write-Err "Build verification failed"
            exit 1
        }
    } else {
        $buildDir = Join-Path $ScriptDir "target\release"
        if (Test-Path $buildDir) {
            Write-Host "  Build directory: " -NoNewline
            Write-Host $buildDir -ForegroundColor Green
            Write-Info "Headless build verification passed!"
        } else {
            Write-Err "Build verification failed"
            exit 1
        }
    }
}

function Show-NextSteps {
    Write-Host ""
    Write-Host "================================================================================" -ForegroundColor White
    Write-Host "                       Installation Complete!" -ForegroundColor White
    Write-Host "================================================================================" -ForegroundColor White
    Write-Host ""

    if ($script:BuildType -eq "graphics") {
        Write-Host "Run the game:"
        Write-Host "  cd $ScriptDir"
        Write-Host "  .\target\release\revgame.exe"
        Write-Host ""
        Write-Host "Or with cargo:"
        Write-Host "  cargo run --release --features graphics"
    } else {
        Write-Host "The headless library has been built."
        Write-Host ""
        Write-Host "Run tests:"
        Write-Host "  cargo test"
    }

    Write-Host ""
    Write-Host "Configuration:"
    Write-Host "  Edit config.json to set backend URL"
    Write-Host ""
}

# Main
function Main {
    Show-Banner

    # First pass: check dependencies
    $null = Test-Dependencies

    # If Rust is missing, install it before asking about build type
    if ("rust" -in $script:MissingDeps) {
        Write-Host ""
        if (Read-YesNo "Rust is not installed. Install it now?") {
            Install-Rust
            $script:MissingDeps = $script:MissingDeps | Where-Object { $_ -ne "rust" }
        } else {
            Write-Err "Rust is required to build RevGame"
            exit 1
        }
    }

    # Ask about build type
    Select-BuildType

    # Re-check dependencies
    $script:MissingDeps = @()
    $null = Test-Dependencies

    # Install remaining dependencies
    Install-Dependencies

    # Build
    Build-RevGame

    # Verify
    Test-Build

    # Done
    Show-NextSteps
}

Main
