# ============================================================
# WIN11 PERFORMANCE RESTORATION SCRIPT
# Version 3.2 - Comprehensive reversal of battery optimization
# Handles stuck services, Start Menu restoration, cache rebuild, Widgets disable
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
$ServiceStartTimeoutSec = 8     # Max seconds to wait per individual service start
$StuckServiceTimeoutSec = 5     # Max seconds to wait for stuck (Pending) services
$MaxRetries = 2                 # Retry attempts for stubborn services

Write-Host "=== WIN11 PERFORMANCE RESTORATION v3.2 ===" -ForegroundColor Cyan
Write-Host "Reversing all battery optimization changes" -ForegroundColor Yellow
Write-Host ""

$totalTime = [System.Diagnostics.Stopwatch]::StartNew()

# ============================================================
# HELPER: UNSTICK A SERVICE IN PENDING STATE
# ============================================================
# Services stuck in StopPending or StartPending will block
# Start-Service indefinitely. This function detects and recovers
# from stuck states by force-killing the underlying process.

function Resolve-StuckService {
    param(
        [string]$ServiceName,
        [int]$TimeoutSeconds = 5
    )

    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if (-not $service) { return $false }

    $status = $service.Status
    $isStuck = ($status -eq 'StopPending') -or ($status -eq 'StartPending') -or ($status -eq 'ContinuePending') -or ($status -eq 'PausePending')

    if (-not $isStuck) { return $true }  # Not stuck, nothing to do

    Write-Host "    [$ServiceName] stuck in '$status' - recovering..." -ForegroundColor DarkYellow

    # Wait briefly in case it's genuinely transitioning
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt [Math]::Min($TimeoutSeconds, 3)) {
        Start-Sleep -Milliseconds 400
        $service.Refresh()
        if ($service.Status -eq 'Running' -or $service.Status -eq 'Stopped') {
            return $true
        }
    }

    # Still stuck - get the PID and force kill it
    try {
        $wmi = Get-CimInstance -ClassName Win32_Service -Filter "Name='$ServiceName'" -ErrorAction SilentlyContinue
        if ($wmi -and $wmi.ProcessId -gt 0) {
            Write-Host "    [$ServiceName] force-killing PID $($wmi.ProcessId)..." -ForegroundColor DarkYellow
            $null = taskkill /F /PID $wmi.ProcessId 2>$null
            Start-Sleep -Milliseconds 500
        }
    } catch { }

    # If still stuck, try sc.exe to reset the service state
    $service.Refresh()
    if ($service.Status -ne 'Running' -and $service.Status -ne 'Stopped') {
        # sc.exe can sometimes nudge a stuck service
        if ($status -like '*Stop*') {
            $null = sc.exe stop $ServiceName 2>$null
        }
        Start-Sleep -Milliseconds 300

        # Last resort: use WMI to kill by executable path pattern
        try {
            $wmi = Get-CimInstance -ClassName Win32_Service -Filter "Name='$ServiceName'" -ErrorAction SilentlyContinue
            if ($wmi -and $wmi.ProcessId -gt 0) {
                Stop-Process -Id $wmi.ProcessId -Force -ErrorAction SilentlyContinue
                Start-Sleep -Milliseconds 500
            }
        } catch { }
    }

    $service.Refresh()
    $resolved = ($service.Status -eq 'Running' -or $service.Status -eq 'Stopped')
    if (-not $resolved) {
        Write-Host "    [$ServiceName] could not be unstuck (state: $($service.Status)) - will skip" -ForegroundColor Red
    }
    return $resolved
}

# ============================================================
# HELPER: START A SERVICE WITH TIMEOUT + STUCK-STATE HANDLING
# ============================================================

