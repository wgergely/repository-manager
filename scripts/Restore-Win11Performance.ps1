# ============================================================
# WIN11 PERFORMANCE RESTORATION SCRIPT
# Version 3.0 - Comprehensive reversal of battery optimization
# Run as Administrator - Paste directly into elevated PowerShell
# Restores all services, processes, power settings, and tasks
# ============================================================

#Requires -RunAsAdministrator

# --- ADMIN CHECK ---
if (-NOT ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Host "ERROR: This script requires Administrator privileges!" -ForegroundColor Red
    Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
    pause
    exit 1
}

# --- CONFIGURATION ---
$ServiceStartTimeoutSec = 10    # Max seconds to wait per service batch
$MaxRetries = 2                 # Retry attempts for stubborn services

Write-Host "=== WIN11 PERFORMANCE RESTORATION v3.0 ===" -ForegroundColor Cyan
Write-Host "Reversing all battery optimization changes" -ForegroundColor Yellow
Write-Host ""

$totalTime = [System.Diagnostics.Stopwatch]::StartNew()

# ============================================================
# HELPER: START A SERVICE WITH RETRIES
# ============================================================

function Start-ServiceRobust {
    param(
        [string]$ServiceName,
        [string]$StartupType = $null,
        [int]$MaxRetries = 2
    )

    $result = @{
        Name = $ServiceName
        Status = "Unknown"
    }

    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if (-not $service) {
        $result.Status = "NotFound"
        return $result
    }

    # Restore startup type if specified
    if ($StartupType) {
        try {
            Set-Service -Name $ServiceName -StartupType $StartupType -ErrorAction SilentlyContinue
        } catch { }
    }

    # Already running?
    if ($service.Status -eq 'Running') {
        $result.Status = "AlreadyRunning"
        return $result
    }

    # If the service is disabled, set it to Manual so we can start it
    $svcWmi = Get-CimInstance -ClassName Win32_Service -Filter "Name='$ServiceName'" -ErrorAction SilentlyContinue
    if ($svcWmi -and $svcWmi.StartMode -eq 'Disabled') {
        if (-not $StartupType) {
            # Default to Manual if no explicit type given and currently disabled
            Set-Service -Name $ServiceName -StartupType Manual -ErrorAction SilentlyContinue
        }
    }

    for ($attempt = 0; $attempt -le $MaxRetries; $attempt++) {
        try {
            Start-Service -Name $ServiceName -ErrorAction Stop
            Start-Sleep -Milliseconds 300
            $service.Refresh()
            if ($service.Status -eq 'Running') {
                $result.Status = "Started"
                return $result
            }
        } catch {
            # Some services depend on others; brief pause before retry
            Start-Sleep -Milliseconds 500
        }
    }

    # Final check
    $service.Refresh()
    if ($service.Status -eq 'Running') {
        $result.Status = "Started"
    } else {
        $result.Status = "Failed"
    }

    return $result
}

# ============================================================
# HELPER: BATCH START SERVICES
# ============================================================

function Start-ServicesBatch {
    param(
        [array]$ServiceDefs,  # Array of @{Name=...; StartupType=...}
        [string]$CategoryName
    )

    Write-Host "  Restoring $CategoryName..." -ForegroundColor Gray

    $started = 0
    $alreadyRunning = 0
    $notFound = 0
    $failed = @()

    foreach ($def in $ServiceDefs) {
        $name = $def.Name
        $startupType = $def.StartupType

        $r = Start-ServiceRobust -ServiceName $name -StartupType $startupType -MaxRetries $MaxRetries
        switch ($r.Status) {
            "Started"        { $started++ }
            "AlreadyRunning" { $alreadyRunning++ }
            "NotFound"       { $notFound++ }
            "Failed"         { $failed += $name }
        }
    }

    $msg = "    Started: $started | Already running: $alreadyRunning | Not found: $notFound"
    Write-Host $msg -ForegroundColor DarkGray
    if ($failed.Count -gt 0) {
        Write-Host "    Failed ($($failed.Count)): $($failed -join ', ')" -ForegroundColor DarkYellow
    }

    return @{
        Started = $started
        AlreadyRunning = $alreadyRunning
        NotFound = $notFound
        Failed = $failed
    }
}

# ============================================================
# SERVICE DEFINITIONS WITH DEFAULT STARTUP TYPES
# ============================================================
# StartupType values: Automatic, Manual, Disabled
# These reflect standard Windows 11 defaults.

