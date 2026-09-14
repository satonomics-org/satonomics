import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import { buildCloudSnapshot, generateCloudSnapshot } from '../../scripts/cloud-snapshot.mjs';

export const seriesNames = [
  'price_ohlc_cents',
  'awake_price_cents',
  'awake_capitalized_price_cents',
  'coinflow_price_cents',
  'coinflow_capitalized_price_cents',
];

export function buildSnapshot(histories, start, end, generatedAt) {
  return buildCloudSnapshot(histories, start, end, generatedAt, seriesNames);
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
  const latest = snapshot.rows.at(-1);
  console.log(`Generated ${snapshot.rows.length.toLocaleString()} daily rows: ${snapshot.start} through ${snapshot.through}`);
  console.log(`Latest Holders Cloud: $${(latest[4] / 100).toFixed(2)} – $${(latest[5] / 100).toFixed(2)}`);
}
