import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import test from 'node:test';
import { buildSnapshot, generateSnapshot, seriesNames, trendBounds } from '../../website_next_next/sth/generate.mjs';

const candle = (low, high) => [(low + high) / 2, high, low, (low + high) / 2];

test('expanding extrema switch sides immediately while contractions preserve the trend', () => {
  const minima = [80, 80, 82, 82, 78, 78, 84];
  const maxima = [120, 124, 124, 122, 122, 124, 124];
  const result = trendBounds(minima, maxima);
  assert.deepEqual(result, [null, [80, 0], [82, 0], [82, 0], [122, 1], [78, 0], [84, 0]]);
  for (let end = 1; end <= minima.length; end++) {
    assert.deepEqual(trendBounds(minima.slice(0, end), maxima.slice(0, end)), result.slice(0, end));
  }
});

test('ambiguous expansions preserve the selection; missing observations reset knowledge', () => {
  assert.deepEqual(trendBounds(
    [80, 80, 70, null, 80, 80], [120, 130, 140, null, 120, 130],
  ), [null, [80, 0], [70, 0], null, null, [80, 0]]);
  assert.deepEqual(trendBounds([80, 70], [120, 130]), [null, null]);
  assert.deepEqual(trendBounds([0, 0], [10, 20]), [null, [0, 0]]);
  assert.deepEqual(trendBounds([120], [80]), [null]);
});

function histories() {
  return seriesNames.map((_, index) => ({
    type: index === 0 ? 'OHLCCents' : 'Cents', index: 'day1', start: 0, end: 6,
    data: index === 0 ? [candle(80, 100), ...Array.from({ length: 5 }, () => candle(90, 110))] :
      index > 6 && index % 2 === 0 ? [100, 120, 120, 120, 120, 120] : Array(6).fill(index <= 6 ? 90 + index : 80),
  }));
}

test('display cropping preserves the trend and candle overlap cannot override it', () => {
  const input = histories();
  const snapshot = buildSnapshot(input, 3, 6, '2009-01-06T12:00:00.000Z');
  assert.equal(snapshot.start, '2009-01-04');
  assert.equal(snapshot.through, '2009-01-06');
  assert.deepEqual(snapshot.rows[0], [100, 110, 90, 100, 91, 96]);
  assert.deepEqual(snapshot.bounds.map(bound => bound.points[0]), [[80, 0], [80, 0], [80, 0]]);
  // Candle overlap alone no longer selects a side.
  input[0].data[5] = candle(110, 120);
  assert.deepEqual(buildSnapshot(input, 3, 6, snapshot.generatedAt).bounds, snapshot.bounds);
  // A new low in today's partial data selects the ceiling.
  for (const index of [7, 9, 11]) input[index].data[5] = 70;
  const updated = buildSnapshot(input, 3, 6, '2009-01-06T13:00:00.000Z');
  assert.deepEqual(updated.bounds.map(bound => bound.points.at(-1)), [[120, 1], [120, 1], [120, 1]]);
  assert.deepEqual(updated.bounds[0].points.slice(0, -1), snapshot.bounds[0].points.slice(0, -1));
});

test('each age pair selects its own side independently', () => {
  const input = histories();
  input[9].data.fill(60);
  input[9].data[0] = 80;
  input[10].data.fill(100);
  input[11].data.fill(40);
  input[12].data.fill(140);
  const snapshot = buildSnapshot(input, 3, 6, '2009-01-06T12:00:00.000Z');
  assert.deepEqual(snapshot.bounds.map(bound => bound.points.at(-1)), [[80, 0], [100, 1], null]);
});

test('generator refreshes an embedded snapshot and preserves it if the backend is not ready', async t => {
  const directory = await mkdtemp(join(tmpdir(), 'sth-bounds-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const output = pathToFileURL(join(directory, 'index.html'));
  await writeFile(output, '<script id="chart-data" type="application/json">{"old":true}</script>');
  let input = histories();
  t.mock.method(globalThis, 'fetch', async url => {
    const query = new URL(url).searchParams;
    assert.equal(query.get('start'), '0');
    assert.equal(query.get('end'), '6');
    assert.equal(query.get('series'), seriesNames.join(','));
    return Response.json(input);
  });
  const options = { startDate: '2009-01-04', now: new Date('2009-01-06T12:00:00Z'), output };
  const snapshot = await generateSnapshot(options);
  const saved = await readFile(output, 'utf8');
  assert.deepEqual(JSON.parse(saved.slice(saved.indexOf('>') + 1, saved.indexOf('</script>'))), snapshot);
  input[7].data[5] = null;
  input[8].data[5] = null;
  await assert.rejects(generateSnapshot(options), /Missing latest <4M bounds/);
  assert.equal(await readFile(output, 'utf8'), saved);
  input = histories();
  input[8].end = 5;
  await assert.rejects(generateSnapshot(options), /Incomplete or misaligned/);
  assert.equal(await readFile(output, 'utf8'), saved);
});
