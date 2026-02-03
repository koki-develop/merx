import { defineConfig } from "vitepress";
import { withMermaid } from "vitepress-plugin-mermaid";

// https://vitepress.dev/reference/site-config
export default withMermaid(
  defineConfig({
    title: "merx",
    description: "Run your flowcharts.",
    themeConfig: {
      // https://vitepress.dev/reference/default-theme-config
      nav: [
        { text: "Home", link: "/" },
        { text: "Getting Started", link: "/getting-started/installation" },
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
