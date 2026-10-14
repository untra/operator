use operator_telemetry::EventName;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    Bootstrap,
    Login,
    Logout,
    PasswordReset,
    SessionRevoked,
    DeviceCodeIssued,
    DeviceApproved,
    RefreshTokenReuse,
    AccessKeyExchange,
    AccessKeyCreated,
    AccessKeyRevoked,
    AdminCreated,
    OidcConnectionCreated,
    OidcConnectionDisabled,
    UserPermissionsChanged,
    OidcLogin,
    AccessKeyRotated,
}
impl AuditAction {
    #[cfg(test)]
    pub const ALL: [Self; 17] = [
        Self::Bootstrap,
        Self::Login,
        Self::Logout,
        Self::PasswordReset,
        Self::SessionRevoked,
        Self::DeviceCodeIssued,
        Self::DeviceApproved,
        Self::RefreshTokenReuse,
        Self::AccessKeyExchange,
        Self::AccessKeyCreated,
        Self::AccessKeyRevoked,
        Self::AdminCreated,
        Self::OidcConnectionCreated,
        Self::OidcConnectionDisabled,
        Self::UserPermissionsChanged,
        Self::OidcLogin,
        Self::AccessKeyRotated,
    ];
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bootstrap => "bootstrap",
            Self::Login => "login",
            Self::Logout => "logout",
            Self::PasswordReset => "password reset",
            Self::SessionRevoked => "session revoked",
            Self::DeviceCodeIssued => "device code issued",
            Self::DeviceApproved => "device approved",
            Self::RefreshTokenReuse => "refresh token reuse",
            Self::AccessKeyExchange => "access key exchange",
            Self::AccessKeyCreated => "access key created",
            Self::AccessKeyRevoked => "access key revoked",
            Self::AdminCreated => "admin created",
            Self::OidcConnectionCreated => "oidc connection created",
            Self::OidcConnectionDisabled => "oidc connection disabled",
            Self::UserPermissionsChanged => "user permissions changed",
            Self::OidcLogin => "oidc login",
            Self::AccessKeyRotated => "access key rotated",
        }
    }
    pub const fn event(self) -> EventName {
        match self {
            Self::Bootstrap => EventName::OperatorAuthBootstrap,
            Self::Login => EventName::OperatorAuthLogin,
            Self::Logout => EventName::OperatorAuthLogout,
            Self::PasswordReset => EventName::OperatorAuthPasswordReset,
            Self::SessionRevoked => EventName::OperatorAuthSessionRevoked,
            Self::DeviceCodeIssued => EventName::OperatorAuthDeviceCodeIssued,
            Self::DeviceApproved => EventName::OperatorAuthDeviceApproved,
            Self::RefreshTokenReuse => EventName::OperatorAuthRefreshTokenReuse,
            Self::AccessKeyExchange => EventName::OperatorAuthAccessKeyExchange,
            Self::AccessKeyCreated => EventName::OperatorAuthAccessKeyCreated,
            Self::AccessKeyRevoked => EventName::OperatorAuthAccessKeyRevoked,
            Self::AdminCreated => EventName::OperatorAuthAdminCreated,
            Self::OidcConnectionCreated => EventName::OperatorAuthOidcConnectionCreated,
            Self::OidcConnectionDisabled => EventName::OperatorAuthOidcConnectionDisabled,
            Self::UserPermissionsChanged => EventName::OperatorAuthUserPermissionsChanged,
            Self::OidcLogin => EventName::OperatorAuthOidcLogin,
            Self::AccessKeyRotated => EventName::OperatorAuthAccessKeyRotated,
        }
    }
}
