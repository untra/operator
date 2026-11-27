$ErrorActionPreference = 'Stop'
$dir = [IO.DirectoryInfo]::new($env:OPERATOR_TERMINAL_REGISTRATIONS)
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
foreach ($item in @($dir.Parent,$dir)) {
  if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {exit 1}
  $acl = $item.GetAccessControl()
  if (!$acl.GetOwner([Security.Principal.SecurityIdentifier]).Equals($sid)) {exit 1}
  foreach ($rule in $acl.GetAccessRules($true,$true,[Security.Principal.SecurityIdentifier])) {
    if ($rule.AccessControlType -eq 'Allow' -and $rule.IdentityReference.Value -notin @($sid.Value,'S-1-5-18','S-1-5-32-544') -and ($rule.FileSystemRights -band [Security.AccessControl.FileSystemRights]'Write, Delete, DeleteSubdirectoriesAndFiles, ChangePermissions, TakeOwnership')) {exit 1}
  }
}
$private = New-Object Security.AccessControl.DirectorySecurity
$private.SetOwner($sid)
$private.SetAccessRuleProtection($true, $false)
$private.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow')))
$dir.SetAccessControl($private)