$updateServices = @(
    @{Name="BITS";            StartupType="Automatic"},
    @{Name="wuauserv";        StartupType="Manual"},
    @{Name="UsoSvc";          StartupType="Automatic"},
    @{Name="WaaSMedicSvc";    StartupType="Manual"},
    @{Name="DoSvc";           StartupType="Automatic"},
    @{Name="InstallService";  StartupType="Manual"}
)

$telemetryServices = @(
    @{Name="DiagTrack";                                    StartupType="Automatic"},
    @{Name="dmwappushservice";                             StartupType="Manual"},
    @{Name="diagnosticshub.standardcollector.service";     StartupType="Manual"},
    @{Name="DPS";                                          StartupType="Automatic"},
    @{Name="WdiServiceHost";                               StartupType="Manual"},
    @{Name="WdiSystemHost";                                StartupType="Manual"},
    @{Name="WerSvc";                                       StartupType="Manual"},
    @{Name="PcaSvc";                                       StartupType="Automatic"},
    @{Name="SSDPSRV";                                      StartupType="Manual"},
    @{Name="lfsvc";                                        StartupType="Manual"},
    @{Name="MapsBroker";                                   StartupType="Automatic"},
    @{Name="SysMain";                                      StartupType="Automatic"},
    @{Name="WSearch";                                      StartupType="Automatic"}
)

$systemServices = @(
    @{Name="Spooler";                     StartupType="Automatic"},
    @{Name="Fax";                         StartupType="Manual"},
    @{Name="PrintNotify";                 StartupType="Manual"},
    @{Name="TabletInputService";          StartupType="Manual"},
    @{Name="WbioSrvc";                    StartupType="Manual"},
    @{Name="wisvc";                       StartupType="Manual"},
    @{Name="RetailDemo";                  StartupType="Manual"},
    @{Name="RemoteRegistry";              StartupType="Disabled"},
    @{Name="RemoteAccess";                StartupType="Disabled"},
    @{Name="SCardSvr";                    StartupType="Manual"},
    @{Name="ScDeviceEnum";                StartupType="Manual"},
    @{Name="SCPolicySvc";                 StartupType="Manual"},
    @{Name="seclogon";                    StartupType="Manual"},
    @{Name="ShellHWDetection";            StartupType="Automatic"},
    @{Name="TrkWks";                      StartupType="Automatic"},
    @{Name="wercplsupport";              StartupType="Manual"},
    @{Name="WMPNetworkSvc";              StartupType="Manual"},
    @{Name="icssvc";                      StartupType="Manual"},
    @{Name="PhoneSvc";                    StartupType="Manual"},
    @{Name="SEMgrSvc";                    StartupType="Manual"},
    @{Name="WpcMonSvc";                   StartupType="Manual"},
    @{Name="XblAuthManager";              StartupType="Manual"},
    @{Name="XblGameSave";                 StartupType="Manual"},
    @{Name="XboxGipSvc";                  StartupType="Manual"},
    @{Name="XboxNetApiSvc";               StartupType="Manual"},
    @{Name="GameInputSvc";                StartupType="Manual"},
    @{Name="WpnService";                  StartupType="Automatic"},
    @{Name="Themes";                      StartupType="Automatic"},
    @{Name="FrameServer";                 StartupType="Manual"},
    @{Name="FrameServerMonitor";          StartupType="Manual"},
    @{Name="stisvc";                      StartupType="Manual"},
    @{Name="GraphicsPerfSvc";             StartupType="Manual"},
    @{Name="DisplayEnhancementService";   StartupType="Manual"},
    @{Name="TimeBrokerSvc";               StartupType="Manual"},
    @{Name="SgrmBroker";                  StartupType="Automatic"},
    @{Name="spectrum";                    StartupType="Manual"},
    @{Name="perceptionsimulation";        StartupType="Manual"},
    @{Name="HvHost";                      StartupType="Manual"},
    @{Name="vmcompute";                   StartupType="Manual"},
    @{Name="vmicguestinterface";          StartupType="Manual"},
    @{Name="vmicheartbeat";               StartupType="Manual"},
    @{Name="vmickvpexchange";             StartupType="Manual"},
    @{Name="vmicrdv";                     StartupType="Manual"},
    @{Name="vmicshutdown";                StartupType="Manual"},
    @{Name="vmictimesync";                StartupType="Manual"},
    @{Name="vmicvmsession";               StartupType="Manual"},
    @{Name="vmicvss";                     StartupType="Manual"},
    @{Name="edgeupdate";                  StartupType="Automatic"},
    @{Name="edgeupdatem";                 StartupType="Manual"},
    @{Name="MicrosoftEdgeElevationService"; StartupType="Manual"},
    @{Name="uhssvc";                      StartupType="Manual"}
)

