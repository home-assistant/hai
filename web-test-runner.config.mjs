import { esbuildPlugin } from "@web/dev-server-esbuild";

export default {
  files: "test/unit/**/*.test.ts",
  nodeResolve: true,
  plugins: [
    esbuildPlugin({
      ts: true,
      tsconfig: "./tsconfig.test.json",
    }),
  ],
  testFramework: {
    config: {
      ui: "bdd",
      timeout: 5000,
    },
  },
};
