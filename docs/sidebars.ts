import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  tutorialsSidebar: [
    {
      type: 'category',
      label: 'Tutorials',
      items: [
        'tutorials/getting-started',
        'tutorials/setup-environment',
        'tutorials/first-project',
        'tutorials/deploy-contract',
      ],
    },
  ],
  howToSidebar: [
    {
      type: 'category',
      label: 'How-To Guides',
      items: [
        'how-to-guides/interact-with-contracts',
        'how-to-guides/upgrade-contracts',
        'how-to-guides/optimize-gas',
        'how-to-guides/integrate-tools',
      ],
    },
  ],
  referenceSidebar: [
    {
      type: 'category',
      label: 'Reference',
      items: [
        'reference/api-docs',
        'reference/contract-interfaces',
        'reference/configuration',
        'reference/cli-commands',
        'reference/architecture',
      ],
    },
  ],
  explanationSidebar: [
    {
      type: 'category',
      label: 'Explanation',
      items: [
        'explanation/what-is-bonsol',
        'explanation/design-principles',
        'explanation/comparisons',
        'explanation/security',
        'explanation/roadmap',
      ],
    },
  ],
};

export default sidebars;