$btServices = @(
    @{Name="bthserv";       StartupType="Manual"},
    @{Name="BthAvctpSvc";   StartupType="Manual"},
    @{Name="BTAGService";    StartupType="Manual"}
)

$perUserServicePatterns = @(
    "BcastDVRUserService_*", "CDPUserSvc_*", "DevicesFlowUserSvc_*",
    "MessagingService_*", "PimIndexMaintenance_*", "UnistoreSvc_*",
    "UserDataSvc_*", "OneSyncSvc_*", "WpnUserService_*", "cbdhsvc_*"
)

# ============================================================
# PHASE 1: RESTORE POWER SETTINGS
# ============================================================

Write-Host "Restoring power settings..." -ForegroundColor Green

# Switch to Balanced power plan (GUID is the same on all Windows installs)
$balancedGuid = "381b4222-f694-41f0-9685-ff5bb260df2e"
$null = powercfg /setactive $balancedGuid 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "  Balanced power plan activated" -ForegroundColor Gray
} else {
    # Fallback: list plans and try to find Balanced
    Write-Host "  Balanced plan GUID not found, attempting lookup..." -ForegroundColor Yellow
    $plans = powercfg /list 2>$null
    $balancedLine = $plans | Select-String -Pattern "Balanced" | Select-Object -First 1
    if ($balancedLine -and $balancedLine -match '([0-9a-fA-F\-]{36})') {
        $foundGuid = $Matches[1]
        $null = powercfg /setactive $foundGuid 2>$null
        Write-Host "  Balanced plan activated ($foundGuid)" -ForegroundColor Gray
    } else {
        Write-Host "  Could not find Balanced plan; using current plan" -ForegroundColor Yellow
    }
}

# Restore CPU to 100% on both AC and DC
$null = powercfg /setacvalueindex scheme_current sub_processor PROCTHROTTLEMAX 100 2>$null
$null = powercfg /setdcvalueindex scheme_current sub_processor PROCTHROTTLEMAX 100 2>$null
$null = powercfg /setactive scheme_current 2>$null
Write-Host "  CPU max restored (AC:100%, DC:100%)" -ForegroundColor Gray

# Also restore the Balanced plan's processor values directly
$null = powercfg /setacvalueindex $balancedGuid sub_processor PROCTHROTTLEMAX 100 2>$null
$null = powercfg /setdcvalueindex $balancedGuid sub_processor PROCTHROTTLEMAX 100 2>$null

Write-Host ""

# ============================================================
# PHASE 2: RESTORE ALL SERVICES
# ============================================================

Write-Host "Restoring services..." -ForegroundColor Green

# Start services in dependency order: core system first, then higher-level

# 2a. Critical networking and system event services (dependencies for others)
$criticalFirst = @(
    @{Name="SENS";                   StartupType="Automatic"},
    @{Name="Dhcp";                   StartupType="Automatic"},
    @{Name="Dnscache";               StartupType="Automatic"},
    @{Name="NlaSvc";                 StartupType="Automatic"},
    @{Name="Wlansvc";                StartupType="Automatic"},
    @{Name="Wcmsvc";                 StartupType="Automatic"},
    @{Name="Audiosrv";               StartupType="Automatic"},
    @{Name="AudioEndpointBuilder";   StartupType="Automatic"}
)
Start-ServicesBatch -ServiceDefs $criticalFirst -CategoryName "Critical services (network, audio)"

# 2b. Update services
Start-ServicesBatch -ServiceDefs $updateServices -CategoryName "Windows Update services"

# 2c. Telemetry and diagnostics
Start-ServicesBatch -ServiceDefs $telemetryServices -CategoryName "Telemetry & diagnostics services"

# 2d. System services
Start-ServicesBatch -ServiceDefs $systemServices -CategoryName "System services"

# 2e. Bluetooth
Start-ServicesBatch -ServiceDefs $btServices -CategoryName "Bluetooth services"

# 2f. Per-user services
$perUserDefs = @()
foreach ($pattern in $perUserServicePatterns) {
    $found = Get-Service -Name $pattern -ErrorAction SilentlyContinue
    if ($found) {
        foreach ($svc in $found) {
            $perUserDefs += @{Name=$svc.Name; StartupType=$null}
        }
    }
}
if ($perUserDefs.Count -gt 0) {
    Start-ServicesBatch -ServiceDefs $perUserDefs -CategoryName "Per-user services"
}

