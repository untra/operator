---
title: "Downloads"
description: "Download Operator! binaries for macOS, Linux, and Windows."
layout: doc
---

Download <span class="operator-brand">Operator!</span> for your platform. Current version: **v{{ site.version }}**

## VS Code Extension (Recommended)

The **VS Code Extension** is the recommended way to get started with Operator. It provides integrated terminal management, ticket tracking, and a streamlined workflow directly in your editor.

<a href="https://marketplace.visualstudio.com/items?itemName=untra.operator-terminals" target="_blank" class="button">Install from VS Code Marketplace</a>

Works on **macOS**, **Linux**, and **Windows**. A local install needs no additional setup; to use an Operator server elsewhere, choose **Connect to a server** and sign in ([details](/getting-started/ides/vscode/#choose-a-connection)).

---

## CLI Downloads

For headless servers, CI/CD pipelines, or advanced workflows, download the CLI binary for your platform.

<div id="recommended-download">
  <noscript>See the download tables below for all available platforms.</noscript>
</div>

### All <span class="operator-brand">Operator!</span> Downloads

| Platform | Architecture | Download |
|----------|--------------|----------|
| macOS | ARM64 (Apple Silicon) | [operator-macos-arm64]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-macos-arm64)<br><small>sha256:<code>{{ site.data.checksums.operator.macos_arm64 }}</code></small> |
| Linux | ARM64 | [operator-linux-arm64]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-linux-arm64)<br><small>sha256:<code>{{ site.data.checksums.operator.linux_arm64 }}</code></small> |
| Linux | x86_64 | [operator-linux-x86_64]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-linux-x86_64)<br><small>sha256:<code>{{ site.data.checksums.operator.linux_x86_64 }}</code></small> |
| Windows | x86_64 | [operator-windows-x86_64.exe]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-windows-x86_64.exe)<br><small>sha256:<code>{{ site.data.checksums.operator.windows_x86_64 }}</code></small> |
| Windows | ARM64 | [operator-windows-arm64.exe]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-windows-arm64.exe)<br><small>sha256:<code>{{ site.data.checksums.operator.windows_arm64 }}</code></small> |

### opr8r step wrapper

`opr8r` runs beside each agent CLI on the machine where agents execute. It reports step completion to Operator and starts the next workflow step. The container image and the VS Code extension already include it, so you only need these downloads for a hand-built execution host. A machine that only serves models, such as a gaming PC running [LM Studio](/getting-started/model-servers/lmstudio/), does not need `opr8r`. See [platform support](/getting-started/platform-support/) for which operating systems can be execution hosts.

| Platform | Architecture | Download |
|----------|--------------|----------|
| macOS | ARM64 (Apple Silicon) | [opr8r-macos-arm64]({{ site.github.repo }}/releases/download/v{{ site.version }}/opr8r-macos-arm64)<br><small>sha256:<code>{{ site.data.checksums.opr8r.macos_arm64 }}</code></small> |
| Linux | ARM64 | [opr8r-linux-arm64]({{ site.github.repo }}/releases/download/v{{ site.version }}/opr8r-linux-arm64)<br><small>sha256:<code>{{ site.data.checksums.opr8r.linux_arm64 }}</code></small> |
| Linux | x86_64 | [opr8r-linux-x86_64]({{ site.github.repo }}/releases/download/v{{ site.version }}/opr8r-linux-x86_64)<br><small>sha256:<code>{{ site.data.checksums.opr8r.linux_x86_64 }}</code></small> |
| Windows | x86_64 | [opr8r-windows-x86_64.exe]({{ site.github.repo }}/releases/download/v{{ site.version }}/opr8r-windows-x86_64.exe)<br><small>sha256:<code>{{ site.data.checksums.opr8r.windows_x86_64 }}</code></small> |
| Windows | ARM64 | [opr8r-windows-arm64.exe]({{ site.github.repo }}/releases/download/v{{ site.version }}/opr8r-windows-arm64.exe)<br><small>sha256:<code>{{ site.data.checksums.opr8r.windows_arm64 }}</code></small> |

## Kubernetes

Run Operator in a cluster from the official OCI Helm chart:

```bash
helm install operator oci://ghcr.io/untra/charts/operator \
  --namespace operator --create-namespace \
  --set publicUrl=https://operator.example.com
```

See the [Kubernetes guide](/getting-started/platforms/kubernetes/) for bootstrap, TLS, persistence, and NetworkPolicy.

## Container image

```bash
docker pull untra/operator:{{ site.version }}
```

Multi-arch (`linux/amd64`, `linux/arm64`). See the [Docker guide](/getting-started/platforms/docker/).

## All Releases

[View all releases on GitHub]({{ site.github.repo }}/releases)

<script>
(function() {
  var container = document.getElementById('recommended-download');

  // Detect OS from userAgentData or fallback
  function detectOS() {
    if (navigator.userAgentData && navigator.userAgentData.platform) {
      var p = navigator.userAgentData.platform.toLowerCase();
      if (p.indexOf('mac') !== -1) return 'macos';
      if (p.indexOf('win') !== -1) return 'windows';
      if (p.indexOf('linux') !== -1) return 'linux';
    }
    // Fallback to navigator.platform
    var platform = (navigator.platform || '').toLowerCase();
    if (platform.indexOf('mac') !== -1) return 'macos';
    if (platform.indexOf('win') !== -1) return 'windows';
    return 'linux';
  }

  // Render the download recommendation
  function render(os, arch) {
    var artifactName, url, label, archLabel;

    if (os === 'windows') {
      artifactName = 'operator-windows-' + arch + '.exe';
      url = '{{ site.github.repo }}/releases/download/v{{ site.version }}/' + artifactName;
      label = 'Windows';
      archLabel = arch === 'arm64' ? 'ARM64' : 'x86_64';
    } else {
      artifactName = 'operator-' + os + '-' + arch;
      url = '{{ site.github.repo }}/releases/download/v{{ site.version }}/' + artifactName;
      label = os === 'macos' ? 'macOS' : 'Linux';
      archLabel = arch === 'arm64' ? (os === 'macos' ? 'Apple Silicon' : 'ARM64') : 'x86_64';
    }

    container.innerHTML = '<div class="recommended-box">' +
      '<p><strong>Recommended for your system:</strong> ' + label + ' ' + archLabel + '</p>' +
      '<a href="' + url + '" class="download-button">Download ' + artifactName + '</a>' +
      '</div>';
  }

  var os = detectOS();

  // Use User-Agent Client Hints API for accurate architecture detection
  if (navigator.userAgentData && navigator.userAgentData.getHighEntropyValues) {
    navigator.userAgentData.getHighEntropyValues(['architecture', 'bitness'])
      .then(function(values) {
        var arch = 'x86_64';
        if (values.architecture === 'arm') {
          arch = 'arm64';
        }
        render(os, arch);
      })
      .catch(function() {
        // Fallback if high entropy values fail
        render(os, 'x86_64');
      });
  } else {
    // Fallback for browsers without userAgentData (Safari, Firefox)
    // On macOS, check for Apple Silicon via WebGL renderer
    var arch = 'x86_64';
    if (os === 'macos') {
      try {
        var canvas = document.createElement('canvas');
        var gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
        if (gl) {
          var debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
          if (debugInfo) {
            var renderer = gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL);
            // Apple Silicon GPUs contain "Apple" in the renderer string
            if (renderer && renderer.indexOf('Apple') !== -1 && renderer.indexOf('Intel') === -1) {
              arch = 'arm64';
            }
          }
        }
      } catch (e) {
        // WebGL detection failed, default to x86_64
      }
    }
    render(os, arch);
  }
})();
</script>
