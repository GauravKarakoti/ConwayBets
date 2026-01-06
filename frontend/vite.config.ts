import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import fs from 'fs'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

// 1. Helper to find the absolute path to the .wasm file
function findLineraWasmPath() {
  const packageLocations = [
    path.resolve(__dirname, 'node_modules/@linera/client'),       // Local
    path.resolve(__dirname, '../node_modules/@linera/client'),    // Hoisted
    path.resolve(__dirname, '../linera/node_modules/@linera/client') 
  ];

  const relativePaths = [
    'dist/linera_bg.wasm',       // Found in your previous logs
    'linera_client_bg.wasm',
    'pkg/linera_client_bg.wasm',
    'dist/linera_client_bg.wasm',
    'linera_bg.wasm'
  ];

  for (const pkgDir of packageLocations) {
    if (!fs.existsSync(pkgDir)) continue;

    // Check specific known paths
    for (const relPath of relativePaths) {
      const fullPath = path.join(pkgDir, relPath);
      if (fs.existsSync(fullPath)) return fullPath;
    }

    // Fallback: Scan directory (wrapped in try/catch for older Node versions)
    try {
      const files = fs.readdirSync(pkgDir, { recursive: true }) as string[];
      const wasmFile = files.find((f: string) => f.endsWith('.wasm'));
      if (wasmFile) return path.join(pkgDir, wasmFile);
    } catch (e) {}
  }
  return null;
}

// 2. Custom Plugin to resolve the ID
function lineraWasmPlugin() {
  const wasmPath = findLineraWasmPath();
  
  if (wasmPath) {
    console.log(`[Vite Config] ✅ Linera WASM located: ${wasmPath}`);
  } else {
    console.error('[Vite Config] ❌ CRITICAL: Linera WASM file not found.');
  }

  return {
    name: 'resolve-linera-wasm',
    resolveId(id: string) {
      // Intercept imports for "linera-wasm" or "linera-wasm?url"
      if (id === 'linera-wasm?url' || id === 'linera-wasm') {
        if (!wasmPath) throw new Error('Linera WASM file not found');
        
        // If the import asked for ?url, append it to the absolute path
        // This tells Vite's asset plugin to handle it as a static asset URL
        if (id.endsWith('?url')) {
          return wasmPath + '?url';
        }
        return wasmPath;
      }
    }
  }
}

export default defineConfig({
  resolve: {
    alias: {
      // ⚠️ IMPORTANT: Force the JS import to match the WASM location.
      '@linera/client': path.resolve(__dirname, 'node_modules/@linera/client/dist/linera.js'),
      '@linera/client-index': path.resolve(__dirname, 'node_modules/@linera/client/dist/index.js')
    }
  },
  plugins: [
    react(), 
    lineraWasmPlugin() // Register the custom resolver
  ],
  optimizeDeps: {
    // Keep this to prevent Vite from trying to bundle the WASM as JS
    exclude: ['@linera/client']
  },
  server: {
    headers: {
      "Cross-Origin-Embedder-Policy": "credentialless",
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Resource-Policy": "cross-origin",
    },
    fs: {
      allow: ['..']
    }
  },
  build: {
    target: 'esnext'
  },
  esbuild: {
    supported: {
      "top-level-await": true,
    },
  },
})