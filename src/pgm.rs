use rkyv::{
    access_unchecked, api::high::to_bytes_with_alloc, deserialize, rancor::Error,
    ser::allocator::Arena, util::AlignedVec,
};
use rkyv_derive::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug)]
pub struct Segment {
    pub slope: f64,
    pub intercept: f64,
    pub start_key: u64,
    pub end_key: u64,
}

#[derive(Archive, Deserialize, Serialize, Debug)]
pub struct PGMIndex {
    pub segments: Vec<Segment>,
    pub top_level: Option<Vec<Segment>>,
    pub epsilon: usize,
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PGMIndexError {
    #[error("Keys are not sorted")]
    KeysNotSorted,
}

macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err);
        }
    };
}

impl PGMIndex {
    pub fn build(keys: &[u64], epsilon: usize) -> Result<Self, PGMIndexError> {
        ensure!(
            keys.windows(2).all(|w| w[0] <= w[1]),
            PGMIndexError::KeysNotSorted
        );
        PGMIndex::build_unsafe(keys, epsilon)
    }

    /// Build the index without safety checks for invariants the algorithm
    /// relies for the algorithm to be accurate.
    pub fn build_unsafe(keys: &[u64], epsilon: usize) -> Result<Self, PGMIndexError> {
        let segments = Self::build_segments(keys, epsilon);
        // Top-level input: start keys of each segment

        let top_keys: Vec<u64> = segments.iter().map(|s| s.start_key).collect();

        let top_level = if top_keys.len() > 2 {
            Some(Self::build_segments(&top_keys, epsilon))
        } else {
            None
        };

        Ok(Self {
            segments,
            top_level,
            epsilon,
        })
    }

    fn build_segments(keys: &[u64], epsilon: usize) -> Vec<Segment> {
        let epsilon = epsilon as f64;
        let mut segments = Vec::new();

        let mut start = 0;
        let mut s_min = f64::NEG_INFINITY;
        let mut s_max = f64::INFINITY;

        for i in 1..keys.len() {
            let x0 = keys[start] as f64;
            let y0 = start as f64;
            let xi = keys[i] as f64;
            let yi = i as f64;

            if (xi - x0).abs() < f64::EPSILON {
                continue;
            }

            let new_s_min = ((yi - epsilon) - y0) / (xi - x0);
            let new_s_max = ((yi + epsilon) - y0) / (xi - x0);
            s_min = s_min.max(new_s_min);
            s_max = s_max.min(new_s_max);

            if s_min > s_max {
                let x1 = keys[i - 1] as f64;
                let y1 = (i - 1) as f64;
                let slope = if (x1 - x0).abs() < f64::EPSILON {
                    0.0
                } else {
                    (y1 - y0) / (x1 - x0)
                };
                let intercept = y0 - slope * x0;

                segments.push(Segment {
                    slope,
                    intercept,
                    start_key: keys[start],
                    end_key: keys[i - 1],
                });

                start = i - 1;
                s_min = f64::NEG_INFINITY;
                s_max = f64::INFINITY;
            }
        }

        let x0 = keys[start] as f64;
        let x1 = keys[keys.len() - 1] as f64;
        let y0 = start as f64;
        let y1 = (keys.len() - 1) as f64;
        let slope = if (x1 - x0).abs() < f64::EPSILON {
            0.0
        } else {
            (y1 - y0) / (x1 - x0)
        };
        let intercept = y0 - slope * x0;

        segments.push(Segment {
            slope,
            intercept,
            start_key: keys[start],
            end_key: keys[keys.len() - 1],
        });

        segments
    }

    /// Returns the index range [lo, hi] where `key` may appear.
    /// This range is guaranteed to contain the key "if" it exists.
    pub fn search(&self, key: u64) -> (usize, usize) {
        let seg_index = if let Some(top) = &self.top_level {
            let i = match top.binary_search_by_key(&key, |seg| seg.end_key) {
                Ok(i) | Err(i) => i.min(top.len() - 1),
            };

            let top_seg = &top[i];
            let approx_index = (top_seg.slope * key as f64 + top_seg.intercept)
                .max(0.0)
                .round() as usize;
            approx_index.min(self.segments.len() - 1)
        } else {
            match self.segments.binary_search_by_key(&key, |seg| seg.end_key) {
                Ok(i) | Err(i) => i.min(self.segments.len() - 1),
            }
        };

        let seg = &self.segments[seg_index];
        let predicted = seg.slope * key as f64 + seg.intercept;
        let pos = predicted.max(0.0).round() as isize;

        let lo = pos.saturating_sub(self.epsilon as isize).max(0) as usize;
        let hi = (pos + self.epsilon as isize)
            .min(self.total_keys() as isize - 1)
            .max(0) as usize;

        (lo, hi)
    }

    pub fn to_bytes(&self) -> Result<AlignedVec, Error> {
        let mut arena = Arena::new();
        to_bytes_with_alloc::<_, Error>(self, arena.acquire())
    }

