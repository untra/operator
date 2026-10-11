---
title: "Operator Premium"
description: "What is included, what Premium adds, and how a license is installed and verified."
layout: doc
---

Operator is free for local work for developers to work on their laptops.
Operator Premium is a paid tier that includes features ideal for larger teams or work done on more machines.

## What each tier covers

| | Included | Premium |
|---|---|---|
| Local agents | Multiple, in parallel | Multiple, in parallel |
| Local containers ([Docker](/getting-started/platforms/docker/)) | Yes | Yes |
| Named configurations | Yes | Yes |
| Dashboard over the network | Yes | Yes |
| External model APIs | Yes | Yes |
| [SSH hosts](/getting-started/remote-targets/ssh/) | - | Yes |
| [Coder workspaces](/getting-started/remote-targets/coder/) | - | Yes |
| [Campaigns](/getting-started/concepts/campaigns/) | - | Yes |

Running an **agent process** on another machine requires Premium.

## Installing a license

A license is a single text key. Install it from either surface:

- **Terminal** - the Operator Premium step of the setup wizard, or the License
  section of the dashboard.
- **Browser** - *License* under Premium in the sidebar.

Installing validates the key before storing it. The key is stored in the configuration's state directory with owner-only permissions and is never returned by the API or written to logs.

## How verification works

Verification is **entirely offline**. Operator never contacts a licensing service, at install time or afterwards, and there is no activation step.

A license key is an token plus a root-signed attestation for the key that signed it. The token identifies customer it was issued to, its license id, the configuration it belongs to, and validity dates.

Operator checks against a root public keyring compiled into the binary, then checks those claims. A license is bound to one **configuration** at a time.

### Status

| Status | Meaning |
|---|---|
| Free | No license installed. Local execution is unaffected |
| Premium | Verified and currently valid |
| Expired | Verified, but past its end date |
| Not yet valid | Verified, but its start date is in the future |
| Invalid | Signature, issuer, tier, configuration or format rejected |

Only **Premium** grants remote execution. An unrecognised tier grants nothing.
