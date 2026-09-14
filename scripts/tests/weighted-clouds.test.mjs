import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import test from 'node:test';
import * as holders from '../../website_next_next/holders/generate.mjs';
import * as lth from '../../website_next_next/lth/generate.mjs';

function histories(seriesNames) {
  const sources = [[70, 130, 80, 90], [80, 70, 140, 90], [80, 90, 70, 150], [160, 90, 80, 70]];
  return seriesNames.map((_, index) => ({
    type: index === 0 ? 'OHLCCents' : 'Cents', index: 'day1', start: 0, end: 4,
    data: index === 0 ? Array.from({ length: 4 }, () => [100, 180, 90, 120]) : sources[index - 1],
  }));
}

for (const [name, generator, prefix] of [['Holders', holders, ''], ['LTH', lth, 'lth_']]) {
  const { buildSnapshot, generateSnapshot, seriesNames } = generator;
  test(`${name}: all four weighted sources contribute, with price separate and no trends`, () => {
    assert.deepEqual(seriesNames, ['price_ohlc_cents', `${prefix}awake_price_cents`, `${prefix}awake_capitalized_price_cents`,
      `${prefix}coinflow_price_cents`, `${prefix}coinflow_capitalized_price_cents`]);
    const snapshot = buildSnapshot(histories(seriesNames), 0, 4, '2009-01-04T12:00:00.000Z');
    assert.deepEqual(snapshot.rows.map(row => row.slice(4)), [[70, 160], [70, 130], [70, 140], [70, 150]]);
    assert.equal(snapshot.through, '2009-01-04');
    assert.equal('bounds' in snapshot, false);
    assert.deepEqual(snapshot.columns, ['open', 'high', 'low', 'close', 'min', 'max']);
  });

  test(`${name}: refresh includes the partial day and refuses missing data without replacing the page`, async t => {
    const directory = await mkdtemp(join(tmpdir(), 'holders-cloud-'));
    t.after(() => rm(directory, { recursive: true, force: true }));
    const output = pathToFileURL(join(directory, 'index.html'));
    await writeFile(output, '<script id="chart-data" type="application/json">{}</script>');
    const input = histories(seriesNames);
    t.mock.method(globalThis, 'fetch', async url => {
      const query = new URL(url).searchParams;
      assert.equal(query.get('series'), seriesNames.join(','));
      assert.equal(query.get('end'), '4');
      return Response.json(input);
    });
    const options = { startDate: '2009-01-02', now: new Date('2009-01-04T12:00:00Z'), output };
    const snapshot = await generateSnapshot(options);
    assert.equal(snapshot.rows.length, 3);
    const saved = await readFile(output, 'utf8');
    assert.deepEqual(JSON.parse(saved.slice(saved.indexOf('>') + 1, saved.indexOf('</script>'))), snapshot);
    input[2].data[3] = null;
    await assert.rejects(generateSnapshot(options), /Invalid price on 2009-01-04/);
    assert.equal(await readFile(output, 'utf8'), saved);
  });

}