Write-Host ""

# ============================================================
# PHASE 3: RE-ENABLE SCHEDULED TASKS
# ============================================================

Write-Host "Re-enabling scheduled tasks..." -ForegroundColor Green

$taskPaths = @(
    "\Microsoft\Windows\UpdateOrchestrator\",
    "\Microsoft\Windows\WindowsUpdate\",
    "\Microsoft\Windows\Application Experience\",
    "\Microsoft\Windows\Customer Experience Improvement Program\",
    "\Microsoft\Windows\Diagnosis\",
    "\Microsoft\Windows\Maintenance\",
    "\Microsoft\Windows\Windows Error Reporting\"
)

$tasksEnabled = 0
$tasksAlreadyReady = 0
foreach ($taskPath in $taskPaths) {
    $tasks = Get-ScheduledTask -TaskPath $taskPath -ErrorAction SilentlyContinue
    if ($tasks) {
        foreach ($task in $tasks) {
            if ($task.State -eq 'Disabled') {
                Enable-ScheduledTask -TaskName $task.TaskName -TaskPath $task.TaskPath -ErrorAction SilentlyContinue | Out-Null
                $tasksEnabled++
            } else {
                $tasksAlreadyReady++
            }
        }
    }
}
Write-Host "  Enabled: $tasksEnabled | Already active: $tasksAlreadyReady" -ForegroundColor Gray

Write-Host ""

# ============================================================
# PHASE 4: RESTART EXPLORER AND SHELL PROCESSES
# ============================================================

Write-Host "Restarting Explorer and shell..." -ForegroundColor Green

# Start Explorer if not running
$explorerProc = Get-Process -Name "explorer" -ErrorAction SilentlyContinue
if (-not $explorerProc) {
    Start-Process "explorer.exe"
    Write-Host "  Explorer started" -ForegroundColor Gray
} else {
    Write-Host "  Explorer already running" -ForegroundColor Gray
}

# Give Explorer a moment to initialize the shell
Start-Sleep -Seconds 2

# ctfmon (text input framework) - needed for keyboard input in some apps
$ctfmon = Get-Process -Name "ctfmon" -ErrorAction SilentlyContinue
if (-not $ctfmon) {
    $ctfmonPath = "$env:SystemRoot\System32\ctfmon.exe"
    if (Test-Path $ctfmonPath) {
        Start-Process $ctfmonPath -ErrorAction SilentlyContinue
        Write-Host "  ctfmon (text input) started" -ForegroundColor Gray
    }
}

# SecurityHealthSystray (Windows Security icon)
$secHealthPath = "$env:ProgramFiles\Windows Defender\MSASCuiL.exe"
if (-not $secHealthPath) {
    $secHealthPath = "${env:ProgramFiles(x86)}\Windows Defender\MSASCuiL.exe"
}
if (Test-Path $secHealthPath -ErrorAction SilentlyContinue) {
    $secProc = Get-Process -Name "SecurityHealthSystray" -ErrorAction SilentlyContinue
    if (-not $secProc) {
        Start-Process $secHealthPath -ErrorAction SilentlyContinue
        Write-Host "  Windows Security systray started" -ForegroundColor Gray
    }
}

# Smartscreen (background process, will be restarted by Explorer on demand)
# No manual action needed - Explorer triggers it.

Write-Host ""

# ============================================================
# PHASE 5: VERIFICATION
# ============================================================

Write-Host "Verifying restoration..." -ForegroundColor Cyan

$verifyServices = @(
    @{Name="Wlansvc";              Desc="WiFi"},
    @{Name="Dhcp";                 Desc="DHCP"},
    @{Name="Dnscache";             Desc="DNS"},
    @{Name="NlaSvc";               Desc="Network Location"},
    @{Name="SENS";                 Desc="System Events"},
    @{Name="Wcmsvc";               Desc="Connection Manager"},
    @{Name="Audiosrv";             Desc="Audio"},
    @{Name="AudioEndpointBuilder"; Desc="Audio Endpoint"},
    @{Name="BITS";                 Desc="Background Transfer (BITS)"},
    @{Name="wuauserv";             Desc="Windows Update"},
    @{Name="UsoSvc";               Desc="Update Orchestrator"},
    @{Name="SysMain";              Desc="Superfetch/SysMain"},
    @{Name="WSearch";              Desc="Windows Search"},
    @{Name="Themes";               Desc="Themes"},
    @{Name="WpnService";           Desc="Push Notifications"},
    @{Name="ShellHWDetection";     Desc="Shell HW Detection"},
    @{Name="Spooler";              Desc="Print Spooler"},
    @{Name="DPS";                  Desc="Diagnostic Policy"},
    @{Name="DiagTrack";            Desc="Diagnostics Tracking"},
    @{Name="TrkWks";               Desc="Distributed Link Tracking"},
    @{Name="PcaSvc";               Desc="Program Compatibility"}
)

