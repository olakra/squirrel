//! Chunked spatial grid with sleep and wake.
//!
//! The world is divided into square chunks of `CHUNK_SIZE` by `CHUNK_SIZE`
//! tiles. Each chunk counts the live entities (machines, flowing belts) on
//! its tiles. A chunk with a count above zero is `Active` and gets ticked. A
//! chunk with a count of zero is `Asleep` and is skipped until activity
//! arrives again.
//!
//! Ordering: `ChunkGrid` stores chunks in a `BTreeMap`, not a `HashMap`. The
//! standard `HashMap` seeds its hasher randomly per process, so its iteration
//! order changes between runs. That would make tick order and the state hash
//! non-reproducible. A `BTreeMap` always iterates in key order, which is the
//! sorted `(x, y)` order given by the derived `Ord` on `ChunkCoord`.

use crate::core::hash::Fnv1a;
use std::collections::BTreeMap;

/// Edge length of a chunk, in tiles.
pub const CHUNK_SIZE: i32 = 32;

/// Number of tiles in one chunk. Valid local indices are `0..CHUNK_TILES`.
pub const CHUNK_TILES: usize = (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);

/// A tile position in world space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

impl TileCoord {
    /// Index of this tile within its chunk, in row-major order.
    ///
    /// Works for negative coordinates: tile `-1` in x is local column 31 of
    /// chunk `-1`. The result is always in `0..CHUNK_TILES`.
    #[inline]
    pub fn local_index(self) -> usize {
        let col = self.x.rem_euclid(CHUNK_SIZE);
        let row = self.y.rem_euclid(CHUNK_SIZE);
        (row * CHUNK_SIZE + col) as usize
    }
}

/// Position of a chunk in the chunk grid.
///
/// The derived `Ord` compares `x` first, then `y`. This is the iteration
/// order of `ChunkGrid`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

impl ChunkCoord {
    /// Returns the chunk that contains `tile_x, tile_y`.
    ///
    /// Uses floor division, so negative tiles map to the chunk below zero.
    /// For example, tile `-1` maps to chunk `-1`, and tile `-33` maps to
    /// chunk `-2`. Truncating division would wrongly send `-1` to chunk `0`.
    #[inline]
    pub fn from_tile(tile_x: i32, tile_y: i32) -> ChunkCoord {
        ChunkCoord {
            x: tile_x.div_euclid(CHUNK_SIZE),
            y: tile_y.div_euclid(CHUNK_SIZE),
        }
    }
}

/// Whether a chunk is ticked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkState {
    /// At least one live entity is on the chunk. The chunk is ticked.
    Active,
    /// No live entities. The chunk is skipped until activity arrives.
    Asleep,
}

/// A single chunk's activity counter.
///
/// The count is the only stored state. `ChunkState` is derived from it, so
/// the two can never disagree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Chunk {
    live: u32,
}

impl Chunk {
    /// Number of live entities on this chunk.
    #[inline]
    pub fn entity_count(&self) -> u32 {
        self.live
    }

    /// Activity state derived from the entity count.
    #[inline]
    pub fn state(&self) -> ChunkState {
        if self.live > 0 {
            ChunkState::Active
        } else {
            ChunkState::Asleep
        }
    }

    /// Adds one live entity. Adding wakes the chunk.
    #[inline]
    fn acquire(&mut self) {
        self.live += 1;
    }

    /// Removes one live entity. Returns `false`, and changes nothing, when
    /// the count is already zero.
    #[inline]
    fn release(&mut self) -> bool {
        if self.live == 0 {
            return false;
        }
        self.live -= 1;
        true
    }
}

/// Sparse grid of chunks, keyed by chunk coordinate.
///
/// Chunks are created on demand. A chunk that has been created stays in the
/// grid after it sleeps, so the hash covers every chunk that has ever been
/// touched.
#[derive(Clone, Debug, Default)]
pub struct ChunkGrid {
    chunks: BTreeMap<ChunkCoord, Chunk>,
}

impl ChunkGrid {
    /// Creates an empty grid with no chunks.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates the chunk at `coord` if it does not exist. An existing chunk
    /// is left unchanged.
    pub fn ensure_chunk(&mut self, coord: ChunkCoord) {
        self.chunks.entry(coord).or_default();
    }

    /// Registers one live entity on the tile. Creates the chunk if needed.
    /// The chunk becomes `Active`.
    pub fn add_entity(&mut self, tile: TileCoord) {
        let coord = ChunkCoord::from_tile(tile.x, tile.y);
        self.chunks.entry(coord).or_default().acquire();
    }

