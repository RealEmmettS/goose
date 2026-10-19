[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$BaselineBinary,
    [Parameter(Mandatory)][string]$CandidateBinary,
    [Parameter(Mandatory)][string]$BaselineSettingsBinary,
    [Parameter(Mandatory)][string]$CandidateSettingsBinary,
    [Parameter(Mandatory)][string]$EvidenceDirectory,
    [ValidateSet('Walking','Leaves','Mud','Props','SettingsIdle','SettingsScroll')]
    [string[]]$Workloads=@('Walking','Leaves','Mud','Props','SettingsIdle','SettingsScroll'),
    [switch]$FrameProfile,
    [ValidatePattern('^[A-Za-z0-9_.-]{1,80}$')]
    [string]$BackgroundProcessName
)

# Both runtimes must expose the same opt-in timers when -FrameProfile is used.
# A timing-instrumented old source build must be labeled as such in the report;
# it does not replace the separate measurements of the installed public binary.
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$root=[IO.Path]::GetFullPath($EvidenceDirectory)
if(Test-Path -LiteralPath $root){throw 'Use a new comparison directory; raw trials are immutable.'}
New-Item -ItemType Directory -Path $root|Out-Null
$profiler=Join-Path $PSScriptRoot 'profile_windows_runtime.ps1'
$summarizer=Join-Path $PSScriptRoot 'summarize_windows_profile.py'
$binaries=@{baseline=(Resolve-Path -LiteralPath $BaselineBinary).Path;candidate=(Resolve-Path -LiteralPath $CandidateBinary).Path}
$settings=@{baseline=(Resolve-Path -LiteralPath $BaselineSettingsBinary).Path;candidate=(Resolve-Path -LiteralPath $CandidateSettingsBinary).Path}
$sequence=[Collections.Generic.List[object]]::new()

function Measure-Pair([string]$Workload,[int]$Round){
    $order=if($Round%2 -eq 1){@('baseline','candidate')}else{@('candidate','baseline')}
    foreach($label in $order){
        if(@(Get-Process cargo,rustc,zig -ErrorAction SilentlyContinue).Count -gt 0){throw 'A compiler is running; do not collect a comparison under build load.'}
        $directory=Join-Path $root "$($Workload.ToLowerInvariant())-$Round-$label"
        $arguments=@('-NoProfile','-File',$profiler,'-Binary',$binaries[$label],'-SettingsBinary',$settings[$label],'-EvidenceDirectory',$directory,'-Workload',$Workload,'-WarmupSeconds','30','-SampleSeconds','120')
        if($FrameProfile -and $Workload -notlike 'Settings*'){$arguments+='-EnableFrameProfile'}
        if($BackgroundProcessName){$arguments+=@('-BackgroundProcessName',$BackgroundProcessName)}
        $sequence.Add([ordered]@{workload=$Workload;round=$Round;label=$label;directory=$directory;started_utc=[DateTime]::UtcNow.ToString('O')})
        $sequence|ConvertTo-Json -Depth 4|Set-Content -LiteralPath (Join-Path $root 'sequence.json') -Encoding utf8NoBOM
        Write-Output "Comparison $Workload pair $Round`: $label"
        & pwsh @arguments
        if($LASTEXITCODE -ne 0){throw "Comparison phase failed: $directory"}
        $phaseResult=Get-Content -LiteralPath (Join-Path $directory 'result.json') -Raw|ConvertFrom-Json
        if(-not $phaseResult.completed){throw "Comparison phase did not complete: $directory"}
        & python $summarizer $directory --output (Join-Path $directory 'summary.json')
        if($LASTEXITCODE -ne 0){throw "Could not summarize $directory"}
    }
}

foreach($workload in $Workloads){
    foreach($round in 1..3){Measure-Pair $workload $round}
    $varying=$false
    $variation=[Collections.Generic.List[object]]::new()
    foreach($label in @('baseline','candidate')){
        $summaries=@(1..3|ForEach-Object{
            @(Get-Content -LiteralPath (Join-Path $root "$($workload.ToLowerInvariant())-$_-$label/summary.json") -Raw|ConvertFrom-Json)[0]
        })
        $metrics=@{cpu=@($summaries|ForEach-Object{$_.cpu.average_one_core_percent})}
        if($FrameProfile -and $workload -notlike 'Settings*'){$metrics.frame_p95=@($summaries|ForEach-Object{$_.frames.total_us.p95})}
        foreach($metric in $metrics.Keys){
            foreach($value in $metrics[$metric]){
                if($null -eq $value -or -not [double]::IsFinite([double]$value) -or $value -lt 0){throw "Invalid $metric measurement for $workload $label"}
            }
            $values=@($metrics[$metric]|Sort-Object)
            $median=$values[1]
            $zeroMedianRange=$median -eq 0 -and $values[-1] -gt 0
            $spread=if($median -gt 0){($values[-1]-$values[0])/$median}elseif($zeroMedianRange){$null}else{0}
            if($zeroMedianRange -or $spread -gt 0.10){$varying=$true}
            $variation.Add([ordered]@{label=$label;metric=$metric;values=$values;spread_fraction=$spread;zero_median_nonzero_range=$zeroMedianRange})
        }
    }
    $variation|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $root "$($workload.ToLowerInvariant())-variation.json") -Encoding utf8NoBOM
    if($varying){
        Write-Output "$workload exceeds ten-percent three-run variation; completing five pairs."
        foreach($round in 4..5){Measure-Pair $workload $round}
    }
}
Write-Output "Comparison phases complete: $root"
