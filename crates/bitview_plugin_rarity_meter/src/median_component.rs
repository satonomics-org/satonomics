use bitview_transforms::price_ratio;
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyPerBlock, Price, RangeMapVec, RatioPerBlock};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Day1, Height, Lengths, PriceRatio, Version};
use vecdb::{AnyVec, Database, LazyVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use crate::{Component, component};

#[derive(Traversable)]
pub struct MedianComponent<M: StorageMode = Rw> {
    /// Median creation price from the existing weighted cost-basis distribution.
    #[traversable(flatten)]
    pub price: Price<LazyPerBlock<Cents>>,
    /// Spot price divided by the median creation price.
    #[traversable(flatten)]
    pub relative: RatioPerBlock<PriceRatio, M>,
    #[traversable(flatten)]
    pub component: Component<M>,
    #[traversable(skip)]
    day_first_height: Option<RangeMapVec<Day1, Height>>,
}

impl MedianComponent {
    pub fn forced_import_daily(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        source: &impl ReadableCloneableVec<Height, Option<Cents>>,
    ) -> Result<Self> {
        // Missing daily snapshots must yield an undefined spot/reference ratio.
        let price = LazyVec::init(
            &format!("{name}_source"),
            version,
            source.read_only_boxed_clone(),
            |_, value| value.unwrap_or(Cents::ZERO),
        );
        let mut component = Self::forced_import(db, name, version, indexes, &price)?;
        component.day_first_height = Some(indexes.first_height.day1.clone());
        Ok(component)
    }

    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
        source: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        Ok(Self {
            price: Price::from_height_source(name, version, source, indexes),
            relative: RatioPerBlock::forced_import(db, name, version, indexes)?,
            component: component::forced_import(db, name, version, indexes, source)?,
            day_first_height: None,
        })
    }

    pub fn compute(
        &mut self,
        starting_lengths: &Lengths,
        spot: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        let mut starting_lengths = *starting_lengths;
        starting_lengths.height = self.starting_height(starting_lengths.height);
        self.relative.ppm.height.compute_transform2(
            starting_lengths.height,
            spot,
            &self.price.cents.height,
            |(height, spot, price, _)| (height, price_ratio(spot, price)),
            exit,
        )?;
        component::compute(
            &mut self.component,
            &starting_lengths,
            &self.relative.ratio.height,
            exit,
        )
    }

    pub(crate) fn starting_height(&self, height: Height) -> Height {
        if let Some(first_heights) = &self.day_first_height {
            // A revised daily median changes the reference for the entire day,
            // including blocks before the indexer's safe resume height.
            let height = height.min(Height::from(self.relative.ppm.height.len()));
            let day = first_heights.mapping().read().get_shared(height);
            day.and_then(|day| first_heights.collect_one(day))
                .unwrap_or_default()
        } else {
            height
        }
    }
}

#[cfg(test)]
#[path = "median_component_tests.rs"]
mod tests;
