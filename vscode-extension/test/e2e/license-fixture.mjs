import { spawnSync } from "node:child_process";
import {
  createPrivateKey,
  createPublicKey,
  generateKeyPairSync,
  randomUUID,
  sign,
} from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const directory = process.env.OPERATOR_E2E_TRUST_DIR;
if (!directory) {
  throw new Error("Set OPERATOR_E2E_TRUST_DIR to a temporary test directory");
}
const rootFile = join(directory, "root.pem");
const rootId = "operator-e2e-root";
const signerId = "operator-e2e-signing";
const issuer = "operator-e2e";
const [action, ...args] = process.argv.slice(2);

function publicBytes(key) {
  return Buffer.from(createPublicKey(key).export({ format: "jwk" }).x, "base64url").toString(
    "base64",
  );
}

const encode = (value) => Buffer.from(JSON.stringify(value)).toString("base64url");

function jwt(kid, claims, key) {
  const content = `${encode({ alg: "EdDSA", typ: "JWT", kid })}.${encode(claims)}`;
  return `${content}.${sign(null, Buffer.from(content), key).toString("base64url")}`;
}

if (action === "build" && args.shift() === "--") {
  mkdirSync(directory, { recursive: true, mode: 0o700 });
  const root = generateKeyPairSync("ed25519").privateKey;
  writeFileSync(rootFile, root.export({ format: "pem", type: "pkcs8" }), {
    mode: 0o600,
    flag: "wx",
  });
  const result = spawnSync("cargo", args, {
    stdio: "inherit",
    env: {
      ...process.env,
      OPERATOR_LICENSE_ROOT_KEYS: JSON.stringify({ [rootId]: publicBytes(root) }),
      OPERATOR_LICENSE_ISSUER: issuer,
    },
  });
  if (result.error) {
    throw result.error;
  }
  process.exitCode = result.status ?? 1;
} else if (action === "issue" && args.length === 2) {
  const [profile, state] = args;
  const root = createPrivateKey(readFileSync(rootFile));
  const signer = generateKeyPairSync("ed25519").privateKey;
  const now = Math.floor(Date.now() / 1000);
  const window = { version: 1, iat: now - 60, nbf: now - 60, exp: now + 3600 };
  const att = jwt(
    rootId,
    {
      ...window,
      iss: "untra-root",
      aud: "untra-trust",
      sub: "operator",
      purpose: "license-signing",
      kid: signerId,
      pub: publicBytes(signer),
    },
    root,
  );
  const lic = jwt(
    signerId,
    {
      ...window,
      iss: issuer,
      aud: "operator-license",
      sub: "e2e",
      jti: randomUUID(),
      profile_id: profile,
      tier: "premium",
    },
    signer,
  );
  mkdirSync(state, { recursive: true, mode: 0o700 });
  writeFileSync(
    join(state, "license.key"),
    Buffer.from(JSON.stringify({ v: 1, att, lic })).toString("base64"),
    { mode: 0o600, flag: "wx" },
  );
} else {
  throw new Error("Expected build -- <cargo arguments> or issue <profile UUID> <state directory>");
}
