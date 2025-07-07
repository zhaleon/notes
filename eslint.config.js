// eslint.config.js for ESLint v9+ and SvelteKit
import svelte from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import babelParser from '@babel/eslint-parser';
export default [
  {
    ignores: [
      '.svelte-kit/',
      'node_modules/',
      'build/',
      'dist/',
      'src-tauri/target/',
      'src-tauri/release/',
    ],
  },
  // JavaScript files
  {
    files: ['src/**/*.js', 'routes/**/*.js'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      parser: babelParser,
      parserOptions: {
        requireConfigFile: false,
      },
    },
    rules: {},
  },
  // Svelte component files
  {
    files: ['src/**/*.svelte', 'routes/**/*.svelte'],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        extraFileExtensions: ['.svelte'],
      },
    },
    plugins: { svelte },
    rules: {},
  }
];
