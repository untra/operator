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

Reaching the dashboard over a network, and calling a model provider's API over the
internet, are not remote execution. Only running an **agent process** on another
machine requires Premium.

## Installing a license

A license is a single text key. Install it from either surface:

- **Terminal** - the Operator Premium step of the setup wizard, or the License
  section of the dashboard.
- **Browser** - *License* under Premium in the sidebar.

Installing validates the key before storing it, so a rejected key leaves any
existing license in place. The key is stored in the configuration's state
directory with owner-only permissions and is never returned by the API or
written to logs.

## How verification works

Verification is **entirely offline**. Operator never contacts a licensing
service, at install time or afterwards, and there is no activation step.

A license is a signed token carrying the customer it was issued to, its license
id, its tier, the configuration it belongs to, and its validity dates. Operator
checks the signature against verification keys compiled into the binary, then
checks those claims. A license is bound to one **configuration**, identified by
a stable id that survives renaming the configuration.

### Status

| Status | Meaning |
|---|---|
| Free | No license installed. Local execution is unaffected |
| Premium | Verified and currently valid |
| Expired | Verified, but past its end date |
| Not yet valid | Verified, but its start date is in the future |
| Invalid | Signature, issuer, tier, configuration or format rejected |

Only **Premium** grants remote execution. An unrecognised tier grants nothing.

## When a license expires

Nothing is killed. Running agents keep running, and a completion report from an
agent already in flight is still accepted and recorded. What stops is *starting*
further remote work: the next launch is refused before anything is provisioned.

Configured remote targets stay visible and readable without a license, and
removing one always works. Registering, editing or probing a target requires
Premium.

## Building from source

Verification keys are supplied at build time and are **not** in this repository,
so a build from source carries none and rejects every license - Premium is
unreachable in such a build, by design. This repository contains no signing key,
issuer service, checkout, or revocation service; it only *consumes* licenses
issued elsewhere.

The build-time inputs are listed under Licensing in the
[CLI reference](/cli/). A release build refuses to compile without them.
