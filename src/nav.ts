// Navigation structure for the book sidebar
// Each item has: title, href, and optional children

export interface NavItem {
  title: string;
  href: string;
}

export interface NavGroup {
  title: string;
  items: NavItem[];
}

export const navGroups: NavGroup[] = [
  {
    title: "Introduction",
    items: [
      { title: "Getting Started", href: "/docs/getting-started" },
      { title: "What is a Compiler?", href: "/docs/what-is-compiler" },
      { title: "Our Project Plan", href: "/docs/project-plan" },
    ],
  },
  {
    title: "Part 1 — Lexer",
    items: [
      { title: "What is a Lexer?", href: "/docs/lexer-intro" },
      { title: "Building the Lexer", href: "/docs/lexer-build" },
    ],
  },
  {
    title: "Part 2 — Parser",
    items: [
      { title: "What is Parsing?", href: "/docs/parser-intro" },
      { title: "Building the Parser", href: "/docs/parser-build" },
    ],
  },
  {
    title: "Part 3 — Semantic Analysis",
    items: [
      { title: "Semantic Analysis", href: "/docs/semantic" },
    ],
  },
  {
    title: "Part 4 — Code Generation",
    items: [
      { title: "Code Generation Intro", href: "/docs/codegen-intro" },
      { title: "Generating TypeScript", href: "/docs/codegen-typescript" },
    ],
  },
  {
    title: "Part 5 — Full Compiler",
    items: [
      { title: "Putting It All Together", href: "/docs/putting-together" },
    ],
  },
];

// Flat list used for prev/next navigation
export const allPages: NavItem[] = navGroups.flatMap((g) => g.items);
