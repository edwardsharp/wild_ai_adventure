import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';

function inlineHtmlTemplate(): import('vite').Plugin {
  return {
    name: 'generate-webauthn-html',
    generateBundle(_, bundle) {
      const jsAsset = Object.values(bundle).find(
        (file) => file.type === 'chunk' && file.fileName.endsWith('.js')
      );

      if (!jsAsset || jsAsset.type !== 'chunk') {
        console.error('JS asset not found');
        return;
      }

      const templateHtml = `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>WebAuthn Component Test</title>
  <style>
    body {
      font-family: sans-serif;
      padding: 2rem;
      background: black;
      color: white;
    }
    .container {
      max-width: 800px;
      margin: 0 auto;
    }
  </style>
</head>
<body>
  <div class="container">
    <h1>🔐 WebAuthn Component Test</h1>
    <webauthn-auth
      base-url="http://localhost:8080"
      theme="dark">
    </webauthn-auth>

    <h2>Links</h2>
    <ul>
      <li><a href="/public/welcome.html">Public Welcome</a></li>
      <li><a href="/private/dashboard.html">Private Dashboard</a></li>
    </ul>
  </div>

  <script type="module">
\${jsCode}
  </script>

  <script>
    // Listen for WebAuthn events
    const webauthnElement = document.querySelector('webauthn-auth');
    if (webauthnElement) {
      webauthnElement.addEventListener('webauthn-login', (e) => {
        console.log('✅ Login successful:', e.detail.username);
      });

      webauthnElement.addEventListener('webauthn-logout', (e) => {
        console.log('👋 User logged out');
      });

      webauthnElement.addEventListener('webauthn-error', (e) => {
        console.error('❌ Auth error:', e.detail.error);
      });
    }
  </script>
</body>
</html>
      `.trim();

      this.emitFile({
        type: 'asset',
        fileName: 'webauthn-auth-standalone.html',
        source: templateHtml.replace('${jsCode}', jsAsset.code),
      });

      // Also emit just the JS file
      this.emitFile({
        type: 'asset',
        fileName: 'webauthn-auth-standalone.js',
        source: jsAsset.code,
      });

      console.log(
        '✅ Emitted: webauthn-auth-standalone.html and webauthn-auth-standalone.js'
      );
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
        main: './src/webauthn-component.tsx',
      },
      output: {
        entryFileNames: 'webauthn-auth.js',
        chunkFileNames: 'webauthn-auth-[hash].js',
        assetFileNames: 'webauthn-auth-[hash].[ext]',
      },
    },
  },
});