    /// Unregisters one live entity on the tile. When the count reaches zero,
    /// the chunk becomes `Asleep`.
    ///
    /// Removing from an empty or missing chunk is a bug in the caller. It
    /// fails a `debug_assert!` in debug builds. In release builds it does
    /// nothing, so the count never underflows.
    pub fn remove_entity(&mut self, tile: TileCoord) {
        let coord = ChunkCoord::from_tile(tile.x, tile.y);
        let removed = self.chunks.get_mut(&coord).is_some_and(Chunk::release);
        debug_assert!(removed, "remove_entity on empty or missing chunk {coord:?}");
    }

    /// Returns the chunk at `coord`, if it exists.
    pub fn chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    /// Activity state of the chunk at `coord`. A chunk that was never
    /// created has no machinery, so it reports `Asleep`.
    pub fn state(&self, coord: ChunkCoord) -> ChunkState {
        self.chunk(coord).map_or(ChunkState::Asleep, Chunk::state)
    }

    /// Iterates over active chunk coordinates in sorted `(x, y)` order.
    pub fn active_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.chunks
            .iter()
            .filter(|(_, chunk)| chunk.state() == ChunkState::Active)
            .map(|(coord, _)| *coord)
    }

    /// Calls `f` once for each active chunk, in sorted `(x, y)` order.
    /// Asleep chunks are skipped.
    pub fn tick_active(&mut self, mut f: impl FnMut(ChunkCoord)) {
        for coord in self.active_chunks() {
            f(coord);
        }
    }

    /// Feeds every chunk's coordinate and count into `h`, in sorted order.
    /// Two grids with the same chunks and counts hash the same, regardless
    /// of insertion order.
    pub fn hash_into(&self, h: &mut Fnv1a) {
        h.write_u32(self.chunks.len() as u32);
        for (coord, chunk) in &self.chunks {
            // Casting a negative i32 to u32 keeps its bits, so the hash
            // still tells coordinates apart.
            h.write_u32(coord.x as u32);
            h.write_u32(coord.y as u32);
            h.write_u32(chunk.entity_count());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(x: i32, y: i32) -> TileCoord {
        TileCoord { x, y }
    }

    fn chunk_of(x: i32, y: i32) -> ChunkCoord {
        ChunkCoord::from_tile(x, y)
    }

    #[test]
    fn from_tile_positive_and_zero() {
        assert_eq!(chunk_of(0, 0), ChunkCoord { x: 0, y: 0 });
        assert_eq!(chunk_of(31, 31), ChunkCoord { x: 0, y: 0 });
        assert_eq!(chunk_of(32, 32), ChunkCoord { x: 1, y: 1 });
        assert_eq!(chunk_of(63, 64), ChunkCoord { x: 1, y: 2 });
    }

    #[test]
    fn from_tile_negative_uses_floor_division() {
        assert_eq!(chunk_of(-1, -1), ChunkCoord { x: -1, y: -1 });
        assert_eq!(chunk_of(-32, -32), ChunkCoord { x: -1, y: -1 });
        assert_eq!(chunk_of(-33, -33), ChunkCoord { x: -2, y: -2 });
        assert_eq!(chunk_of(-1, 0), ChunkCoord { x: -1, y: 0 });
    }

    #[test]
    fn local_index_covers_every_slot_once() {
        let mut seen = [false; CHUNK_TILES];
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let idx = tile(x, y).local_index();
                assert!(idx < CHUNK_TILES, "index {idx} out of range");
                assert!(!seen[idx], "index {idx} produced twice");
                seen[idx] = true;
            }
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn local_index_is_row_major_and_wraps_negative_tiles() {
        assert_eq!(tile(0, 0).local_index(), 0);
        assert_eq!(tile(31, 0).local_index(), 31);
        assert_eq!(tile(0, 1).local_index(), 32);
        assert_eq!(tile(31, 31).local_index(), 1023);
        // Tile -1 sits at local column 31 of chunk -1.
        assert_eq!(tile(-1, -1).local_index(), 1023);
        assert_eq!(tile(-32, -32).local_index(), 0);
    }

    #[test]
    fn chunk_sleeps_after_last_entity_removed() {
        let mut grid = ChunkGrid::new();
        let a = tile(3, 4);
        let b = tile(10, 10);
        grid.add_entity(a);
        grid.add_entity(b);
        let coord = chunk_of(3, 4);
        assert_eq!(grid.state(coord), ChunkState::Active);

        grid.remove_entity(a);
        assert_eq!(grid.state(coord), ChunkState::Active);
        assert_eq!(grid.chunk(coord).map(Chunk::entity_count), Some(1));

        grid.remove_entity(b);
        assert_eq!(grid.state(coord), ChunkState::Asleep);
        assert_eq!(grid.chunk(coord).map(Chunk::entity_count), Some(0));
    }

    #[test]
    fn add_wakes_a_sleeping_chunk() {
        let mut grid = ChunkGrid::new();
        let coord = chunk_of(40, 0);
        grid.ensure_chunk(coord);
        assert_eq!(grid.state(coord), ChunkState::Asleep);

        grid.add_entity(tile(40, 0));
        assert_eq!(grid.state(coord), ChunkState::Active);
    }

    #[test]
    fn active_chunks_are_sorted_regardless_of_insertion_order() {
        let order = [
            (5, -3),
            (-2, 7),
            (0, 0),
            (3, 3),
            (-1, -1),
            (40, 0),
            (0, -40),
        ];
        let mut grid = ChunkGrid::new();
        for (x, y) in order {
            grid.add_entity(tile(x * CHUNK_SIZE, y * CHUNK_SIZE));
        }

        let got: Vec<ChunkCoord> = grid.active_chunks().collect();
        let mut expected: Vec<ChunkCoord> =
            order.iter().map(|&(x, y)| ChunkCoord { x, y }).collect();
        expected.sort();
        assert_eq!(got, expected);

        let mut ticked = Vec::new();
        grid.tick_active(|c| ticked.push(c));
        assert_eq!(ticked, expected);
    }

    #[test]
    fn tick_active_skips_sleeping_chunks() {
        let mut grid = ChunkGrid::new();
        grid.add_entity(tile(0, 0));
        grid.add_entity(tile(100, 100));
        grid.remove_entity(tile(100, 100));

        let mut ticked = Vec::new();
        grid.tick_active(|c| ticked.push(c));
        assert_eq!(ticked, vec![ChunkCoord { x: 0, y: 0 }]);
    }

    fn hash_of(grid: &ChunkGrid) -> u64 {
        let mut h = Fnv1a::new();
        grid.hash_into(&mut h);
        h.finish()
    }

    #[test]
    fn hash_is_independent_of_insertion_order() {
        let entities = [tile(1, 1), tile(-40, 5), tile(70, -70), tile(1, 2)];

        let mut forward = ChunkGrid::new();
        for &t in &entities {
            forward.add_entity(t);
        }

        let mut backward = ChunkGrid::new();
        for &t in entities.iter().rev() {
            backward.add_entity(t);
        }

        assert_eq!(hash_of(&forward), hash_of(&backward));
    }

    #[test]
    fn hash_differs_when_state_differs() {
        let mut a = ChunkGrid::new();
        a.add_entity(tile(0, 0));
        let mut b = ChunkGrid::new();
        b.add_entity(tile(0, 0));
        assert_eq!(hash_of(&a), hash_of(&b));

        // A second entity on the same chunk changes the count.
        b.add_entity(tile(1, 1));
        assert_ne!(hash_of(&a), hash_of(&b));

        // A chunk in a different place changes the coordinate.
        let mut c = ChunkGrid::new();
        c.add_entity(tile(32, 0));
        assert_ne!(hash_of(&a), hash_of(&c));
    }

    #[test]
    fn hash_distinguishes_negative_coordinates() {
        let mut a = ChunkGrid::new();
        a.add_entity(tile(-1, 0));
        let mut b = ChunkGrid::new();
        b.add_entity(tile(1, 0));
        assert_ne!(hash_of(&a), hash_of(&b));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "remove_entity on empty or missing chunk")]
    fn remove_from_empty_chunk_panics_in_debug() {
        let mut grid = ChunkGrid::new();
        grid.add_entity(tile(0, 0));
        grid.remove_entity(tile(0, 0));
        grid.remove_entity(tile(0, 0));
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn remove_from_empty_chunk_is_noop_in_release() {
        let mut grid = ChunkGrid::new();
        grid.add_entity(tile(0, 0));
        grid.remove_entity(tile(0, 0));
        grid.remove_entity(tile(0, 0));
        assert_eq!(grid.chunk(chunk_of(0, 0)).map(Chunk::entity_count), Some(0));
    }
}
