import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { visualizer } from 'rollup-plugin-visualizer';

export default defineConfig({
    plugins: [
        sveltekit(),
        visualizer({
            filename: 'build/stats.html',
            gzipSize: true,
            brotliSize: true,
        }),
    ],
    // Tauri expects a fixed port, fail if that port is not available
    server: {
        port: 1420,
        strictPort: true,
        watch: {
            // 3. tell vite to ignore watching `src-tauri`
            ignored: ['**/src-tauri/**'],
        },
    },
    build: {
        sourcemap: true,
        assetsInlineLimit: 0,
        chunkSizeWarningLimit: 1700,
        rollupOptions: {
            output: {
                manualChunks(id) {
                    if (id.includes('node_modules')) {
                        // Group large vendor libraries into a separate chunk
                        if (
                            id.includes('highlight.js') ||
                            id.includes('marked') ||
                            id.includes('moment')
                        ) {
                            return 'vendor_ui';
                        }

                        if (
                            id.includes('openai') ||
                            id.includes('ollama') ||
                            id.includes('@google/genai')
                        ) {
                            return 'vendor_ai';
                        }

                        // Group other node_modules into a general vendor chunk
                        return 'vendor';
                    }
                },
            },
        },
    },
});
