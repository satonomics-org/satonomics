use std::{path::Path, result::Result as StdResult};

use bitview_cohort::AgeRangeId;
use bitview_plugin_distribution::AgeRangeUrpds;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, CentsCompact, Date, Sats};

use crate::PriceBounds;

#[derive(Clone, Copy, Default, Traversable)]
pub struct AgePriceBounds<T> {
    /// UTXOs younger than 120 days.
    pub under_4m: T,
    /// UTXOs younger than 150 days (STH).
    pub under_5m: T,
    /// UTXOs younger than 180 days.
    pub under_6m: T,
}

impl<T> AgePriceBounds<T> {
    pub fn try_from_fn<E>(mut create: impl FnMut(&str) -> StdResult<T, E>) -> StdResult<Self, E> {
        Ok(Self {
            under_4m: create("under_4m")?,
            under_5m: create("under_5m")?,
            under_6m: create("under_6m")?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.under_4m, &self.under_5m, &self.under_6m].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.under_4m, &mut self.under_5m, &mut self.under_6m].into_iter()
    }
}

impl AgePriceBounds<PriceBounds<Cents>> {
    pub fn include(&mut self, age: AgeRangeId, price: CentsCompact, sats: Sats) {
        if sats == Sats::ZERO {
            return;
        }
        for (bounds, excluded) in self.iter_mut().zip([
            AgeRangeId::From4MTo5M,
            AgeRangeId::From5MTo6M,
            AgeRangeId::From6MTo9M,
        ]) {
            if age.index() < excluded.index() {
                bounds.include(Cents::from(price));
            }
        }
    }

    pub fn read_if_exists(states_path: &Path, date: Date) -> Result<Self> {
        let mut bounds = Self::default();
        if !AgeRangeUrpds::path(states_path, date).try_exists()? {
            return Ok(bounds);
        }
        let sources = AgeRangeUrpds::read(states_path, date)?;
        for &age in &AgeRangeId::ALL[..AgeRangeId::From6MTo9M.index()] {
            // Snapshot entries are sorted. Only their occupied endpoints can
            // change an age cutoff's bounds; no aggregate URPD is needed.
            let entries = sources.get(age);
            for &(price, sats) in entries
                .iter()
                .find(|(_, sats)| *sats != Sats::ZERO)
                .into_iter()
                .chain(entries.iter().rfind(|(_, sats)| *sats != Sats::ZERO))
            {
                bounds.include(age, price, sats);
            }
        }
        Ok(bounds)
    }
}

#[cfg(test)]
#[path = "../tests/unit/age_price_bounds.rs"]
mod tests;
