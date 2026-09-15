import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';

import { buildCloudSnapshot, generateCloudSnapshot, dateAt, validPrice } from '../../scripts/cloud-snapshot.mjs';

const sources = [
  'rarity_meter_under_4m_realized_price_cents',
  'sth_realized_price_cents',
  'rarity_meter_under_6m_realized_price_cents',
  'rarity_meter_under_4m_capitalized_price_cents',
  'sth_capitalized_price_cents',
  'rarity_meter_under_6m_capitalized_price_cents',
];
const boundsSources = [4, 5, 6].map(months => ({
  name: `<${months}M`,
  series: ['min', 'max'].map(side => `bedrock_under_${months}m_cost_basis_${side}_cents`),
}));
export const seriesNames = ['price_ohlc_cents', ...sources, ...boundsSources.flatMap(source => source.series)];

// A point is [price in cents, displayed side: 0=min / 1=max]. An expanding
// extreme selects the opposite bound; an aging-out contraction does not.
export function trendBounds(minima, maxima) {
  let side = null;
  let previous = null;
  return minima.map((min, index) => {
    const pair = [min, maxima[index]];
    if (!pair.every(validPrice) || pair[0] > pair[1]) {
      side = null;
      previous = null;
      return null;
    }
    if (previous) {
      const lowerMin = pair[0] < previous[0];
      const higherMax = pair[1] > previous[1];
      // Both expanding on the same day leaves their ordering unknown.
      if (lowerMin !== higherMax) side = lowerMin ? 1 : 0;
    }
    previous = pair;
    return side === null ? null : [pair[side], side];
  });
}

export function buildSnapshot(histories, start, end, generatedAt) {
  const snapshot = buildCloudSnapshot(histories, start, end, generatedAt, seriesNames, sources.length);
  const bounds = boundsSources.map(({ name }, index) => {
    const [minima, maxima] = histories.slice(1 + sources.length + index * 2, 3 + sources.length + index * 2)
      .map(history => history.data);
    for (let day = 0; day < end; day++) {
      const pair = [minima[day], maxima[day]];
      if (pair.every(value => value === null)) continue;
      if (!pair.every(validPrice) || pair[0] > pair[1]) {
        throw new Error(`Invalid ${name} bounds on ${dateAt(day)}.`);
      }
    }
    if (!validPrice(minima[end - 1]) || !validPrice(maxima[end - 1])) {
      throw new Error(`Missing latest ${name} bounds; run the updated local indexer first.`);
    }
    return { name, points: trendBounds(minima, maxima).slice(start) };
  });
  return { ...snapshot, bounds };
}

export function generateSnapshot(options = {}) {
  return generateCloudSnapshot({
    ...options,
    output: options.output ?? new URL('./index.html', import.meta.url),
    seriesNames,
    buildSnapshot,
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { values } = parseArgs({ options: {
    api: { type: 'string', default: 'http://localhost:3110/api' },
    start: { type: 'string', default: '2011-01-01' },
  } });
  const snapshot = await generateSnapshot({ api: values.api, startDate: values.start });
  console.log(`Generated ${snapshot.rows.length.toLocaleString()} daily rows including today's partial data: ${snapshot.start} through ${snapshot.through}`);
  const latest = snapshot.rows.at(-1);
  console.log(`Latest cloud: $${(latest[4] / 100).toFixed(2)} – $${(latest[5] / 100).toFixed(2)}`);
  console.log(`Trend bounds: ${snapshot.bounds.map(({ name, points }) => `${name} ${points.at(-1)?.[1] === 0 ? 'min' : points.at(-1)?.[1] === 1 ? 'max' : 'untouched'}`).join(', ')}`);
}
