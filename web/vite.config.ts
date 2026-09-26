import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
// import monacoEditorPlugin from 'vite-plugin-monaco-editor-esm';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter()
		}),
		// monacoEditorPlugin({
			// languageWorkers: ['editorWorkerService'], // add 'typescript', 'json', etc. if you need those language services
		// })
	],
});
// vite.config.ts