function Start-ServiceRobust {
    param(
        [string]$ServiceName,
        [string]$StartupType = $null,
        [int]$MaxRetries = 2,
        [int]$TimeoutSeconds = 8
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

    # Restore startup type if specified (do this even if service is running/stuck)
    if ($StartupType -and $StartupType -ne 'Disabled') {
        try {
            Set-Service -Name $ServiceName -StartupType $StartupType -ErrorAction SilentlyContinue
        } catch { }
    }

    # For services whose default is Disabled, just restore the type and don't start
    if ($StartupType -eq 'Disabled') {
        try {
            Set-Service -Name $ServiceName -StartupType Disabled -ErrorAction SilentlyContinue
        } catch { }
        $result.Status = "AlreadyRunning"  # count as OK - Disabled is the correct state
        return $result
    }

    # Handle stuck Pending states BEFORE attempting start
    $service.Refresh()
    $pendingStates = @('StopPending', 'StartPending', 'ContinuePending', 'PausePending')
    if ($service.Status -in $pendingStates) {
        $unstuck = Resolve-StuckService -ServiceName $ServiceName -TimeoutSeconds $script:StuckServiceTimeoutSec
        if (-not $unstuck) {
            $result.Status = "Stuck"
            return $result
        }
        $service.Refresh()
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
            Set-Service -Name $ServiceName -StartupType Manual -ErrorAction SilentlyContinue
        }
    }

    # Attempt start with timeout protection using a background job
    for ($attempt = 0; $attempt -le $MaxRetries; $attempt++) {
        try {
            # Use a job so Start-Service can't block the script forever
            $job = Start-Job -ScriptBlock {
                param($svcName)
                Start-Service -Name $svcName -ErrorAction Stop
            } -ArgumentList $ServiceName

            # Wait for the job with timeout
            $completed = $job | Wait-Job -Timeout $TimeoutSeconds

            if ($completed) {
                # Job finished (success or error)
                try { Receive-Job -Job $job -ErrorAction SilentlyContinue } catch { }
                Remove-Job -Job $job -Force -ErrorAction SilentlyContinue
            } else {
                # Job timed out - Start-Service is hanging
                Write-Host "    [$ServiceName] start timed out (${TimeoutSeconds}s) - force stopping job..." -ForegroundColor DarkYellow
                Stop-Job -Job $job -ErrorAction SilentlyContinue
                Remove-Job -Job $job -Force -ErrorAction SilentlyContinue

                # The service might now be in StartPending - try to unstick
                $service.Refresh()
                if ($service.Status -in $pendingStates) {
                    Resolve-StuckService -ServiceName $ServiceName -TimeoutSeconds $script:StuckServiceTimeoutSec | Out-Null
                }
            }

            # Check result
            Start-Sleep -Milliseconds 300
            $service.Refresh()
            if ($service.Status -eq 'Running') {
                $result.Status = "Started"
                return $result
            }

            # If stuck again after start attempt, unstick before retry
            if ($service.Status -in $pendingStates) {
                Resolve-StuckService -ServiceName $ServiceName -TimeoutSeconds $script:StuckServiceTimeoutSec | Out-Null
                $service.Refresh()
                if ($service.Status -eq 'Running') {
                    $result.Status = "Started"
                    return $result
                }
            }

        } catch {
            # Brief pause before retry
            Start-Sleep -Milliseconds 500
        }
    }

    # Final check
    $service.Refresh()
    if ($service.Status -eq 'Running') {
        $result.Status = "Started"
    } elseif ($service.Status -in $pendingStates) {
        $result.Status = "Stuck"
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
    $stuck = @()
    $failed = @()

    foreach ($def in $ServiceDefs) {
        $name = $def.Name
        $startupType = $def.StartupType

        $r = Start-ServiceRobust -ServiceName $name -StartupType $startupType -MaxRetries $MaxRetries -TimeoutSeconds $ServiceStartTimeoutSec
        switch ($r.Status) {
            "Started"        { $started++ }
            "AlreadyRunning" { $alreadyRunning++ }
            "NotFound"       { $notFound++ }
            "Stuck"          { $stuck += $name }
            "Failed"         { $failed += $name }
        }
    }

    $msg = "    Started: $started | Already running: $alreadyRunning | Not found: $notFound"
    Write-Host $msg -ForegroundColor DarkGray
    if ($stuck.Count -gt 0) {
        Write-Host "    Stuck ($($stuck.Count)): $($stuck -join ', ') - requires restart" -ForegroundColor Red
    }
    if ($failed.Count -gt 0) {
        Write-Host "    Failed ($($failed.Count)): $($failed -join ', ')" -ForegroundColor DarkYellow
    }

    return @{
        Started = $started
        AlreadyRunning = $alreadyRunning
        NotFound = $notFound
        Stuck = $stuck
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
# PHASE 4: TASKBAR PREFERENCES - HIDE SEARCH, DISABLE WIDGETS
# ============================================================

Write-Host "Configuring taskbar (hide search, disable Widgets)..." -ForegroundColor Green

$explorerRegPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Search"
$taskbarRegPath  = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"

# Hide Search from taskbar (0 = Hidden, 1 = Icon, 2 = Search box)
if (-not (Test-Path $explorerRegPath)) {
    New-Item -Path $explorerRegPath -Force | Out-Null
}
Set-ItemProperty -Path $explorerRegPath -Name "SearchboxTaskbarMode" -Value 0 -Type DWord -Force
Write-Host "  Taskbar search: hidden" -ForegroundColor Gray

# Disable Widgets / MSN News panel (TaskbarDa: 0 = disabled, 1 = enabled)
if (-not (Test-Path $taskbarRegPath)) {
    New-Item -Path $taskbarRegPath -Force | Out-Null
}
Set-ItemProperty -Path $taskbarRegPath -Name "TaskbarDa" -Value 0 -Type DWord -Force
Write-Host "  Widgets panel: disabled" -ForegroundColor Gray

# Also disable Widgets via Group Policy (prevents re-enabling)
$widgetsPolicyPath = "HKLM:\SOFTWARE\Policies\Microsoft\Dsh"
if (-not (Test-Path $widgetsPolicyPath)) {
    New-Item -Path $widgetsPolicyPath -Force -ErrorAction SilentlyContinue | Out-Null
}
if (Test-Path $widgetsPolicyPath) {
    Set-ItemProperty -Path $widgetsPolicyPath -Name "AllowNewsAndInterests" -Value 0 -Type DWord -Force -ErrorAction SilentlyContinue
    Write-Host "  Widgets group policy: disabled" -ForegroundColor Gray
}

# Kill any running Widget processes
$widgetProcs = @("Widgets", "WidgetService")
foreach ($wp in $widgetProcs) {
    $proc = Get-Process -Name $wp -ErrorAction SilentlyContinue
    if ($proc) {
        $proc | Stop-Process -Force -ErrorAction SilentlyContinue
        Write-Host "  Killed $wp process" -ForegroundColor Gray
    }
}

Write-Host ""

# ============================================================
# PHASE 5: START MENU RESTORATION + CACHE REBUILD
# ============================================================

Write-Host "Restoring Start Menu and rebuilding caches..." -ForegroundColor Green

# 5a. Ensure Start Menu dependency services are running
$startMenuServices = @("WpnService", "Themes", "ShellHWDetection", "WSearch", "TimeBrokerSvc")
foreach ($svcName in $startMenuServices) {
    $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
    if ($svc -and $svc.Status -ne 'Running') {
        Start-Service -Name $svcName -ErrorAction SilentlyContinue
    }
}
Write-Host "  Start Menu dependency services verified" -ForegroundColor Gray

# 5b. Rebuild icon cache
Write-Host "  Rebuilding icon cache..." -ForegroundColor Gray
$iconCachePath = "$env:LOCALAPPDATA\IconCache.db"
if (Test-Path $iconCachePath) {
    Remove-Item $iconCachePath -Force -ErrorAction SilentlyContinue
}
# Also clear the newer icon cache files (Windows 10/11 uses iconcache_* in Explorer)
$iconCacheDir = "$env:LOCALAPPDATA\Microsoft\Windows\Explorer"
if (Test-Path $iconCacheDir) {
    Get-ChildItem -Path $iconCacheDir -Filter "iconcache_*" -ErrorAction SilentlyContinue |
        ForEach-Object { Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue }
    Get-ChildItem -Path $iconCacheDir -Filter "thumbcache_*" -ErrorAction SilentlyContinue |
        ForEach-Object { Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue }
}
Write-Host "  Icon cache cleared (will rebuild on Explorer restart)" -ForegroundColor Gray

# 5c. Rebuild Start Menu tile database / layout cache
Write-Host "  Clearing Start Menu tile cache..." -ForegroundColor Gray
$tileDataDir = "$env:LOCALAPPDATA\Packages\Microsoft.Windows.StartMenuExperienceHost_cw5n1h2txyewy\TempState"
if (Test-Path $tileDataDir) {
    Get-ChildItem -Path $tileDataDir -Recurse -ErrorAction SilentlyContinue |
        Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
    Write-Host "  Start Menu tile cache cleared" -ForegroundColor Gray
}

# Also clear the general Start Menu cache
$startMenuCache = "$env:LOCALAPPDATA\Packages\Microsoft.Windows.StartMenuExperienceHost_cw5n1h2txyewy\LocalState"
if (Test-Path $startMenuCache) {
    Get-ChildItem -Path $startMenuCache -Filter "*.json" -ErrorAction SilentlyContinue |
        Remove-Item -Force -ErrorAction SilentlyContinue
    Write-Host "  Start Menu layout cache cleared" -ForegroundColor Gray
}

# 5d. Re-register Start Menu and ShellExperienceHost AppX packages
Write-Host "  Re-registering Start Menu AppX packages..." -ForegroundColor Gray
$appxPackages = @(
    "Microsoft.Windows.StartMenuExperienceHost",
    "Microsoft.Windows.ShellExperienceHost"
)
foreach ($pkgName in $appxPackages) {
    $pkg = Get-AppxPackage -Name $pkgName -ErrorAction SilentlyContinue
    if ($pkg) {
        $manifest = Join-Path $pkg.InstallLocation "AppXManifest.xml"
        if (Test-Path $manifest) {
            Add-AppxPackage -DisableDevelopmentMode -Register $manifest -ErrorAction SilentlyContinue
            Write-Host "    Re-registered $pkgName" -ForegroundColor DarkGray
        }
    }
}

# 5e. Kill and cleanly restart Explorer to pick up registry changes + rebuilt caches
Write-Host "  Restarting Explorer shell (clean restart)..." -ForegroundColor Gray
$explorerProc = Get-Process -Name "explorer" -ErrorAction SilentlyContinue
if ($explorerProc) {
    # Graceful shutdown request first, then force after timeout
    $explorerProc | ForEach-Object {
        $_.CloseMainWindow() | Out-Null
    }
    Start-Sleep -Milliseconds 800
    # Force kill any remaining explorer instances
    Get-Process -Name "explorer" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
}

# Start fresh Explorer
Start-Process "explorer.exe"
Write-Host "  Explorer restarted" -ForegroundColor Gray

# Wait for Explorer to initialize shell, taskbar, and Start Menu host
Write-Host "  Waiting for shell initialization..." -ForegroundColor Gray
$shellWait = [System.Diagnostics.Stopwatch]::StartNew()
$shellReady = $false
while ($shellWait.Elapsed.TotalSeconds -lt 15) {
    Start-Sleep -Milliseconds 500
    $startMenuHost = Get-Process -Name "StartMenuExperienceHost" -ErrorAction SilentlyContinue
    $shellHost = Get-Process -Name "ShellExperienceHost" -ErrorAction SilentlyContinue
    if ($startMenuHost -and $shellHost) {
        $shellReady = $true
        break
    }
}
$shellWait.Stop()

if ($shellReady) {
    Write-Host "  Shell hosts initialized" -ForegroundColor Gray
} else {
    # Force-launch the shell hosts if they didn't auto-start
    Write-Host "  Shell hosts did not auto-start, forcing launch..." -ForegroundColor Yellow

    # Trigger Start Menu host by simulating a Start Menu open via COM
    $wshell = New-Object -ComObject WScript.Shell -ErrorAction SilentlyContinue
    if ($wshell) {
        $wshell.SendKeys('^{ESCAPE}')  # Ctrl+Esc = open Start Menu
        Start-Sleep -Milliseconds 1500
        $wshell.SendKeys('{ESCAPE}')   # Close it
        Start-Sleep -Milliseconds 500
    }

    $startMenuHost = Get-Process -Name "StartMenuExperienceHost" -ErrorAction SilentlyContinue
    if (-not $startMenuHost) {
        Write-Host "  StartMenuExperienceHost still not running - may need reboot" -ForegroundColor Red
    }
}

# 5f. Start ctfmon (text input framework) - required for search/type-to-search in Start Menu
$ctfmon = Get-Process -Name "ctfmon" -ErrorAction SilentlyContinue
if (-not $ctfmon) {
    $ctfmonPath = "$env:SystemRoot\System32\ctfmon.exe"
    if (Test-Path $ctfmonPath) {
        Start-Process $ctfmonPath -ErrorAction SilentlyContinue
        Write-Host "  ctfmon (text input) started" -ForegroundColor Gray
    }
}

# 5g. Start TextInputHost (needed for Windows 11 Start Menu search/typing)
$textInput = Get-Process -Name "TextInputHost" -ErrorAction SilentlyContinue
if (-not $textInput) {
    # TextInputHost is an AppX process - trigger it by opening Start Menu briefly
    $wshell = New-Object -ComObject WScript.Shell -ErrorAction SilentlyContinue
    if ($wshell) {
        $wshell.SendKeys('^{ESCAPE}')
        Start-Sleep -Milliseconds 800
        $wshell.SendKeys('{ESCAPE}')
    }
    $textInput = Get-Process -Name "TextInputHost" -ErrorAction SilentlyContinue
    if ($textInput) {
        Write-Host "  TextInputHost started" -ForegroundColor Gray
    }
}

# 5h. SecurityHealthSystray (Windows Security icon)
$secHealthPath = "$env:ProgramFiles\Windows Defender\MSASCuiL.exe"
if (-not (Test-Path $secHealthPath -ErrorAction SilentlyContinue)) {
    $secHealthPath = "${env:ProgramFiles(x86)}\Windows Defender\MSASCuiL.exe"
}
if (Test-Path $secHealthPath -ErrorAction SilentlyContinue) {
    $secProc = Get-Process -Name "SecurityHealthSystray" -ErrorAction SilentlyContinue
    if (-not $secProc) {
        Start-Process $secHealthPath -ErrorAction SilentlyContinue
        Write-Host "  Windows Security systray started" -ForegroundColor Gray
    }
}

Write-Host ""

# ============================================================
# PHASE 6: VERIFICATION
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
            } elseif ($service.Status -in @('StopPending','StartPending','ContinuePending','PausePending')) {
                Write-Host "  [STUCK] $($svc.Desc) ($($service.Status)) - RESTART REQUIRED" -ForegroundColor Red
                $failCount++
            } else {
                Write-Host "  [WARN] $($svc.Desc) ($($service.Status))" -ForegroundColor Yellow
                $failCount++
            }
        }
    } else {
        Write-Host "  [--]   $($svc.Desc) (not installed)" -ForegroundColor DarkGray
    }
}

