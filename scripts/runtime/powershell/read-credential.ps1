$ErrorActionPreference = 'Stop'
$share = [IO.FileShare]::Read -bor [IO.FileShare]::Delete
$stream = [IO.File]::Open($env:OPERATOR_PRIVATE_FILE, 'Open', 'Read', $share)
try {
  $acl = $stream.GetAccessControl()
  $sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
  if (!$acl.GetOwner([Security.Principal.SecurityIdentifier]).Equals($sid)) {exit 1}
  foreach ($rule in $acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])) {
    if ($rule.AccessControlType -eq 'Allow' -and !$rule.IdentityReference.Equals($sid)) {exit 1}
  }
  if ($stream.Length -gt 4096) {exit 1}
  [Console]::Write((New-Object IO.StreamReader($stream)).ReadToEnd())
} finally { $stream.Dispose() }
