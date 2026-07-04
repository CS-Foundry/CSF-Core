import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	optimizeDeps: {
		include: ['@xterm/xterm', '@xterm/addon-fit'],
	},
	server: {
		watch: {
			usePolling: true,
			interval: 1000,
		},
		proxy: {
			'/api': {
				target: process.env.VITE_API_URL || 'https://localhost:8000',
				changeOrigin: true,
				secure: false,
			},
		},
	},
});
