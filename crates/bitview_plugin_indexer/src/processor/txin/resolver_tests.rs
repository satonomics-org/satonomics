use bitcoin::{
    Network, OutPoint as BitcoinOutPoint, Txid as BitcoinTxid, blockdata::constants::genesis_block,
    hashes::Hash,
};
use brk_types::{Block, Height, Version};
use tempfile::tempdir;
use vecdb::WritableVec;

use super::*;
use crate::{Lengths, Readers, Stores, Vecs, processor::BlockBuffers, test_cache::init_cache};

fn txid(value: u64) -> Txid {
    let mut bytes = [0; 32];
    bytes[..8].copy_from_slice(&value.to_le_bytes());
    BitcoinTxid::from_byte_array(bytes).into()
}

#[test]
fn parent_cache_collisions_invalidation_and_chain_continuity() {
    let mut buffers = BlockBuffers::default();
    let a = TxidPrefix::from(txid(1));
    let b = TxidPrefix::from(txid(1 + (1 << 16)));
    let read = ParentRead {
        tx_index: TxIndex::new(7),
        first_txout_index: TxOutIndex::new(11),
    };
    let cache = &mut buffers.inputs.reads.cache;
    assert_eq!(cache.get(a), None);
    cache.insert(a, read);
    cache.insert(b, read);
    assert_eq!(cache.get(a), None);
    assert_eq!(cache.get(b), Some(read));
    cache.invalidate(a);
    assert_eq!(cache.get(b), Some(read));
    let block = Block::from((Height::ZERO, genesis_block(Network::Bitcoin)));
    buffers.finish_block(*block.hash());
    buffers.continue_from(Some(*block.hash()));
    assert_eq!(buffers.inputs.reads.cache.get(b), Some(read));
    buffers.continue_from(None);
    assert_eq!(buffers.inputs.reads.cache.get(b), None);
}

#[test]
fn parent_resolution_preserves_store_updates_bounds_errors_and_pending_reads() -> Result<()> {
    init_cache();
    let dir = tempdir()?;
    let mut vecs = Vecs::forced_import(dir.path(), Version::new(34))?;
    let mut stores = Stores::forced_import(dir.path(), Version::new(34))?;
    let prefixes: Vec<_> = (0..1002).map(|i| TxidPrefix::from(txid(i + 1))).collect();
    for (i, &prefix) in prefixes.iter().enumerate() {
        vecs.transactions
            .first_txout_index
            .push(TxOutIndex::from(i * 2));
        stores
            .transaction_stores_mut()
            .txid_prefixes
            .insert(prefix, TxIndex::from(i));
    }
    let readers = Readers::new(&vecs);
    let block = Block::from((Height::ZERO, genesis_block(Network::Bitcoin)));
    let mut lengths = Lengths::default();
    let processor = BlockProcessor {
        block: &block,
        height: Height::ZERO,
        check_collisions: true,
        lengths: &mut lengths,
        vecs: &mut vecs,
        stores: &mut stores,
        readers: &readers,
    };
    let mut resolver = InputResolver::default();
    // Both branches of the production parallel/raw read threshold, cold and warm.
    for len in [0, 1, 999, 1000, 1002, 1] {
        resolver
            .reads
            .resolve_parents(&processor, &prefixes[..len], TxIndex::new(2000))?;
        for (i, read) in resolver.reads.parents.iter().enumerate() {
            assert_eq!(read.tx_index, TxIndex::from(i));
            assert_eq!(read.first_txout_index, TxOutIndex::from(i * 2));
        }
    }
    // Exercise the complete input resolver with repeated historical parents,
    // a same-block parent, and a coinbase, using both cold and warm caches.
    processor.vecs.outputs.output_type.push(OutputType::P2PKH);
    processor.vecs.outputs.output_type.push(OutputType::P2PKH);
    processor.vecs.outputs.type_index.push(TypeIndex::new(16));
    processor.vecs.outputs.type_index.push(TypeIndex::new(17));
    processor.lengths.tx_index = TxIndex::new(2000);
    processor.lengths.txout_index = TxOutIndex::new(3000);
    let mut spending = block.txdata[0].clone();
    let mut previous = spending.input[0].clone();
    previous.previous_output = BitcoinOutPoint::new(txid(1).into(), 1);
    let mut same_block = previous.clone();
    same_block.previous_output = BitcoinOutPoint::new(txid(88888).into(), 0);
    spending.input = vec![previous.clone(), previous, same_block];
    let mut txs = [
        ComputedTx::new(
            TxIndex::new(2000),
            &block.txdata[0],
            txid(88888),
            true,
            0,
            0,
        ),
        ComputedTx::new(TxIndex::new(2001), &spending, txid(88889), true, 0, 0),
    ];
    ComputedTx::set_block_offsets(&mut txs);
    resolver.clear_cache();
    let cold = format!("{:?}", resolver.resolve(&processor, &txs)?);
    let warm = resolver.resolve(&processor, &txs)?;
    assert_eq!(cold, format!("{warm:?}"));
    assert!(matches!(warm[0], InputSource::Coinbase));
    for source in &warm[1..3] {
        let InputSource::PreviousBlock {
            outpoint,
            txout_index,
            output_type,
            type_index,
            ..
        } = source
        else {
            panic!("expected historical parent")
        };
        assert_eq!(outpoint.tx_index(), TxIndex::ZERO);
        assert_eq!(*txout_index, TxOutIndex::new(1));
        assert_eq!(*output_type, OutputType::P2PKH);
        assert_eq!(*type_index, TypeIndex::new(17));
    }
    assert!(
        matches!(warm[3], InputSource::SameBlock { txout_index, .. } if txout_index == TxOutIndex::new(3000))
    );
    // Preparing a new block invalidates every prefix it may replace (including BIP30).
    let replacement_id = txid(1);
    let computed = ComputedTx::new(
        TxIndex::new(1002),
        &block.txdata[0],
        replacement_id,
        true,
        0,
        0,
    );
    resolver.prepare(&[computed], TxIndex::new(1002), TxOutIndex::new(2004));
    assert_eq!(resolver.reads.cache.get(prefixes[0]), None);
    processor
        .vecs
        .transactions
        .first_txout_index
        .push(TxOutIndex::new(9999));
    processor
        .stores
        .transaction_stores_mut()
        .txid_prefixes
        .insert(prefixes[0], TxIndex::new(1002));
    resolver
        .reads
        .resolve_parents(&processor, &prefixes[..1], TxIndex::new(2000))?;
    assert_eq!(
        resolver.reads.parent(0).first_txout_index,
        TxOutIndex::new(9999)
    );
    assert!(matches!(
        resolver
            .reads
            .resolve_parents(&processor, &prefixes[..1], TxIndex::new(1002)),
        Err(Error::UnknownTxid)
    ));
    // Rollback discards cached values before reading the changed store.
    resolver.clear_cache();
    processor
        .stores
        .transaction_stores_mut()
        .txid_prefixes
        .remove(prefixes[0]);
    assert!(matches!(
        resolver
            .reads
            .resolve_parents(&processor, &prefixes[..1], TxIndex::new(2000)),
        Err(Error::UnknownTxid)
    ));
    assert_eq!(resolver.reads.cache.get(prefixes[0]), None);
    Ok(())
}
