"""Refresh the standalone comparison from a running Bitview backend."""

import argparse
from concurrent.futures import ThreadPoolExecutor
from datetime import date, datetime, timedelta, timezone
import json
import math
from pathlib import Path
import re
from urllib.parse import urlencode
from urllib.request import urlopen

ROOT = Path(__file__).resolve().parents[2]
OUTPUT = Path(__file__).with_name("compare-all.html")
PERCENTILES = [0.1, 0.5, 1, 2, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 98, 99, 99.5, 99.9]
TAILS = [0.1, 0.5, 1, 2, 5, 95, 98, 99, 99.5, 99.9]
DAY_ZERO = date(2009, 1, 1)


def suffix(percentile, meter=False):
    value = str(percentile).replace(".", "_")
    return "pct" + (value.zfill(2) if meter and percentile in (1, 2, 5) else value)


def label(name):
    return (name.replace("median_price_btc_weighted", "median (BTC)")
            .replace("median_price_usd_weighted", "median (USD)")
            .replace("_", " ").title().replace("Sth", "STH").replace("Lth", "LTH")
            .replace("Btc", "BTC").replace("Usd", "USD"))


def combine(boundaries):
    tails = [max(column) if i < 5 else min(column)
             for i, column in enumerate(zip(*boundaries))]
    low, high = tails[4:6]
    values = []
    for p in PERCENTILES:
        if p in TAILS:
            values.append(tails[TAILS.index(p)])
        else:
            position = (p / 100 - 0.05) / 0.90
            value = (math.exp(math.log(low) + position * (math.log(high) - math.log(low)))
                     if low > 0 and high > 0 else low + position * (high - low))
            values.append(math.floor(value + 0.5))
    return values


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--api", default="http://localhost:3110/api")
    args = parser.parse_args()
    plugin = ROOT / "crates/bitview_plugin_rarity_meter/src"
    components = re.findall(r"pub (\w+): (?:MedianComponent|Component)<M>",
                            (plugin / "components.rs").read_text())
    source = (plugin / "lib.rs").read_text()
    normal = []
    for group in ("local", "cycle"):
        block = re.search(rf"let {group}_components = \[(.*?)\];", source, re.S).group(1)
        normal.extend(re.findall(r"self\.components\.(\w+)", block))
    if not components or not normal or not set(normal) <= set(components):
        raise ValueError("Could not read component definitions")
    v2_groups = {}
    for group in ("local", "cycle"):
        block = re.search(rf"let {group}_v2_components = \[(.*?)\];", source, re.S).group(1)
        v2_groups[group] = re.findall(r"self\.components\.(\w+)", block)
    members = [name for group in v2_groups.values() for name in group]
    if not members or len(members) != len(set(members)) or not set(members) <= set(components):
        raise ValueError("V2 groups must contain distinct known components")
    selected = [name for name in components if name in members]
    components = [name for name in components if name in selected or name in normal]

    component_names = {name: [f"{name}_{suffix(p)}_cents" for p in TAILS] for name in components}
    floor_names = {mode: [f"bedrock_{mode}_floor_pct{p}_cents"
                         for p in ("99_9", "99_5", "99", "98", "95")]
                   for mode in ("raw", "cointime", "coinflow")}
    full_names = [f"rarity_meter_{suffix(p, True)}_cents" for p in PERCENTILES]
    v2_prefixes = dict(full="rarity_meter_v2", local="local_rarity_meter_v2", cycle="cycle_rarity_meter_v2")
    v2_names = {group: [f"{prefix}_{suffix(p, True)}_cents" for p in PERCENTILES]
                for group, prefix in v2_prefixes.items()}
    names = ["price_close_cents", *full_names,
             *(name for group in v2_names.values() for name in group),
             *(f"{prefix}_{metric}" for prefix in v2_prefixes.values() for metric in ("index", "score")),
             *(name for group in component_names.values() for name in group),
             *(name for group in floor_names.values() for name in group)]
    today = datetime.now(timezone.utc).date()
    end = (today - DAY_ZERO).days  # Exclude today's incomplete daily candle.
    batches = [names[i:i + 10] for i in range(0, len(names), 10)]

    def fetch(batch):
        query = urlencode(dict(series=",".join(batch), index="day1", start="2013-01-01", end=end))
        with urlopen(f"{args.api.rstrip('/')}/series/bulk?{query}", timeout=60) as response:
            data = json.load(response)
        if not isinstance(data, list) or len(data) != len(batch):
            raise ValueError(f"Missing component data: {data}")
        for name, series in zip(batch, data):
            if "data" not in series or (name.endswith("_cents") and series.get("type") != "Cents"):
                raise ValueError(f"Unexpected history for {name}: {series}")
        return dict(zip(batch, data))

    series = {}
    with ThreadPoolExecutor(max_workers=4) as pool:
        for result in pool.map(fetch, batches):
            series.update(result)
    start = max(item["start"] for item in series.values())
    end = min(end, *(item["start"] + len(item["data"]) for item in series.values()))

    def at(name, day):
        item = series[name]
        return item["data"][day - item["start"]]

    def verify_v2(group, boundaries, day, spot):
        values = combine(boundaries)
        for p, actual, name in zip(PERCENTILES, values, v2_names[group]):
            expected = at(name, day)
            if expected is None or abs(actual - expected) > (0 if p in TAILS else 1):
                raise ValueError(f"{group.title()} V2 mismatch on day {day}, P{p}: {actual} vs {expected}")
        index = sum(spot > value for value in values[14:]) - sum(spot < value for value in values[:5])
        score = sum(sum(spot > value for value in row[5:]) - sum(spot < value for value in row[:5])
                    for row in boundaries)
        prefix = v2_prefixes[group]
        if index != at(f"{prefix}_index", day) or score != at(f"{prefix}_score", day):
            raise ValueError(f"{group.title()} V2 index/score mismatch on day {day}")
        return values

    rows, excluded = [], 0
    worst_error = 0
    driver_labels = [*(label(name) for name in selected), *(f"{mode.title()} Bedrock" for mode in floor_names)]
    for day in range(start, end):
        spot = at("price_close_cents", day)
        prices = [[at(name, day) for name in component_names[component]] for component in components]
        expected = [at(name, day) for name in full_names]
        if any(value is None or not math.isfinite(value) or value <= 0
               for value in [spot, *expected, *(value for row in prices for value in row)]):
            excluded += 1
            continue
        floors = [[at(name, day) for name in group] for group in floor_names.values()]
        # Optional direct floors use the backend's finite-only rule. Infinity
        # on their upper side ensures they never affect upper boundaries.
        floor_rows = [[value if value is not None and math.isfinite(value) else 0 for value in row]
                      + [math.inf] * 5 for row in floors]
        all_rows = [prices[components.index(name)] for name in selected] + floor_rows
        baseline = combine([prices[components.index(name)] for name in normal] + floor_rows)
        for i, (actual, value) in enumerate(zip(baseline, expected)):
            error = abs(actual - value)
            worst_error = max(worst_error, error)
            tolerance = 0 if PERCENTILES[i] in TAILS else 1
            if error > tolerance:
                raise ValueError(f"Normal Full mismatch on {DAY_ZERO + timedelta(days=day)}, P{PERCENTILES[i]}: {actual} vs {value}")
        experimental = verify_v2("full", all_rows, day, spot)
        for group, members in v2_groups.items():
            boundaries = [prices[components.index(name)] for name in members]
            if group == "cycle":
                boundaries += floor_rows
            verify_v2(group, boundaries, day, spot)
        drivers = [max(range(len(all_rows)), key=lambda i: all_rows[i][4]),
                   min(range(len(all_rows)), key=lambda i: all_rows[i][5])]
        rows.append([day, spot, expected, experimental, drivers])
    if not rows:
        raise ValueError("No complete shared history; build/recompute the new components first")

    payload = dict(rows=rows, percentiles=PERCENTILES, drivers=driver_labels,
                   components=[label(name) for name in selected], normalComponents=[label(name) for name in normal],
                   api=args.api, fetchedAt=datetime.now(timezone.utc).isoformat(),
                   stamps=sorted({item["stamp"] for item in series.values()}),
                   excludedDays=excluded, maxBaselineErrorCents=worst_error)
    html = OUTPUT.read_text()
    data = json.dumps(payload, separators=(",", ":"), allow_nan=False)
    html, count = re.subn(r'(<script id="snapshot" type="application/json">).*?(</script>)',
                         lambda match: match[1] + data + match[2], html, count=1, flags=re.S)
    if count != 1:
        raise ValueError("Missing snapshot placeholder")
    if "/* CHART_LIBRARY */" in html:
        library = ROOT / "website/scripts/modules/lightweight-charts/5.2.1/dist/lightweight-charts.standalone.production.js"
        html = html.replace("/* CHART_LIBRARY */", library.read_text())
    OUTPUT.write_text(html)
    print(f"Built {OUTPUT}: {len(rows)} daily rows, {len(selected)} prices (excluding LTH) + 3 Bedrock floors.")
    print(f"Normal Full verified: exact tail boundaries; max middle error {worst_error} cent(s).")
    print("Full, Local, and Cycle V2 verified against backend: all bands, index, and score.")
    print(f"Latest P5: normal ${rows[-1][2][4]/100:,.2f}, all ${rows[-1][3][4]/100:,.2f}")
    print(f"Latest P95: normal ${rows[-1][2][14]/100:,.2f}, all ${rows[-1][3][14]/100:,.2f}")


if __name__ == "__main__":
    main()
