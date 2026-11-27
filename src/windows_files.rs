//! Handle-based Windows file security. windows-sys exposes these APIs through FFI.
#![allow(unsafe_code, reason = "Win32 calls are encapsulated in this module")]

use std::fs::{File, OpenOptions};
use std::io;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{
    LocalFree, ERROR_SUCCESS, GENERIC_WRITE, HLOCAL, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Globalization::lstrlenW;
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, GetSecurityInfo,
    SDDL_REVISION_1, SE_FILE_OBJECT,
};
use windows_sys::Win32::Security::{
    AclSizeInformation, EqualSid, GetAce, GetAclInformation, GetTokenInformation, IsWellKnownSid,
    MapGenericMask, TokenUser, WinBuiltinAdministratorsSid, WinLocalSystemSid, ACCESS_ALLOWED_ACE,
    ACE_HEADER, ACL, ACL_SIZE_INFORMATION, DACL_SECURITY_INFORMATION, GENERIC_MAPPING,
    OWNER_SECURITY_INFORMATION, PSID, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, CREATE_NEW, DELETE, FILE_ALL_ACCESS, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_DELETE_CHILD, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
    FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES,
    FILE_WRITE_DATA, FILE_WRITE_EA, READ_CONTROL, WRITE_DAC, WRITE_OWNER,
};
use windows_sys::Win32::System::SystemServices::{ACCESS_ALLOWED_ACE_TYPE, ACCESS_DENIED_ACE_TYPE};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

const PARENT_MUTATION_RIGHTS: u32 = FILE_WRITE_DATA
    | FILE_APPEND_DATA
    | FILE_WRITE_EA
    | FILE_WRITE_ATTRIBUTES
    | FILE_DELETE_CHILD
    | DELETE
    | WRITE_DAC
    | WRITE_OWNER;

pub(crate) enum RegistrationObject {
    Parent,
    Directory,
    File,
}

struct LocalAllocation(HLOCAL);

impl Drop for LocalAllocation {
    fn drop(&mut self) {
        // SAFETY: This pointer was allocated by the corresponding Win32 API.
        unsafe { LocalFree(self.0) };
    }
}

pub(crate) struct CurrentUser(Vec<usize>);

impl CurrentUser {
    pub(crate) fn new() -> io::Result<Self> {
        let mut token = null_mut();
        // SAFETY: The token output is writable and the process handle is valid.
        check(unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) })?;
        // SAFETY: OpenProcessToken returned an owned handle.
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut length = 0;
        // SAFETY: A null buffer and zero size request the required buffer size.
        unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                null_mut(),
                0,
                &raw mut length,
            )
        };
        if length < size_of::<TOKEN_USER>() as u32 {
            return Err(io::Error::last_os_error());
        }
        let mut buffer = vec![0usize; (length as usize).div_ceil(size_of::<usize>())];
        // SAFETY: The buffer is aligned and has at least length bytes.
        check(unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                buffer.as_mut_ptr().cast(),
                length,
                &raw mut length,
            )
        })?;
        Ok(Self(buffer))
    }

    fn sid(&self) -> PSID {
        // SAFETY: GetTokenInformation stored TOKEN_USER and its SID in this buffer.
        unsafe { (*self.0.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }

    pub(crate) fn open_registration(
        &self,
        path: &Path,
        object: RegistrationObject,
    ) -> io::Result<Option<File>> {
        let is_file = matches!(object, RegistrationObject::File);
        let access = if is_file {
            FILE_GENERIC_READ
        } else {
            READ_CONTROL | FILE_READ_ATTRIBUTES
        };
        let file = OpenOptions::new()
            .access_mode(access)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(path)?;
        let metadata = file.metadata()?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || (is_file && !metadata.is_file())
            || (!is_file && !metadata.is_dir())
        {
            return Ok(None);
        }
        let private = !matches!(object, RegistrationObject::Parent);
        Ok(Self::allows_registration(&file, self.sid(), private)?.then_some(file))
    }

    fn allows_registration(file: &File, user: PSID, private: bool) -> io::Result<bool> {
        let mut owner = null_mut();
        let mut acl = null_mut();
        let mut descriptor = null_mut();
        // SAFETY: The handle is live and all requested outputs are writable.
        let status = unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                &raw mut owner,
                null_mut(),
                &raw mut acl,
                null_mut(),
                &raw mut descriptor,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(io::Error::from_raw_os_error(status.cast_signed()));
        }
        let _descriptor = LocalAllocation(descriptor);
        // SAFETY: The SID pointers remain valid while their allocations are live.
        if owner.is_null() || acl.is_null() || unsafe { EqualSid(owner, user) } == 0 {
            return Ok(false);
        }

        let mut info = ACL_SIZE_INFORMATION::default();
        // SAFETY: GetSecurityInfo supplied the ACL and info is a writable ACL_SIZE_INFORMATION.
        check(unsafe {
            GetAclInformation(
                acl,
                (&raw mut info).cast(),
                size_of::<ACL_SIZE_INFORMATION>() as u32,
                AclSizeInformation,
            )
        })?;
        for index in 0..info.AceCount {
            if !allows_ace(acl, index, user, private)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn create_private_file(&self, path: &Path) -> io::Result<File> {
        let mut sid_text = null_mut();
        // SAFETY: The SID comes from the live token buffer and the output is writable.
        check(unsafe { ConvertSidToStringSidW(self.sid(), &raw mut sid_text) })?;
        let _sid_allocation = LocalAllocation(sid_text.cast());
        // SAFETY: ConvertSidToStringSidW returns a null-terminated UTF-16 string.
        let length = unsafe { lstrlenW(sid_text) } as usize;
        // SAFETY: length was measured within the live null-terminated allocation.
        let sid = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(sid_text, length) });
        create_file_with_acl(path, &format!("O:{sid}D:P(A;;FA;;;{sid})"))
    }
}

