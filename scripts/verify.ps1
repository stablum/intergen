[CmdletBinding()]
param(
    [ValidateSet("format", "targeted", "full", "smoke", "all")]
    [string]$Mode = "format",

    [string]$TestFilter,

    [ValidateRange(1, 10000)]
    [int]$CaptureDelayFrames = 120
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($Mode -eq "targeted" -and [string]::IsNullOrWhiteSpace($TestFilter)) {
    throw "-TestFilter is required when -Mode targeted is used."
}

$workspace = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$targetDirectory = Join-Path $workspace ".target-verification-runs\cargo-cache"
$smokeImage = Join-Path $workspace ".target-verification-runs\smoke.png"

function Invoke-Cargo {
    param([Parameter(Mandatory)][string[]]$CargoArguments)

    Write-Host "cargo $($CargoArguments -join ' ')"
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "cargo exited with code $LASTEXITCODE"
    }
}

function Invoke-FullTests {
    Invoke-Cargo -CargoArguments @(
        "test",
        "--quiet",
        "--target-dir", $targetDirectory
    )
}

function Invoke-SmokeRun {
    Invoke-Cargo -CargoArguments @(
        "run",
        "--quiet",
        "--target-dir", $targetDirectory,
        "--",
        "--capture", $smokeImage,
        "--capture-delay-frames", $CaptureDelayFrames.ToString()
    )
}

Push-Location $workspace
try {
    Invoke-Cargo -CargoArguments @("fmt", "--", "--check")

    switch ($Mode) {
        "format" {}
        "targeted" {
            Invoke-Cargo -CargoArguments @(
                "test",
                "--quiet",
                "--target-dir", $targetDirectory,
                "--lib",
                $TestFilter
            )
        }
        "full" {
            Invoke-FullTests
        }
        "smoke" {
            Invoke-SmokeRun
        }
        "all" {
            Invoke-FullTests
            Invoke-SmokeRun
        }
    }
}
finally {
    Pop-Location
}

Write-Host "Verification mode '$Mode' completed using $targetDirectory"
