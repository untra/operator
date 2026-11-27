$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$scriptDirectory = Join-Path $PSScriptRoot '../runtime/powershell'
Import-Module PSScriptAnalyzer -RequiredVersion 1.24.0
$diagnostics = Invoke-ScriptAnalyzer -Path $scriptDirectory -Recurse -Severity Error,Warning
if ($diagnostics) {
    $diagnostics | Format-Table -AutoSize
    throw 'Runtime PowerShell lint failed'
}

$powerShell = Join-Path $PSHOME 'powershell.exe'
$readCredential = Get-Content (Join-Path $scriptDirectory 'read-credential.ps1') -Raw
$protectDirectory = Get-Content (Join-Path $scriptDirectory 'protect-registration-directory.ps1') -Raw
$root = Join-Path ([IO.Path]::GetTempPath()) ('operator script test ' + [Guid]::NewGuid())
$null = [IO.Directory]::CreateDirectory($root)

try {
    $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
    $private = New-Object Security.AccessControl.DirectorySecurity
    $private.SetSecurityDescriptorSddlForm("O:${sid}D:P(A;OICI;FA;;;${sid})")
    [IO.Directory]::SetAccessControl($root, $private)
    $directory = Join-Path $root "editor's sessions"
    $null = [IO.Directory]::CreateDirectory($directory)
    $env:OPERATOR_TERMINAL_REGISTRATIONS = $directory
    & $powerShell -NoProfile -NonInteractive -Command $protectDirectory
    if ($LASTEXITCODE -ne 0) { throw 'Protecting the registration directory failed' }

    $path = Join-Path $directory 'credential.json'
    $env:OPERATOR_PRIVATE_FILE = $path
    [IO.File]::WriteAllText($path, 'credential')
    $output = & $powerShell -NoProfile -NonInteractive -Command $readCredential
    if ($LASTEXITCODE -ne 0 -or $output -ne 'credential') {
        throw 'Owner-private credential did not round-trip'
    }

    [IO.File]::WriteAllText($path, 'x' * 4097)
    $output = & $powerShell -NoProfile -NonInteractive -Command $readCredential
    if ($LASTEXITCODE -eq 0 -or $output) { throw 'Oversized credential was accepted' }

    [IO.File]::WriteAllText($path, 'credential')
    $public = New-Object Security.AccessControl.FileSecurity
    $public.SetSecurityDescriptorSddlForm("O:${sid}D:P(A;;FA;;;${sid})(A;;FR;;;WD)")
    [IO.File]::SetAccessControl($path, $public)
    $output = & $powerShell -NoProfile -NonInteractive -Command $readCredential
    if ($LASTEXITCODE -eq 0 -or $output) { throw 'Public credential was accepted' }
} finally {
    Remove-Item -LiteralPath $root -Recurse -Force
}
