import { readFile, writeFile, rename } from 'node:fs/promises';

const DAY_MS = 86_400_000;
const DAY_ZERO = Date.UTC(2009, 0, 1);
export const validPrice = value => Number.isSafeInteger(value) && value >= 0;
export const dateAt = day => new Date(DAY_ZERO + day * DAY_MS).toISOString().slice(0, 10);
const validCandle = candle => Array.isArray(candle) && candle.length === 4 && candle.every(validPrice) &&
  candle[2] <= Math.min(candle[0], candle[3]) && candle[1] >= Math.max(candle[0], candle[3]);

export function buildCloudSnapshot(histories, start, end, generatedAt, names, sourceCount = names.length - 1) {
  if (!Array.isArray(histories) || histories.length !== names.length) {
    throw new Error('Expected Bitcoin price and all source histories.');
  }
  for (const [index, history] of histories.entries()) {
    if (history.type !== (index === 0 ? 'OHLCCents' : 'Cents') || history.index !== 'day1' || history.start !== 0 ||
        history.end !== end || history.data?.length !== end) {
      throw new Error(`Incomplete or misaligned daily history: ${names[index]}`);
    }
  }
  const rows = histories[0].data.slice(start).map((candle, offset) => {
    const prices = histories.slice(1, 1 + sourceCount).map(history => history.data[start + offset]);
    if (!validCandle(candle) || ![...candle, ...prices].every(value => validPrice(value) && value > 0)) {
      throw new Error(`Invalid price on ${dateAt(start + offset)}; refusing a partial cloud.`);
    }
    return [...candle, Math.min(...prices), Math.max(...prices)];
  });
  return {
    generatedAt, start: dateAt(start), through: dateAt(end - 1), unit: 'cents',
    columns: ['open', 'high', 'low', 'close', 'min', 'max'], rows,
  };
}

export async function generateCloudSnapshot({ api = 'http://localhost:3110/api', startDate = '2011-01-01',
  now = new Date(), output, seriesNames, buildSnapshot }) {
  const start = (Date.parse(startDate) - DAY_ZERO) / DAY_MS;
  // The API's end is exclusive; include today's in-progress observation.
  const end = (Date.parse(now.toISOString().slice(0, 10)) - DAY_ZERO) / DAY_MS + 1;
  if (!Number.isInteger(start) || start < 0 || start >= end) {
    throw new Error('--start must be a date between 2009-01-01 and today.');
  }
  // Start at genesis so stateful overlays can select before the display range.
  const query = new URLSearchParams({ series: seriesNames.join(','), index: 'day1', start: '0', end: String(end) });
  const response = await fetch(`${api.replace(/\/$/, '')}/series/bulk?${query}`, {
    signal: AbortSignal.timeout(60_000),
  });
  if (!response.ok) throw new Error(`Local API returned ${response.status}: ${await response.text()}`);
  const snapshot = buildSnapshot(await response.json(), start, end, now.toISOString());
  const html = await readFile(output, 'utf8');
  const dataTag = /(<script id="chart-data" type="application\/json">)[\s\S]*?(<\/script>)/g;
  if ([...html.matchAll(dataTag)].length !== 1) throw new Error('Expected one embedded chart snapshot.');
  const json = JSON.stringify(snapshot).replaceAll('<', '\\u003c');
  const temporary = new URL(output.href + '.tmp');
  await writeFile(temporary, html.replace(dataTag, (_, open, close) => open + json + close));
  await rename(temporary, output);
  return snapshot;
}
