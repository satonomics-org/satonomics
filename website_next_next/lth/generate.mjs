import { pathToFileURL } from 'node:url';
import { parseArgs } from 'node:util';
import { buildCloudSnapshot, generateCloudSnapshot } from '../../scripts/cloud-snapshot.mjs';

// First day with positive values for all four LTH sources on the log chart.
const DEFAULT_START = '2011-02-20';

export const seriesNames = [
  'price_ohlc_cents',
  'lth_awake_price_cents',
  'lth_awake_capitalized_price_cents',
  'lth_coinflow_price_cents',
  'lth_coinflow_capitalized_price_cents',
];

export function buildSnapshot(histories, start, end, generatedAt) {
  return buildCloudSnapshot(histories, start, end, generatedAt, seriesNames);
}

export function generateSnapshot(options = {}) {
  return generateCloudSnapshot({
    startDate: DEFAULT_START,
    ...options,
    output: options.output ?? new URL('./index.html', import.meta.url),
    seriesNames,
    buildSnapshot,
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { values } = parseArgs({ options: {
    api: { type: 'string', default: 'http://localhost:3110/api' },
    start: { type: 'string', default: DEFAULT_START },
  } });
  const snapshot = await generateSnapshot({ api: values.api, startDate: values.start });
  const latest = snapshot.rows.at(-1);
  console.log(`Generated ${snapshot.rows.length.toLocaleString()} daily rows: ${snapshot.start} through ${snapshot.through}`);
  console.log(`Latest LTH Cloud: $${(latest[4] / 100).toFixed(2)} – $${(latest[5] / 100).toFixed(2)}`);
}
