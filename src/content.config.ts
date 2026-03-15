// src/content.config.ts
// Defines the "docs" content collection using Astro's Content Collections API.
// This is the proper way to manage content in Astro — instead of putting
// markdown files directly in src/pages/, we define a collection here and
// let Astro handle loading, validation, and type-safety automatically.

import { defineCollection, z } from "astro:content";
import { glob } from "astro/loaders";

// Define the schema for a single doc page's frontmatter.
// zod validates that every markdown file has the required fields.
const docs = defineCollection({
  // The glob loader scans src/content/docs/ for all .md and .mdx files.
  loader: glob({ pattern: "**/*.{md,mdx}", base: "./src/content/docs" }),

  // Schema: each doc file must have title and description in its frontmatter.
  schema: z.object({
    title: z.string(),
    description: z.string().optional(),
  }),
});

// Export a named `collections` object — Astro picks this up automatically.
export const collections = { docs };
