import js from '@eslint/js'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import globals from 'globals'
import tseslint from 'typescript-eslint'
import eslintReact from '@eslint-react/eslint-plugin'
import prettierConfig from 'eslint-config-prettier'

export default tseslint.config(
  {
    ignores: [
      'dist/**',
      'src-tauri/**',
      '.superpowers/**',
      '.tools/**',
      'src/lib/bindings.ts',
      'coverage/**',
      'jscpd-report/**',
    ],
  },
  { ...eslintReact.configs['recommended-typescript'], files: ['src/**/*.{ts,tsx}'] },
  {
    files: ['**/*.{ts,tsx}'],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    languageOptions: { globals: globals.browser },
    plugins: { 'react-hooks': reactHooks, 'react-refresh': reactRefresh },
    rules: {
      ...reactHooks.configs.recommended.rules,
      '@eslint-react/naming-convention-ref-name': 'off',
      '@eslint-react/use-state': 'off',
      '@eslint-react/no-context-provider': 'off',
      '@eslint-react/no-use-context': 'off',
      'react-refresh/only-export-components': [
        'error',
        { allowConstantExport: true, allowExportNames: ['useTheme', 'buttonVariants'] },
      ],
    },
  },
  {
    files: ['vite.config.ts', 'vitest.config.ts', 'eslint.config.js'],
    languageOptions: { globals: globals.node },
  },
  {
    files: ['scripts/**/*.{mjs,js}'],
    extends: [js.configs.recommended],
    languageOptions: { globals: globals.node },
  },
  {
    files: ['tests/fixtures/**/*.js'],
    extends: [js.configs.recommended],
    languageOptions: { globals: globals.browser },
  },
  {
    files: ['src/**/*.test.{ts,tsx}', 'src/test/**'],
    rules: { 'react-refresh/only-export-components': 'off' },
  },
  {
    files: ['src/components/ui/**/*.{ts,tsx}'],
    rules: { '@eslint-react/no-array-index-key': 'off' },
  },
  prettierConfig,
)
