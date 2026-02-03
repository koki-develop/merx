import { defineConfig } from "vitepress";
import { withMermaid } from "vitepress-plugin-mermaid";

// https://vitepress.dev/reference/site-config
export default withMermaid(
  defineConfig({
    base: "/merx/",
    title: "merx",
    description: "Run your flowcharts.",
    themeConfig: {
      // https://vitepress.dev/reference/default-theme-config
      nav: [
        { text: "Home", link: "/" },
        { text: "Getting Started", link: "/getting-started/installation" },
        { text: "Guide", link: "/guide/program-structure" },
        { text: "Examples", link: "/examples" },
      ],

      sidebar: [
        {
          text: "Getting Started",
          items: [
            { text: "Installation", link: "/getting-started/installation" },
            { text: "Quick Start", link: "/getting-started/quick-start" },
          ],
        },
        {
          text: "Guide",
          items: [
            {
              text: "Program Structure",
              link: "/guide/program-structure",
            },
            { text: "Nodes and Edges", link: "/guide/nodes-and-edges" },
            { text: "Output", link: "/guide/output" },
            {
              text: "Variables and Types",
              link: "/guide/variables-and-types",
            },
            { text: "Operators", link: "/guide/operators" },
            { text: "Control Flow", link: "/guide/control-flow" },
            { text: "Input", link: "/guide/input" },
          ],
        },
        {
          text: "Examples",
          link: "/examples",
        },
      ],

      socialLinks: [
        { icon: "github", link: "https://github.com/koki-develop/merx" },
      ],
    },
  }),
);
