# Downloads

> Download Operator! binaries for macOS, Linux, and Windows.

Download <span class="operator-brand">Operator!</span> for your platform. Current version: **v0.2.17**

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
| macOS | ARM64 (Apple Silicon) | [operator-macos-arm64](https://github.com/untra/operator/releases/download/v0.2.17/operator-macos-arm64)<br><small>sha256:<code>1da5e5e995a441a9e9d5d2bd68d7685b49838e59f885bbdbc51f6ba7a13d310b</code></small> |
| Linux | ARM64 | [operator-linux-arm64](https://github.com/untra/operator/releases/download/v0.2.17/operator-linux-arm64)<br><small>sha256:<code>9c68afa65550dfdae1062b2acb87c39680a95266b6291ebf8d72d8613c357e31</code></small> |
| Linux | x86_64 | [operator-linux-x86_64](https://github.com/untra/operator/releases/download/v0.2.17/operator-linux-x86_64)<br><small>sha256:<code>2bd742d8e5cd1ee6f681d78b79321406478aa89102c6b820331391347f5bb191</code></small> |
| Windows | x86_64 | [operator-windows-x86_64.exe](https://github.com/untra/operator/releases/download/v0.2.17/operator-windows-x86_64.exe)<br><small>sha256:<code>628886e32587e2a2c6d49328d0522edd142f5d8469a8b6a73eda7274ef35b6bb</code></small> |
| Windows | ARM64 | [operator-windows-arm64.exe](https://github.com/untra/operator/releases/download/v0.2.17/operator-windows-arm64.exe)<br><small>sha256:<code>e89d31a398a350d0d45fdbd7441e4e159f47b36bed7d4cf12618760c68035dba</code></small> |

### opr8r step wrapper

`opr8r` runs beside each agent CLI on the machine where agents execute. It reports step completion to Operator and starts the next workflow step. The container image and the VS Code extension already include it, so you only need these downloads for a hand-built execution host. A machine that only serves models, such as a gaming PC running [LM Studio](/getting-started/model-servers/lmstudio/), does not need `opr8r`. See [platform support](/getting-started/platform-support/) for which operating systems can be execution hosts.

| Platform | Architecture | Download |
|----------|--------------|----------|
| macOS | ARM64 (Apple Silicon) | [opr8r-macos-arm64](https://github.com/untra/operator/releases/download/v0.2.17/opr8r-macos-arm64)<br><small>sha256:<code>8d99d8e928846d32c47131837da0558e102a8436adfa77102abf566b5a577c94</code></small> |
| Linux | ARM64 | [opr8r-linux-arm64](https://github.com/untra/operator/releases/download/v0.2.17/opr8r-linux-arm64)<br><small>sha256:<code>d57716ec170848962267cbc5031373eae1a63566f73c6799e08022fefc552bc2</code></small> |
| Linux | x86_64 | [opr8r-linux-x86_64](https://github.com/untra/operator/releases/download/v0.2.17/opr8r-linux-x86_64)<br><small>sha256:<code>88fbd0f3b313dd8e8d1ebd3b7b260ce543b5985c5e05f6a97028fd890367b0ca</code></small> |
| Windows | x86_64 | [opr8r-windows-x86_64.exe](https://github.com/untra/operator/releases/download/v0.2.17/opr8r-windows-x86_64.exe)<br><small>sha256:<code>996f1b8ec16d4a6de6f870c85bb792322e35ea99ac33bc3fab6f9804550d37e2</code></small> |
| Windows | ARM64 | [opr8r-windows-arm64.exe](https://github.com/untra/operator/releases/download/v0.2.17/opr8r-windows-arm64.exe)<br><small>sha256:<code>53c2c97a7c14db6f27d135650abe3031aa83448521db19c5f91b9e25cc750f38</code></small> |

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
docker pull untra/operator:0.2.17
```

Multi-arch (`linux/amd64`, `linux/arm64`). See the [Docker guide](/getting-started/platforms/docker/).

## All Releases

[View all releases on GitHub](https://github.com/untra/operator/releases)

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
      url = 'https://github.com/untra/operator/releases/download/v0.2.17/' + artifactName;
      label = 'Windows';
      archLabel = arch === 'arm64' ? 'ARM64' : 'x86_64';
    } else {
      artifactName = 'operator-' + os + '-' + arch;
      url = 'https://github.com/untra/operator/releases/download/v0.2.17/' + artifactName;
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
