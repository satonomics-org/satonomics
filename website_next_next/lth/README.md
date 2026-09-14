# Long Term Holders Cloud

Open `index.html` directly in a browser. Styles, chart code, and data are embedded; fonts and the chart library load from a CDN.

The cloud is the daily minimum and maximum of four LTH-weighted prices:

- Cointime-weighted realized price (`lth_awake_price_cents`)
- Cointime-weighted capitalized price (`lth_awake_capitalized_price_cents`)
- Coinflow-weighted realized price (`lth_coinflow_price_cents`)
- Coinflow-weighted capitalized price (`lth_coinflow_capitalized_price_cents`)

The page shows Bitcoin price and the cloud, with candles when zoomed in and no trend lines. Its fuchsia accent matches `colors.term.long` in `website/`.

With the updated backend running on localhost:3110, refresh from this folder:

```sh
node generate.mjs
```

Optional arguments: `--api http://localhost:3110/api` and `--start 2011-02-20`. The default start is the first day with positive values for all four LTH sources; earlier zero prices cannot form this logarithmic cloud.

The generator includes today's partial data and validates all four inputs before atomically replacing the embedded snapshot. Shared validation and snapshot-writing code lives in `scripts/cloud-snapshot.mjs`.
