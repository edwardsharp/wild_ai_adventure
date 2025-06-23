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

      const demoAsset = Object.values(bundle).find(
        (file) =>
          file.type === 'chunk' && file.fileName.includes('websocket-demo')
      );

      // Generate WebAuthn standalone HTML
      if (jsAsset && jsAsset.type === 'chunk') {
        const webauthnHtml = `<!DOCTYPE html>
  <html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>WebAuthn Component Test</title>
  </head>
  <body>
    <h1>🔐 WebAuthn Component Test</h1>
    <webauthn-auth base-url="http://localhost:8080" theme="dark"></webauthn-auth>

    <script type="module">
  PLACEHOLDER_JS_CODE
    </script>
  </body>
  </html>`;

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
</head>
<body>
  <h1>🔌 WebSocket Components Test</h1>
  <h2>Connection Status</h2>
  <websocket-status status="disconnected" showText="true" showUserCount="true"></websocket-status>
  <h2>WebSocket Handler</h2>
  <websocket-handler websocketUrl="ws://localhost:8080/ws" autoConnect="false" showDebugLog="true"></websocket-handler>

  <script type="module">`;

        const htmlAfter = `  </script>
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

      // Generate WebSocket Demo standalone HTML
      if (demoAsset && demoAsset.type === 'chunk') {
        const demoHtml = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>WebSocket Demo - Modular Components</title>
</head>
<body>
  <websocket-demo websocketUrl="ws://localhost:8080/ws" autoConnect="false" showDebugLog="true"></websocket-demo>

  <script type="module">
${demoAsset.code}
  </script>
</body>
</html>`;

        this.emitFile({
          type: 'asset',
          fileName: 'websocket-demo-standalone.html',
          source: demoHtml,
        });

        this.emitFile({
          type: 'asset',
          fileName: 'websocket-demo-standalone.js',
          source: demoAsset.code,
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
        'websocket-demo': './src/websocket-demo.tsx',
        'all-components': './src/index.tsx',
      },
      output: {
        entryFileNames: (chunkInfo) => {
          if (chunkInfo.name === 'webauthn') return 'webauthn-auth.js';
          if (chunkInfo.name === 'websocket') return 'websocket-components.js';
          if (chunkInfo.name === 'websocket-demo') return 'websocket-demo.js';
          if (chunkInfo.name === 'all-components') return 'all-components.js';
          return '[name].js';
        },
        chunkFileNames: '[name]-[hash].js',
        assetFileNames: '[name]-[hash].[ext]',
      },
    },
  },
});
