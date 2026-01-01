import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import { fileURLToPath } from 'url'

// Fix for ESM: Define __dirname manually
const __dirname = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      // This tells Vite: "When you see 'linera-wasm', look at this specific file path"
      'linera-wasm': path.resolve(__dirname, 'node_modules/@linera/client/linera_client_bg.wasm')
    }
  },
  optimizeDeps: {
    // Exclude the library from pre-bundling so Vite doesn't try to mess with its paths
    exclude: ['@linera/client']
  }
})