# Check Explorer and shell host processes
$shellChecks = @(
    @{Process="explorer";                   Desc="Explorer shell"},
    @{Process="StartMenuExperienceHost";    Desc="Start Menu host"},
    @{Process="ShellExperienceHost";        Desc="Shell Experience host"},
    @{Process="TextInputHost";              Desc="Text Input host"},
    @{Process="ctfmon";                     Desc="CTF text input"}
)

foreach ($sc in $shellChecks) {
    $proc = Get-Process -Name $sc.Process -ErrorAction SilentlyContinue
    if ($proc) {
        Write-Host "  [OK]   $($sc.Desc)" -ForegroundColor Green
        $okCount++
    } else {
        Write-Host "  [FAIL] $($sc.Desc)" -ForegroundColor Red
        $failCount++
    }
}

# Verify Widgets are disabled
$widgetsProc = Get-Process -Name "Widgets" -ErrorAction SilentlyContinue
if (-not $widgetsProc) {
    Write-Host "  [OK]   Widgets disabled" -ForegroundColor Green
    $okCount++
} else {
    Write-Host "  [WARN] Widgets still running" -ForegroundColor Yellow
    $failCount++
}

# Verify search is hidden from taskbar
$searchMode = Get-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Search" -Name "SearchboxTaskbarMode" -ErrorAction SilentlyContinue
if ($searchMode -and $searchMode.SearchboxTaskbarMode -eq 0) {
    Write-Host "  [OK]   Taskbar search hidden" -ForegroundColor Green
    $okCount++
} else {
    Write-Host "  [WARN] Taskbar search may still be visible" -ForegroundColor Yellow
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