fn allows_ace(acl: *const ACL, index: u32, user: PSID, private: bool) -> io::Result<bool> {
    let mut ace = null_mut();
    // SAFETY: The descriptor owns acl, and index is within its reported ACE count.
    check(unsafe { GetAce(acl, index, &raw mut ace) })?;
    // SAFETY: GetAce returned a pointer to the ACE header in the live ACL.
    let header = unsafe { &*ace.cast::<ACE_HEADER>() };
    match u32::from(header.AceType) {
        ACCESS_DENIED_ACE_TYPE => return Ok(true),
        ACCESS_ALLOWED_ACE_TYPE => {}
        _ => return Ok(false),
    }
    // SAFETY: The header identifies an ACCESS_ALLOWED_ACE in the OS-supplied ACL.
    let ace = unsafe { &*ace.cast::<ACCESS_ALLOWED_ACE>() };
    let sid = (&raw const ace.SidStart).cast_mut().cast();
    // SAFETY: Both SIDs are held by the live ACL and token buffer.
    if unsafe { EqualSid(sid, user) } != 0 {
        return Ok(true);
    }
    if private {
        return Ok(false);
    }

    let mut mask = ace.Mask;
    let mapping = GENERIC_MAPPING {
        GenericRead: FILE_GENERIC_READ,
        GenericWrite: FILE_GENERIC_WRITE,
        GenericExecute: FILE_GENERIC_EXECUTE,
        GenericAll: FILE_ALL_ACCESS,
    };
    // SAFETY: All pointers are valid and the SID stays live in the ACL.
    Ok(unsafe {
        MapGenericMask(&raw mut mask, &raw const mapping);
        mask & PARENT_MUTATION_RIGHTS == 0
            || IsWellKnownSid(sid, WinLocalSystemSid) != 0
            || IsWellKnownSid(sid, WinBuiltinAdministratorsSid) != 0
    })
}

fn create_file_with_acl(path: &Path, sddl: &str) -> io::Result<File> {
    let filename = path.file_name().ok_or(io::ErrorKind::InvalidInput)?;
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let path = parent.canonicalize()?.join(filename);
    let mut name: Vec<_> = path.as_os_str().encode_wide().collect();
    if name.contains(&0) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    name.push(0);

    let sddl: Vec<_> = sddl.encode_utf16().chain([0]).collect();
    let mut descriptor = null_mut();
    // SAFETY: The SDDL is null terminated and descriptor is a writable output.
    check(unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &raw mut descriptor,
            null_mut(),
        )
    })?;
    let descriptor = LocalAllocation(descriptor);
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    // SAFETY: name is null terminated and attributes holds a live security descriptor.
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            GENERIC_WRITE,
            0,
            &raw const attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: CreateFileW returned an owned file handle.
    Ok(unsafe { File::from_raw_handle(handle) })
}

fn check(result: i32) -> io::Result<()> {
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn test_windows_registration_reads_owner_private_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("editor's registration.json");
        let user = CurrentUser::new().unwrap();
        user.create_private_file(&path)
            .unwrap()
            .write_all(b"private")
            .unwrap();
        let mut file = user
            .open_registration(&path, RegistrationObject::File)
            .unwrap()
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        assert_eq!(contents, "private");
    }

    #[test]
    fn test_windows_registration_rejects_a_different_owner() {
        use windows_sys::Win32::Security::Authorization::ConvertStringSidToSidW;

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("registration.json");
        let user = CurrentUser::new().unwrap();
        drop(user.create_private_file(&path).unwrap());
        let file = File::open(path).unwrap();

        let null_sid: Vec<_> = "S-1-0-0".encode_utf16().chain([0]).collect();
        let mut sid = null_mut();
        // SAFETY: The SID string is null terminated and the output is writable.
        check(unsafe { ConvertStringSidToSidW(null_sid.as_ptr(), &raw mut sid) }).unwrap();
        let _sid_allocation = LocalAllocation(sid);
        assert!(!CurrentUser::allows_registration(&file, sid, true).unwrap());
    }

    #[test]
    fn test_windows_registration_rejects_a_directory_junction() {
        let directory = tempfile::tempdir().unwrap();
        let junction = directory.path().join("junction");
        let target = directory.path().join("target");
        std::fs::create_dir(&target).unwrap();
        assert!(std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&target)
            .output()
            .unwrap()
            .status
            .success());

        let user = CurrentUser::new().unwrap();
        for object in [RegistrationObject::Parent, RegistrationObject::Directory] {
            assert!(user.open_registration(&junction, object).unwrap().is_none());
        }
    }

    #[test]
    fn test_windows_registration_denies_file_with_public_grant() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("public.json");
        drop(create_file_with_acl(&path, "D:P(A;;FA;;;WD)").unwrap());
        let user = CurrentUser::new().unwrap();
        assert!(user
            .open_registration(&path, RegistrationObject::File)
            .unwrap()
            .is_none());
    }
}
