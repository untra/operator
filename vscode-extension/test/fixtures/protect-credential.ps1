$acl = New-Object Security.AccessControl.FileSecurity
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User
$acl.SetOwner($sid)
$acl.SetAccessRuleProtection($true, $false)
$acl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($sid, 'FullControl','Allow')))
[IO.File]::SetAccessControl($env:OPERATOR_PRIVATE_FILE, $acl)
