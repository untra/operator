CREATE TABLE oidc_connection (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL CHECK(provider IN ('keycloak', 'authentik')),
    issuer TEXT NOT NULL UNIQUE,
    client_id TEXT NOT NULL,
    client_secret_file TEXT,
    redirect_uri TEXT NOT NULL,
    disabled INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
CREATE TABLE operator_user (
    id TEXT PRIMARY KEY,
    connection_id TEXT NOT NULL REFERENCES oidc_connection(id),
    issuer TEXT NOT NULL,
    subject TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 0,
    administrator INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    UNIQUE(issuer, subject)
);
CREATE TABLE user_grant (
    user_id TEXT NOT NULL REFERENCES operator_user(id),
    profile_id TEXT NOT NULL,
    capability TEXT NOT NULL,
    PRIMARY KEY(user_id, profile_id, capability)
);
CREATE TABLE user_project (
    user_id TEXT NOT NULL REFERENCES operator_user(id),
    profile_id TEXT NOT NULL,
    project TEXT NOT NULL,
    PRIMARY KEY(user_id, profile_id, project)
);
CREATE TABLE user_profile (
    user_id TEXT NOT NULL REFERENCES operator_user(id),
    profile_id TEXT NOT NULL,
    instance_operations INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY(user_id, profile_id)
);
ALTER TABLE session ADD COLUMN user_id TEXT REFERENCES operator_user(id);
ALTER TABLE session ADD COLUMN authenticated_at INTEGER;
ALTER TABLE access_key ADD COLUMN user_id TEXT REFERENCES operator_user(id);
ALTER TABLE access_key ADD COLUMN profile_id TEXT;
ALTER TABLE access_key ADD COLUMN instance_operations INTEGER NOT NULL DEFAULT 0;
ALTER TABLE access_key ADD COLUMN replaces_id TEXT REFERENCES access_key(id);
CREATE UNIQUE INDEX idx_key_replacement ON access_key(replaces_id);
CREATE INDEX idx_key_owner_profile ON access_key(user_id, profile_id, created_at);
CREATE TABLE key_capability (
    key_id TEXT NOT NULL REFERENCES access_key(id),
    capability TEXT NOT NULL,
    PRIMARY KEY(key_id, capability)
);
CREATE TABLE key_project (
    key_id TEXT NOT NULL REFERENCES access_key(id),
    project TEXT NOT NULL,
    PRIMARY KEY(key_id, project)
);
UPDATE access_key SET revoked_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE revoked_at IS NULL;

CREATE TABLE key_issuance (
    user_id TEXT PRIMARY KEY REFERENCES operator_user(id),
    window_start INTEGER NOT NULL,
    issued INTEGER NOT NULL
);