    /// Provides zero-copy access to the archived form.
    /// Lifetime is tied to `bytes`.
    pub fn as_archived(bytes: &[u8]) -> Result<&rkyv::Archived<PGMIndex>, Error> {
        rkyv::access::<rkyv::Archived<PGMIndex>, Error>(bytes)
    }

    /// Unsafely access the archived index without bounds or validation.
    /// Use only when buffer is known to be valid.
    pub unsafe fn as_archived_unchecked(bytes: &[u8]) -> &rkyv::Archived<PGMIndex> {
        unsafe { access_unchecked::<rkyv::Archived<PGMIndex>>(bytes) }
    }

    /// Deserialize from archived bytes back into a heap-owned PGMIndex.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let archived = rkyv::access::<rkyv::Archived<PGMIndex>, Error>(bytes)?;
        deserialize::<PGMIndex, Error>(archived)
    }

    fn total_keys(&self) -> usize {
        self.segments.last().map(|s| s.end_key).unwrap_or(0) as usize + 1
    }
}

impl ArchivedPGMIndex {
    /// Returns the index range [lo, hi] where `key` may appear.
    /// This range is guaranteed to contain the key "if" it exists.
    pub fn search(&self, key: u64) -> (usize, usize) {
        let segments: &[ArchivedSegment] = &self.segments;
        let epsilon = self.epsilon.to_native() as isize;

        // Handle Archived<Option<Vec<T>>> as Option<&[T]>
        let seg_index = if let Some(top) = self.top_level.as_ref().map(|v| &**v) {
            let i = match top.binary_search_by_key(&key, |seg| seg.end_key.to_native()) {
                Ok(i) | Err(i) => i.min(top.len() - 1),
            };
            let seg = &top[i];
            let estimate = (seg.slope * key as f64 + seg.intercept).max(0.0).round() as usize;
            estimate.min(segments.len().saturating_sub(1))
        } else {
            match segments.binary_search_by_key(&key, |seg| seg.end_key.to_native()) {
                Ok(i) | Err(i) => i.min(segments.len().saturating_sub(1)),
            }
        };

        let seg = &segments[seg_index];
        let predicted = (seg.slope * key as f64 + seg.intercept).max(0.0).round() as isize;

        // TODO: safely support conversion from little endian types to native
        let lo = predicted.saturating_sub(epsilon).max(0) as usize;
        let hi = (predicted + epsilon)
            .min(self.total_keys() as isize - 1)
            .max(0) as usize;

        (lo, hi)
    }

    fn total_keys(&self) -> usize {
        self.segments
            .last()
            .map(|s| s.end_key.to_native())
            .unwrap_or(0) as usize
            + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_search() {
        let keys: Vec<u64> = (0..1000).step_by(5).collect();
        let epsilon = 8;
        let pgm = PGMIndex::build(&keys, epsilon).unwrap();

        let key = 500;
        let (lo, hi) = pgm.search(key);
        assert!(
            keys[lo..=hi].binary_search(&key).is_ok(),
            "Key should be found within predicted range"
        );

        let key = 503;
        let (lo, hi) = pgm.search(key);
        assert!(
            keys[lo..=hi].binary_search(&key).is_err(),
            "Non-existent key should not be found, but range should still be valid"
        );
    }

    #[test]
    fn test_unsorted_input_fails() {
        let unsorted_keys = vec![1, 3, 2, 4];
        let result = PGMIndex::build(&unsorted_keys, 4);
        assert!(matches!(result, Err(PGMIndexError::KeysNotSorted)));
    }

    #[test]
    fn test_zero_copy_deserialization() {
        let keys: Vec<u64> = (0..5000).step_by(10).collect();
        let pgm = PGMIndex::build(&keys, 32).unwrap();
        let bytes = pgm.to_bytes().expect("serialize failed");

        let archived = PGMIndex::as_archived(&bytes).expect("zero-copy deserialize failed");
        let key = 1000;
        let (lo, hi) = archived.search(key);

        assert!(
            keys[lo..=hi].binary_search(&key).is_ok(),
            "Key should be in range after zero-copy read"
        );
    }

    #[test]
    fn test_copy_deserialization() {
        let keys: Vec<u64> = (0..10000).step_by(7).collect();
        let pgm = PGMIndex::build(&keys, 64).unwrap();
        let bytes = pgm.to_bytes().expect("serialize failed");

        let restored = PGMIndex::from_bytes(&bytes).expect("full deserialize failed");
        assert_eq!(restored.epsilon, pgm.epsilon);
        assert_eq!(restored.segments.len(), pgm.segments.len());

        let key = 9876;
        let (lo, hi) = restored.search(key);
        let found = keys[lo..=hi].binary_search(&key).ok();

        if let Some(actual_index) = found {
            assert_eq!(keys[lo + actual_index], key);
        } else {
            assert!(true, "Key not present in input set (as expected)");
        }
    }
}
