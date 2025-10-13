import cloudflare from "@astrojs/cloudflare";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightImageZoom from "starlight-image-zoom";
import starlightLinksValidator from "starlight-links-validator";
import starlightLlmsTxt from "starlight-llms-txt";

// https://astro.build/config
export default defineConfig({
  redirects: {
    "/": "/start-here/getting-started",
  },
  adapter: cloudflare({
    imageService: "compile",
  }),
  site: "https://docs.kittynode.com",
  integrations: [
    starlight({
      plugins: [
        starlightLinksValidator(),
        starlightImageZoom(),
        starlightLlmsTxt(),
      ],
      title: "Kittynode Docs",
      editLink: {
        baseUrl: "https://github.com/futurekittylabs/kittynode/edit/main/docs/",
      },
      components: {
        Footer: "./src/components/overrides/Footer.astro",
        SiteTitle: "./src/components/overrides/SiteTitle.astro",
      },
      customCss: ["./src/styles/custom.css"],
      favicon: "/images/favicon.ico",
      head: [
        {
          tag: "link",
          attrs: {
            rel: "icon",
            type: "image/png",
            sizes: "16x16",
            href: "/images/favicon-16x16.png",
          },
        },
        {
          tag: "link",
          attrs: {
            rel: "icon",
            type: "image/png",
            sizes: "32x32",
            href: "/images/favicon-32x32.png",
          },
        },
        {
          tag: "link",
          attrs: {
            rel: "apple-touch-icon",
            sizes: "180x180",
            href: "/images/apple-touch-icon.png",
          },
        },
      ],
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/futurekittylabs/kittynode",
        },
        {
          icon: "discord",
          label: "Discord",
          href: "https://discord.kittynode.com",
        },
      ],
      sidebar: [
        {
          label: "Start Here",
          items: [
            {
              label: "Getting started",
              slug: "start-here/getting-started",
            },
          ],
        },
        {
          label: "Guides",
          items: [
            {
              label: "Run Ethereum",
              slug: "guides/run-ethereum",
            },
          ],
        },
        {
          label: "Reference",
          items: [
            {
              label: "Architecture",
              slug: "reference/architecture",
            },
            {
              label: "Build from source",
              slug: "reference/build-from-source",
            },
          ],
        },
      ],
    }),
  ],
});
