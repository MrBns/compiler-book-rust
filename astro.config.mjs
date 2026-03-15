import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";

export default defineConfig({
  integrations: [mdx()],
  output: "static",
  markdown: {
    shikiConfig: {
      theme: "github-dark",
      langs: ["rust", "typescript", "go", "bash", "toml", "json"],
    },
  },
});
