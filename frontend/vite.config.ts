import { defineConfig } from 'vite';
import { qwikVite } from '@builder.io/qwik/optimizer';
import { qwikCity } from '@builder.io/qwik-city/vite';
import tsconfigPaths from 'vite-tsconfig-paths';
import { match } from 'assert';

export default defineConfig(async () => {
  const { default: rehypePrettyCode } = await import("rehype-pretty-code");
  return {
    plugins: [qwikCity({
      mdxPlugins: {
        remarkGfm: true,
        rehypeSyntaxHighlight: true,
        rehypeAutolinkHeadings: true
      },
      mdx: {
        rehypePlugins: [
          [
            rehypePrettyCode,
            {
              theme: "rose-pine-moon",
              lineNumbers: true,

            }
          ],
        ],
      },
      trailingSlash: false,
    }), qwikVite(), tsconfigPaths()],
    preview: {
      headers: {
        'Cache-Control': 'public, max-age=600',
      },
    },
  };
});