$okCount = 0
$failCount = 0

foreach ($svc in $verifyServices) {
    $service = Get-Service -Name $svc.Name -ErrorAction SilentlyContinue
    if ($service) {
        if ($service.Status -eq 'Running') {
            Write-Host "  [OK]   $($svc.Desc)" -ForegroundColor Green
            $okCount++
        } else {
            # Some Manual-start services won't be running and that's correct
            $wmi = Get-CimInstance -ClassName Win32_Service -Filter "Name='$($svc.Name)'" -ErrorAction SilentlyContinue
            if ($wmi -and $wmi.StartMode -eq 'Manual') {
                Write-Host "  [IDLE] $($svc.Desc) (Manual start - normal)" -ForegroundColor DarkGray
                $okCount++
            } else {
                Write-Host "  [WARN] $($svc.Desc) ($($service.Status))" -ForegroundColor Yellow
                $failCount++
            }
        }
    } else {
        Write-Host "  [--]   $($svc.Desc) (not installed)" -ForegroundColor DarkGray
    }
}

# Check Explorer
$explorerCheck = Get-Process -Name "explorer" -ErrorAction SilentlyContinue
if ($explorerCheck) {
    Write-Host "  [OK]   Explorer shell" -ForegroundColor Green
    $okCount++
} else {
    Write-Host "  [FAIL] Explorer shell" -ForegroundColor Red
    $failCount++
}

# Check power plan
Write-Host ""
Write-Host "  Power plan:" -ForegroundColor Cyan
$activePlan = powercfg /getactivescheme 2>$null
if ($activePlan) {
    Write-Host "  $activePlan" -ForegroundColor Gray
}
$cpuAC = powercfg /query scheme_current sub_processor PROCTHROTTLEMAX 2>$null
$acMax = ($cpuAC | Select-String "Current AC Power Setting Index" | ForEach-Object { $_ -replace '.*:\s*', '' })
$dcMax = ($cpuAC | Select-String "Current DC Power Setting Index" | ForEach-Object { $_ -replace '.*:\s*', '' })
if ($acMax) {
    $acPct = [Convert]::ToInt32($acMax, 16)
    Write-Host "  CPU max (AC): ${acPct}%" -ForegroundColor $(if ($acPct -eq 100) { "Green" } else { "Yellow" })
}
if ($dcMax) {
    $dcPct = [Convert]::ToInt32($dcMax, 16)
    Write-Host "  CPU max (DC): ${dcPct}%" -ForegroundColor $(if ($dcPct -eq 100) { "Green" } else { "Yellow" })
}

# Parsec check
$parsecSvc = Get-Service -Name "*parsec*" -ErrorAction SilentlyContinue
$parsecProc = Get-Process -Name "*parsec*" -ErrorAction SilentlyContinue
if ($parsecSvc -and $parsecSvc.Status -eq 'Running') {
    Write-Host "  [OK]   Parsec (service)" -ForegroundColor Green
} elseif ($parsecProc) {
    Write-Host "  [OK]   Parsec (process)" -ForegroundColor Green
} else {
    Write-Host "  [--]   Parsec (not detected)" -ForegroundColor DarkGray
}

# ============================================================
# SUMMARY
# ============================================================

$totalTime.Stop()

Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
if ($failCount -eq 0) {
    Write-Host "RESTORATION COMPLETE - ALL CLEAR" -ForegroundColor Green
} else {
    Write-Host "RESTORATION COMPLETE - $failCount WARNINGS" -ForegroundColor Yellow
}
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Time: $([math]::Round($totalTime.Elapsed.TotalSeconds, 1))s" -ForegroundColor DarkGray
Write-Host "  Services OK: $okCount | Warnings: $failCount" -ForegroundColor DarkGray
Write-Host ""
if ($failCount -gt 0) {
    Write-Host "  Some services may require a RESTART to fully restore." -ForegroundColor Yellow
    Write-Host "  Services with 'Manual' start type will start on-demand." -ForegroundColor DarkGray
} else {
    Write-Host "  All services and settings restored to Windows defaults." -ForegroundColor Green
}
Write-Host ""
