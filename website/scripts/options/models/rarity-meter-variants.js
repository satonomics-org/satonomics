import { bitview } from "../../utils/client.js";

/** Build each meter's source list to match the Rarity Meter plugin. */
export function createRarityMeterVariants() {
  const { rarityMeter, cohorts, cointime, coinflow, bedrock } = bitview.series;
  const references = rarityMeter.referencePrices;
  const components = rarityMeter.components;
  const realized = cohorts.realized;
  const sources = {
    under4mRealizedPrice: { name: "<4M Realized Price", series: references.under4m },
    under6mRealizedPrice: { name: "<6M Realized Price", series: references.under6m },
    under4mCapitalizedPrice: { name: "<4M Capitalized Price", series: references.under4mCapitalizedPrice },
    under6mCapitalizedPrice: { name: "<6M Capitalized Price", series: references.under6mCapitalizedPrice },
    sthRealizedPrice: { name: "STH Realized Price", series: realized.price.term.short },
    sthCapitalizedPrice: { name: "STH Capitalized Price", series: realized.capitalizedPrice.sth },
    sthMedianPriceBtcWeighted: { name: "STH Median (BTC Weighted)", series: components.sthMedianPriceBtcWeighted },
    sthMedianPriceUsdWeighted: { name: "STH Median (USD Weighted)", series: components.sthMedianPriceUsdWeighted },
    over4mRealizedPrice: { name: ">4M Realized Price", series: references.over4m },
    over6mRealizedPrice: { name: ">6M Realized Price", series: references.over6m },
    realizedPrice: { name: "Realized Price", series: realized.price.all },
    capitalizedPrice: { name: "Capitalized Price", series: realized.capitalizedPrice.all },
    lthRealizedPrice: { name: "LTH Realized Price", series: realized.price.term.long },
    lthCapitalizedPrice: { name: "LTH Capitalized Price", series: realized.capitalizedPrice.lth },
    medianPriceBtcWeighted: { name: "Median (BTC Weighted)", series: components.medianPriceBtcWeighted },
    medianPriceUsdWeighted: { name: "Median (USD Weighted)", series: components.medianPriceUsdWeighted },
    cointimeMedianPriceBtcWeighted: { name: "Cointime Median (BTC Weighted)", series: components.cointimeMedianPriceBtcWeighted },
    cointimeMedianPriceUsdWeighted: { name: "Cointime Median (USD Weighted)", series: components.cointimeMedianPriceUsdWeighted },
    coinflowMedianPriceBtcWeighted: { name: "Coinflow Median (BTC Weighted)", series: components.coinflowMedianPriceBtcWeighted },
    coinflowMedianPriceUsdWeighted: { name: "Coinflow Median (USD Weighted)", series: components.coinflowMedianPriceUsdWeighted },
    vaultedPrice: { name: "Vaulted Price", series: cointime.prices.vaulted },
    activePrice: { name: "Active Price", series: cointime.prices.active },
    trueMarketMeanPrice: { name: "True Market Mean", series: cointime.prices.trueMarketMean },
    cointimePrice: { name: "Cointime Price", series: cointime.prices.cointime },
    awakePrice: { name: "Awake Price", series: cointime.awake.price },
    coinflowPrice: { name: "Coinflow Price", series: coinflow.price },
  };
  const floors = [
    { name: "Raw", tree: bedrock.raw.floor },
    { name: "Cointime", tree: bedrock.cointime.floor },
    { name: "Coinflow", tree: bedrock.coinflow.floor },
  ].flatMap((model) =>
    /** @type {const} */ ([
      { key: "pct999", name: "P99.9" },
      { key: "pct995", name: "P99.5" },
      { key: "pct99", name: "P99" },
      { key: "pct98", name: "P98" },
      { key: "pct95", name: "P95" },
    ]).map((percentile) => ({
      name: `${model.name} Bedrock ${percentile.name}`,
      series: model.tree[percentile.key],
      style: /** @type {const} */ (2),
    })),
  );
  const local = [
    sources.under4mRealizedPrice,
    sources.under6mRealizedPrice,
    sources.sthRealizedPrice,
    sources.sthCapitalizedPrice,
  ];
  const cycle = [
    sources.over4mRealizedPrice,
    sources.over6mRealizedPrice,
    sources.realizedPrice,
    sources.capitalizedPrice,
    sources.lthRealizedPrice,
    sources.lthCapitalizedPrice,
    ...floors,
  ];
  const localV2 = [
    sources.under4mRealizedPrice,
    sources.under6mRealizedPrice,
    sources.under4mCapitalizedPrice,
    sources.under6mCapitalizedPrice,
    sources.sthRealizedPrice,
    sources.sthCapitalizedPrice,
    sources.sthMedianPriceBtcWeighted,
    sources.sthMedianPriceUsdWeighted,
  ];
  const cycleV2 = [
    sources.realizedPrice,
    sources.capitalizedPrice,
    sources.medianPriceBtcWeighted,
    sources.medianPriceUsdWeighted,
    sources.cointimeMedianPriceBtcWeighted,
    sources.cointimeMedianPriceUsdWeighted,
    sources.coinflowMedianPriceBtcWeighted,
    sources.coinflowMedianPriceUsdWeighted,
    sources.over6mRealizedPrice,
    sources.over4mRealizedPrice,
    sources.vaultedPrice,
    sources.activePrice,
    sources.trueMarketMeanPrice,
    sources.cointimePrice,
    sources.awakePrice,
    sources.coinflowPrice,
    ...floors,
  ];

  return [
    {
      name: "V1",
      variants: [
        { name: "Full", meter: rarityMeter.full, sources: [...local, ...cycle] },
        { name: "Local", meter: rarityMeter.local, sources: local },
        { name: "Cycle", meter: rarityMeter.cycle, sources: cycle },
      ],
    },
    {
      name: "V2",
      variants: [
        { name: "Full", meter: rarityMeter.fullV2, sources: [...localV2, ...cycleV2] },
        { name: "Local", meter: rarityMeter.localV2, sources: localV2 },
        { name: "Cycle", meter: rarityMeter.cycleV2, sources: cycleV2 },
      ],
    },
  ];
}
