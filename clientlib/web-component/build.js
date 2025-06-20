#!/usr/bin/env node

import { build } from 'esbuild';
import { readFileSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const pkg = JSON.parse(readFileSync(join(__dirname, 'package.json'), 'utf8'));

async function buildWebComponent() {
  console.log('🚀 Building WebAuthn Web Component...');

  try {
    // Build ESM bundle (external solid-js for when it's provided)
    await build({
      entryPoints: ['src/index.ts'],
      bundle: true,
      format: 'esm',
      outfile: 'dist/webauthn-auth.js',
      external: ['solid-js'],
      minify: true,
      sourcemap: true,
      target: 'es2022',
      platform: 'browser',
      define: {
        'process.env.NODE_ENV': '"production"',
      },
      banner: {
        js: `/* WebAuthn Web Component v${pkg.version} */`,
      },
    });

    // Build standalone bundle (includes solid-js)
    await build({
      entryPoints: ['src/index.ts'],
      bundle: true,
      format: 'iife',
      outfile: 'dist/webauthn-auth-standalone.js',
      globalName: 'WebAuthnAuth',
      minify: true,
      sourcemap: true,
      target: 'es2022',
      platform: 'browser',
      define: {
        'process.env.NODE_ENV': '"production"',
      },
      banner: {
        js: `/* WebAuthn Web Component v${pkg.version} - Standalone */`,
      },
    });

    // TypeScript declarations are handled by the manual .d.ts file below
    console.log('📝 Generating TypeScript declarations...');

    // Create a simple .d.ts file since we're not using tsc
    const dtsContent = `
declare module '@webauthn/web-component' {
  export interface WebAuthnAuthProps {
    baseUrl?: string;
    onLogin?: (username: string) => void;
    onLogout?: () => void;
    onError?: (error: string) => void;
    className?: string;
    theme?: 'light' | 'dark' | 'auto';
  }

  export default function WebAuthnAuth(props: WebAuthnAuthProps): any;
  export class WebAuthnAuthElement extends HTMLElement {}
  export function createWebAuthnAuth(container: Element, props: WebAuthnAuthProps): () => void;
  export const VERSION: string;
}

declare global {
  interface HTMLElementTagNameMap {
    'webauthn-auth': import('@webauthn/web-component').WebAuthnAuthElement;
  }
}

export {};
`;

    writeFileSync('dist/webauthn-auth.d.ts', dtsContent.trim());

    console.log('✅ Build completed successfully!');
    console.log('📦 Generated files:');
    console.log('  - dist/webauthn-auth.js (ESM bundle)');
    console.log('  - dist/webauthn-auth-standalone.js (IIFE bundle)');
    console.log('  - dist/webauthn-auth.d.ts (TypeScript declarations)');
  } catch (error) {
    console.error('❌ Build failed:', error);
    process.exit(1);
  }
}

// Run the build
buildWebComponent();
