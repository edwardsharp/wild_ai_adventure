import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';

function inlineHtmlTemplate(): import('vite').Plugin {
  return {
    name: 'generate-html-templates',
    generateBundle(_, bundle) {
      const jsAsset = Object.values(bundle).find(
        (file) => file.type === 'chunk' && file.fileName.includes('webauthn')
      );

      const wsAsset = Object.values(bundle).find(
        (file) => file.type === 'chunk' && file.fileName.includes('websocket')
      );

      const allAsset = Object.values(bundle).find(
        (file) =>
          file.type === 'chunk' && file.fileName.includes('all-components')
      );

      // Generate WebAuthn standalone HTML
      if (jsAsset && jsAsset.type === 'chunk') {
        const webauthnHtml = [
          '<!DOCTYPE html>',
          '<html lang="en">',
          '<head>',
          '  <meta charset="UTF-8" />',
          '  <meta name="viewport" content="width=device-width, initial-scale=1.0" />',
          '  <title>WebAuthn Component Test</title>',
          '  <style>',
          '    body {',
          '      font-family: sans-serif;',
          '      padding: 2rem;',
          '      background: black;',
          '      color: white;',
          '    }',
          '    .container {',
          '      max-width: 800px;',
          '      margin: 0 auto;',
          '    }',
          '  </style>',
          '</head>',
          '<body>',
          '  <div class="container">',
          '    <h1>🔐 WebAuthn Component Test</h1>',
          '    <webauthn-auth',
          '      base-url="http://localhost:8080"',
          '      theme="dark">',
          '    </webauthn-auth>',
          '',
          '    <h2>Links</h2>',
          '    <ul>',
          '      <li><a href="/public/welcome.html">Public Welcome</a></li>',
          '      <li><a href="/private/dashboard.html">Private Dashboard</a></li>',
          '    </ul>',
          '  </div>',
          '',
          '  <script type="module">',
          'PLACEHOLDER_JS_CODE',
          '  </script>',
          '',
          '  <script>',
          '    // Listen for WebAuthn events',
          '    const webauthnElement = document.querySelector("webauthn-auth");',
          '    if (webauthnElement) {',
          '      webauthnElement.addEventListener("webauthn-login", (e) => {',
          '        console.log("✅ Login successful:", e.detail.username);',
          '      });',
          '',
          '      webauthnElement.addEventListener("webauthn-logout", (e) => {',
          '        console.log("👋 User logged out");',
          '      });',
          '',
          '      webauthnElement.addEventListener("webauthn-error", (e) => {',
          '        console.error("❌ Auth error:", e.detail.error);',
          '      });',
          '    }',
          '  </script>',
          '</body>',
          '</html>',
        ].join('\n');

        this.emitFile({
          type: 'asset',
          fileName: 'webauthn-auth-standalone.html',
          source: webauthnHtml.replace('PLACEHOLDER_JS_CODE', jsAsset.code),
        });

        this.emitFile({
          type: 'asset',
          fileName: 'webauthn-auth-standalone.js',
          source: jsAsset.code,
        });
      }

      // Generate WebSocket standalone HTML and JS (safe approach without string replacement)
      if (wsAsset && wsAsset.type === 'chunk') {
        const htmlBefore = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>WebSocket Components Test</title>
  <style>
    body { font-family: sans-serif; padding: 2rem; background: #f5f5f5; }
    .container { max-width: 1200px; margin: 0 auto; }
  </style>
</head>
<body>
  <div class="container">
    <h1>🔌 WebSocket Components Test</h1>
    <div style="margin-bottom: 2rem;">
      <h2>Connection Status</h2>
      <websocket-status status="disconnected" showText="true" showUserCount="true"></websocket-status>
    </div>
    <div>
      <h2>WebSocket Handler</h2>
      <websocket-handler websocketUrl="ws://localhost:8080/ws" autoConnect="false" showDebugLog="true"></websocket-handler>
    </div>
  </div>
  <script type="module">`;

        const htmlAfter = `  </script>
  <script>
    document.addEventListener("status-change", (e) => console.log("Status:", e.detail));
    document.addEventListener("media-blobs-received", (e) => console.log("Blobs:", e.detail));
  </script>
</body>
</html>`;

        this.emitFile({
          type: 'asset',
          fileName: 'websocket-components-standalone.html',
          source: htmlBefore + '\n' + wsAsset.code + '\n' + htmlAfter,
        });

        this.emitFile({
          type: 'asset',
          fileName: 'websocket-components-standalone.js',
          source: wsAsset.code,
        });
      }

      // Generate all components HTML
      if (allAsset && allAsset.type === 'chunk') {
        this.emitFile({
          type: 'asset',
          fileName: 'all-components-standalone.js',
          source: allAsset.code,
        });
      }

      console.log('✅ Generated standalone files for all available components');
    },
  };
}

export default defineConfig({
  plugins: [solid(), inlineHtmlTemplate()],
  build: {
    outDir: 'dist',
    target: 'esnext',
    minify: true,
    sourcemap: true,
    rollupOptions: {
      input: {
        webauthn: './src/webauthn-component.tsx',
        websocket: './src/websocket-handler.tsx',
        'all-components': './src/index.tsx',
      },
      output: {
        entryFileNames: (chunkInfo) => {
          if (chunkInfo.name === 'webauthn') return 'webauthn-auth.js';
          if (chunkInfo.name === 'websocket') return 'websocket-components.js';
          if (chunkInfo.name === 'all-components') return 'all-components.js';
          return '[name].js';
        },
        chunkFileNames: '[name]-[hash].js',
        assetFileNames: '[name]-[hash].[ext]',
      },
    },
  },
});
