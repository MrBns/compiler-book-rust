import { defineConfig } from "astro/config";
import mdx from "@astrojs/mdx";

export default defineConfig({
  integrations: [mdx()],
  output: "static",
  // Disable the Astro dev toolbar.
  // The toolbar injects a client script whose Vite-generated source map
  // reference points to a file that is never served, producing a spurious
  // 404 in the browser console during `astro dev`.
  // A documentation site has no need for the toolbar, so we turn it off.
  devToolbar: {
    enabled: false,
  },
  markdown: {
    shikiConfig: {
      theme: "github-dark",
      langs: ["rust", "typescript", "go", "bash", "toml", "json"],
    },
  },
});
