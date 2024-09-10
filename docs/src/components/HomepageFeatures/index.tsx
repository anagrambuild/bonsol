import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Prove Anything',
    Svg: require('@site/static/img/prove.svg').default,
    description: (
      <>
        With Bonsol you can prove anything, from simple computations to complex business logic.
        Using state of the art zk(risc0) technology, Bonsol allows you to prove computations that are impossible to run on-chain.
      </>
    ),
  },
  {
    title: 'Solana Integrated',
    Svg: require('@site/static/img/sol.svg').default,
    description: (
      <>
        Bonsol can integrate with any solana program, allowing you to take action after a proof is verified.
        String on and off chain computations together with ease.
      </>
    ),
  },
  {
    title: 'Batteries Included',
    Svg: require('@site/static/img/bat.svg').default,
    description: (
      <>
        Anchor integrated, CLI, SDKs will help you get up and running quickly.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
