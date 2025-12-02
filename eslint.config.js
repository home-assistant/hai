import eslint from "@eslint/js";
import tseslint from "@typescript-eslint/eslint-plugin";
import tsparser from "@typescript-eslint/parser";
import litPlugin from "eslint-plugin-lit";
import prettierConfig from "eslint-config-prettier";

export default [
  eslint.configs.recommended,
  prettierConfig,
  {
    files: ["src/**/*.ts"],
    languageOptions: {
      parser: tsparser,
      parserOptions: {
        ecmaVersion: 2020,
        sourceType: "module",
      },
      globals: {
        window: "readonly",
        document: "readonly",
        console: "readonly",
        HTMLElement: "readonly",
        HTMLInputElement: "readonly",
        CustomEvent: "readonly",
      },
    },
    plugins: {
      "@typescript-eslint": tseslint,
      lit: litPlugin,
    },
    rules: {
      ...tseslint.configs.recommended.rules,
      "@typescript-eslint/no-unused-vars": [
        "error",
        { argsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/no-explicit-any": "warn",
      "lit/no-legacy-template-syntax": "error",
      "lit/binding-positions": "error",
      "lit/no-invalid-html": "error",
    },
  },
  {
    ignores: ["dist/", "node_modules/", "src-tauri/"],
  },
];
