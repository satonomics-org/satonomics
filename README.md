# SATONOMICS

![Image of the Satonomics Web App](./assets/latest.jpg)

## Description

Satonomics is a better, FOSS, Bitcoin-only, self-hostable Glassnode.

While [mempool.space](https://mempool.space) gives a very micro view of the network where you can follow the journey of any address, this tool is the exact opposite and very complimentary by giving you a much more global/macro view of the flow and various dynamics of the network via thousands of charts.

To promote even more transparency and trust in the network, this project is committed to making on-chain data accessible and verifiable by all, no matter your intentions or financial situation. That is why, the whole project is completely free, from code to services, including a real-time API with thousands and thousands of routes which can be used at will.

**Having anyone be able to easily do a health-check of the network is incredibly important and should be wanted by every single bitcoiner.**

## Warning

This project is in a very early stage. The web app will have bugs, the API might break and the data can definitely to be false or slightly false.

## Instances

Web App:

- [app.satonomics.xyz](https://app.satonomics.xyz)

API:

- [api.satonomics.xyz](https://api.satonomics.xyz)

## Structure

- `parser`: The backbone of the project, it does most of the work by parsing and then computing datasets from the timechain.
- `server`: A small server which automatically creates routes to access through an API all created datasets.
- `app`: A web app which displays the generated datasets in various charts and dashboards.

## Goals / Philosophy

Adjectives that describe what this project is or strives to be:

- **Best**: Be the go to and replace Glassnode
- **Diverse**: Have as many charts/datasets as possible and something for everyone
- **Free**: Is and always will be completely free
- **Independent**: Only one, easily swappable, dependency (Price API)
- **Sovereign**: Be self-hostable on accessible hardware
- **Timeless**: Be relevant and usable 10 years from now

## Milestones

Big features that are planned, in no particular order:

- **More Datasets/Charts**: If a dataset can be computed, it should exist and have its related charts
- **Dashboards**: For a quick and real-time view of the latest data of all the datasets
- **NOSTR integration**: First to save preferences, later to add some social functionnality
- **Datasets by block timestamp**: In addition to having datasets by block date and block height
- **Homepage**: A landing page to explain the project and what it does

_Maybe_:

- A Desktop app
- A mobile app

## Brand

- **Name**: Willing to change if someone thinks of something better !
- **Logo**: Most likely a placeholder

## Git

- Repositories:
  - [Github](https://github.com/satonomics-org/satonomics)
  - [Codeberg](https://codeberg.org/satonomics/satonomics)
- Issues:
  - [Github](https://github.com/satonomics-org/satonomics/issues)
  - [NOSTR](https://gitworkshop.dev/r/naddr1qq99xct5dahx7mtfvdesz9thwden5te0wp6hyurvv4ex2mrp0yhxxmmdqgsfw5dacngjlahye34krvgz7u0yghhjgk7gxzl5ptm9v6n2y3sn03srqsqqqaueek2h03/issues)
- Proposals:
  - [Github](https://github.com/satonomics-org/satonomics/pulls)
  - [NOSTR](https://gitworkshop.dev/r/naddr1qq99xct5dahx7mtfvdesz9thwden5te0wp6hyurvv4ex2mrp0yhxxmmdqgsfw5dacngjlahye34krvgz7u0yghhjgk7gxzl5ptm9v6n2y3sn03srqsqqqaueek2h03/proposals)

## Proof of Work

Aka: Previous iterations

The initial idea was totally different yet morphed over time into what it is today: a fully FOSS self-hostable on-chain data generator

- https://github.com/drgarlic/satonomics
- https://github.com/drgarlic/satonomics-parser
- https://github.com/drgarlic/satonomics-explorer
- https://github.com/drgarlic/satonomics-server
- https://github.com/drgarlic/satonomics-app
- https://github.com/drgarlic/bitalisys
- https://github.com/drgarlic/bitesque-app
- https://github.com/drgarlic/bitesque-back
- https://github.com/drgarlic/bitesque-front
- https://github.com/drgarlic/bitesque-assets
- https://github.com/drgarlic/syf
