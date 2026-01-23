
# Wrapper script for launching a subagent via PowerShell
# Delegates to the Python script which handles cross-platform logic

param(
    [Parameter(Mandatory=$true)]
    [string]$skill,

    [Parameter(Mandatory=$true)]
    [string]$task,

    [switch]$NoYolo,

    [string]$OutputFormat = "text"
)

$ErrorActionPreference = "Stop"

# Resolve path to the python script
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$PythonScript = Join-Path $ScriptDir "spawn_subagent.py"

if (-not (Test-Path $PythonScript)) {
    Write-Error "Could not find spawn_subagent.py at $PythonScript"
    exit 1
}

# Build arguments
$ArgsList = @(
    $PythonScript,
    "--skill", $skill,
    "--task", $task,
    "--output-format", $OutputFormat
)

if ($NoYolo) {
    $ArgsList += "--no-yolo"
}

# Execute python
try {
    & python $ArgsList
    exit $LASTEXITCODE
} catch {
    Write-Error "Failed to execute python subagent script: $_"
    exit 1
}
