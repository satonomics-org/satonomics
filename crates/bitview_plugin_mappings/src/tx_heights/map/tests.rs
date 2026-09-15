use super::*;

fn check(map: &TxHeightMap, queries: impl IntoIterator<Item = u32>) {
    for query in queries {
        let index = TxIndex::new(query);
        assert_eq!(
            map.get_shared(index),
            map.ranges.get_shared(index),
            "{query}"
        );
    }
}

#[test]
fn floor_lookup_handles_empty_gaps_duplicates_and_maximum_key() {
    for starts in [
        vec![],
        vec![0],
        vec![1, 1, 1],
        vec![65_535, 65_536, 65_536, 65_537],
        vec![0, 1, 65_536, 2_000_000, u32::MAX],
        vec![u32::MAX, u32::MAX],
    ] {
        let map = TxHeightMap::from(starts.into_iter().map(TxIndex::new).collect::<Vec<_>>());
        check(
            &map,
            [
                0,
                1,
                2,
                65_534,
                65_535,
                65_536,
                65_537,
                65_538,
                1_000_000,
                2_000_000,
                u32::MAX - 1,
                u32::MAX,
            ],
        );
    }
}

#[test]
fn append_and_reorg_rebuild_only_the_changed_directory_suffix() {
    let mut map = TxHeightMap::from(Vec::new());
    let mut seed = 413u64;
    for round in 0..30 {
        let from = if round % 4 == 0 {
            map.len() / 2
        } else {
            map.len()
        };
        let mut last = map
            .as_slice()
            .get(from.wrapping_sub(1))
            .copied()
            .map(u32::from)
            .unwrap_or(0);
        let starts = (0..1100)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                last = last.saturating_add((seed % 65_537) as u32);
                TxIndex::new(last)
            })
            .collect::<Vec<_>>();
        map.update_at(from, starts);
        check(&map, (0..20_000).map(|i| i * (last / 20_000)));
        check(&map, [last, last.saturating_add(1), u32::MAX]);
    }
    map.update_at(0, []);
    check(&map, [0, u32::MAX]);
    map.update_at(0, [TxIndex::new(123)]);
    check(&map, [0, 122, 123, 124, u32::MAX]);
}
