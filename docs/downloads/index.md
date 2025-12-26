---
title: "Downloads"
layout: doc
---

# Downloads

Download <span class="operator-brand">Operator!</span> for your platform. Current version: **v{{ site.version }}**

> **Note:** <span class="operator-brand">Operator!</span> is currently available for **macOS and Linux only**. Windows support is not yet available.

<div id="recommended-download">
  <noscript>See the download tables below for all available platforms.</noscript>
</div>

## All <span class="operator-brand">Operator!</span> Downloads

| Platform | Architecture | Download |
|----------|--------------|----------|
| macOS | ARM64 (Apple Silicon) | [operator-macos-arm64]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-macos-arm64) |
| macOS | x86_64 (Intel) | [operator-macos-x86_64]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-macos-x86_64) |
| Linux | ARM64 | [operator-linux-arm64]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-linux-arm64) |
| Linux | x86_64 | [operator-linux-x86_64]({{ site.github.repo }}/releases/download/v{{ site.version }}/operator-linux-x86_64) |

## Backstage Server

Optional companion server for web-based project monitoring dashboard.

| Platform | Architecture | Download |
|----------|--------------|----------|
| macOS | ARM64 | [backstage-server-bun-darwin-arm64]({{ site.github.repo }}/releases/download/v{{ site.version }}/backstage-server-bun-darwin-arm64) |
| macOS | x64 | [backstage-server-bun-darwin-x64]({{ site.github.repo }}/releases/download/v{{ site.version }}/backstage-server-bun-darwin-x64) |
| Linux | ARM64 | [backstage-server-bun-linux-arm64]({{ site.github.repo }}/releases/download/v{{ site.version }}/backstage-server-bun-linux-arm64) |
| Linux | x64 | [backstage-server-bun-linux-x64]({{ site.github.repo }}/releases/download/v{{ site.version }}/backstage-server-bun-linux-x64) |

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
    if (os === 'windows') {
      container.innerHTML = '<div class="recommended-box"><p><strong>Windows detected</strong> - Operator is not yet available for Windows. Please use macOS or Linux.</p></div>';
      return;
    }

    var artifactName = 'operator-' + os + '-' + arch;
    var url = '{{ site.github.repo }}/releases/download/v{{ site.version }}/' + artifactName;
    var label = os === 'macos' ? 'macOS' : 'Linux';
    var archLabel = arch === 'arm64' ? (os === 'macos' ? 'Apple Silicon' : 'ARM64') : 'x86_64';

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
