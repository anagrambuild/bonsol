import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import { themes } from 'prism-react-renderer';

const config: Config = {
  title: 'Bonsol Documentation',
  tagline: 'Zk plubing on solana',
  favicon: 'img/favicon.ico',

  // Set the production url of your site here
  url: 'https://doc.bonsol.sh',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'anagrambuild', // Usually your GitHub org/user name.
  projectName: 'bonsol', // Usually your repo name.

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl:
            'https://github.com/anagrambuild/bonsol/tree/main/docs/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      // Replace with your project's social card
      image: 'img/docusaurus-social-card.jpg',
      navbar: {
        title: 'Bonsol Docs',
        logo: {
          alt: 'Bonsol Logo',
          src: 'img/logo.svg',
        },
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'tutorialsSidebar',
            position: 'left',
            label: 'Tutorials',
          },
          {
            type: 'docSidebar',
            sidebarId: 'howToSidebar',
            position: 'left',
            label: 'How-To Guides',
          },
          {
            type: 'docSidebar',
            sidebarId: 'referenceSidebar',
            position: 'left',
            label: 'Reference',
          },
          {
            type: 'docSidebar',
            sidebarId: 'explanationSidebar',
            position: 'left',
            label: 'Explanation',
          },
          {
            href: 'https://github.com/anagrambuild/bonsol',
            label: 'GitHub',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              {
                label: 'Tutorials',
                to: '/docs/category/tutorials',
              },
              {
                label: 'How-To Guides',
                to: '/docs/category/how-to-guides',
              },
              {
                label: 'Reference',
                to: '/docs/category/reference',
              },
              {
                label: 'Explanation',
                to: '/docs/category/explanation',
              },
            ],
          },
          {
            title: 'Community',
            items: [
              {
                label: 'Stack Overflow',
                href: 'https://stackoverflow.com/questions/tagged/bonsol',
              },
              {
                label: 'Discord',
                href: 'https://discordapp.com/invite/bonsol',
              },
              {
                label: 'Twitter',
                href: 'https://twitter.com/bonsol',
              },
            ],
          },
          {
            title: 'More',
            items: [
              {
                label: 'GitHub',
                href: 'https://github.com/anagrambuild/bonsol',
              },
            ],
          },
        ],
        copyright: `Copyright © ${new Date().getFullYear()} Bonsol Project. Built by Anagram, Built with Docusaurus.`,
      },
      prism: {
        theme: themes.oneLight,
        darkTheme: themes.nightOwl,
      },
    }),
};

export default config;
