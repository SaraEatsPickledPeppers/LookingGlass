# Interactive, two-process development launcher. Each player has separate settings.
# Campaign co-op is unfinished; this currently opens the movement test.
param(
    [Parameter(Mandatory = $true)][string]$Data,
    [string]$Map = 'skool1',
    [switch]$Local,
    [switch]$Menu
)
$ErrorActionPreference = 'Stop'
if ($Menu -and $Local) { throw 'The in-game Host menu creates internet invites. Omit -Local when using -Menu.' }
$projectRoot = Split-Path -Parent $PSScriptRoot
$executable = Join-Path $projectRoot 'target\debug\looking-glass.exe'
if (-not (Test-Path -LiteralPath $executable)) {
    throw 'Build Looking Glass first with cargo build --locked.'
}
$dataPath = (Resolve-Path -LiteralPath $Data).Path
$sessionFolder = Join-Path $projectRoot ('private\coop\manual-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $sessionFolder
$inviteFile = Join-Path $sessionFolder 'invite.txt'
$owned = @()
try {
    foreach ($role in @('host', 'guest')) {
        if ($role -eq 'guest' -and -not $Menu) {
            $deadline = [DateTime]::UtcNow.AddSeconds(150)
            while (-not (Test-Path -LiteralPath $inviteFile)) {
                if ($owned[0].HasExited) { throw 'The host closed before its invite was ready.' }
                if ([DateTime]::UtcNow -gt $deadline) { throw 'The internet invite took too long to open.' }
                Start-Sleep -Milliseconds 100
            }
        }
        $roleFlag = if ($role -eq 'host') { '--coop-host' } else { '--coop-join' }
        $profile = Join-Path $sessionFolder $role
        $null = New-Item -ItemType Directory -Path $profile
        $arguments = @('--data', $dataPath, '--profile', $profile, '--no-audio')
        if (-not $Menu) { $arguments += @('--map', $Map, $roleFlag, $inviteFile) }
        if ($Local -and $role -eq 'host') { $arguments += '--coop-local' }
        # These are native Windows arguments, not shell source. Paths cannot
        # contain a quotation mark; reject it in the user-supplied map too.
        if ($arguments | Where-Object { $_.Contains('"') }) { throw 'An argument contains an invalid quotation mark.' }
        $quoted = $arguments | ForEach-Object { '"' + [regex]::Replace($_, '(\\+)$', '$1$1') + '"' }
        $owned += Start-Process -FilePath $executable -ArgumentList $quoted -WorkingDirectory $projectRoot `
            -WindowStyle Normal -PassThru -RedirectStandardOutput (Join-Path $sessionFolder "$role.log") `
            -RedirectStandardError (Join-Path $sessionFolder "$role-error.log")
    }
    if ($Menu) {
        Write-Host 'Two independent main menus are open. Choose Co-op > Host in one, copy its invite, then Co-op > Join in the other.'
    } else {
        Write-Host 'Two clients are open: Blue is the host; Red is the guest.'
    }
    Write-Host 'Focus a window to control that Alice. ESC opens its menu; Q leaves co-op.'
    Write-Host 'This is a movement test. Campaign co-op remains unfinished.'
    Write-Host "Profiles and diagnostics: $sessionFolder"
    while (@($owned | Where-Object { -not $_.HasExited }).Count -gt 0) {
        Start-Sleep -Milliseconds 200
    }
} finally {
    foreach ($process in $owned) {
        if (-not $process.HasExited) {
            $null = $process.CloseMainWindow()
            if (-not $process.WaitForExit(5000)) {
                & "$env:SystemRoot\System32\taskkill.exe" /PID $process.Id /T /F | Out-Null
            }
        }
    }
    if (Test-Path -LiteralPath $inviteFile) { Remove-Item -LiteralPath $inviteFile }
}
