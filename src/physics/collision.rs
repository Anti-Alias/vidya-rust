use bevy::{utils::HashMap};
use bevy::math::{Vec3, UVec3, UVec2, IVec3};

/// All of the terrain in a [`World`] at a given time as a resource.
pub struct Terrain {
    chunks: HashMap<ChunkCoords, Chunk>,
    piece_size: Vec3,
    chunk_size: UVec3
}
impl Terrain {

    /// Constructs a new [`Terrain`] instance.
    pub fn new(
        piece_size: Vec3,
        chunk_size: UVec3
    ) -> Self {
        if piece_size.x <= 0.0 || piece_size.y <= 0.0 || piece_size.z <= 0.0 {
            panic!("Invalid piece size");
        };
        if chunk_size.x == 0 || chunk_size.y == 0 || chunk_size.z == 0 {
            panic!("Invalid chunk size");
        };
        Self {
            chunks: HashMap::default(),
            piece_size,
            chunk_size
        }
    }

    /// Gets terrain piece at specified coords or None if the chunk it belongs to does not exist.
    pub fn get(&self, coords: Coords) -> Option<&TerrainPiece> {
        let (chunk_coords, chunk_idx) = self.to_indices(coords);
        let chunk = self.get_chunk(chunk_coords)?;
        Some(&chunk.0[chunk_idx])
    }

    /// Gets reference to terrain piece at specified coords.
    /// If the chunk it belongs to is not found, creates one with each value being [`TerrainPiece::Empty`]
    pub fn get_or_empty(&mut self, coords: Coords) -> &TerrainPiece {
        let (chunk_coords, chunk_idx) = self.to_indices(coords);
        let chunk = self.get_or_create_chunk(chunk_coords);
        &chunk.0[chunk_idx]
    }

    /// Gets mutable referenceto  terrain piece at specified coords.
    /// If the chunk it belongs to is not found, creates one with each value being [`TerrainPiece::Empty`]
    pub fn get_or_create_mut(&mut self, coords: Coords) -> &mut TerrainPiece {
        let (chunk_coords, chunk_idx) = self.to_indices(coords);
        let chunk = self.get_or_create_chunk(chunk_coords);
        &mut chunk.0[chunk_idx]
    }


    /// Iterates over chunks within a range.
    /// Both min and max are inclusive.
    /// If max values are < min values (max.x < min.x || max.y < min.y || max.z < min.z), iterator will be empty.
    pub fn iter_chunks<'terrain>(
        &'terrain self,
        min: ChunkCoords,
        max: ChunkCoords
    ) -> impl Iterator<Item=ChunkRef<'terrain>> {
        ChunkIter {
            terrain: self,
            min,
            max,
            pos: min
        }
    }

    /// Iterates over all terrain pieces within global range specified
    pub fn iter_pieces(&self, min: Coords, max: Coords) -> impl Iterator<Item=TerrainPieceRef> + '_ {
        let chunk_min = self.to_chunk_coords(min);
        let chunk_max = self.to_chunk_coords(max);
        self.iter_chunks(chunk_min, chunk_max)
            .flat_map(move |chunk| {
                let (inter_min, inter_max) = chunk.intersect(min, max);
                let local_min = UVec3::new(
                    (inter_min.x - chunk.position.x) as u32,
                    (inter_min.y - chunk.position.y) as u32,
                    (inter_min.z - chunk.position.z) as u32
                );
                let local_max = UVec3::new(
                    (inter_max.x - chunk.position.x) as u32,
                    (inter_max.y - chunk.position.y) as u32,
                    (inter_max.z - chunk.position.z) as u32
                );
                println!("min: {:?}, max: {:?}", min, max);
                println!("Inter min: {:?}, inter max: {:?}", inter_min, inter_max);
                println!("Chunk pos: {:?}!!!!!!!!!!!!!!", chunk.position);
                println!("Local min: {:?}, local max: {:?}", local_min, local_max);
                chunk.iter_pieces(local_min, local_max)
            })
            .filter(|piece| match piece.piece {
                TerrainPiece::Empty => false,
                _ => true
            })
    }

    /// Gets reference to chunk at specified coordinates.
    fn get_chunk(&self, coords: ChunkCoords) -> Option<&Chunk> {
        self.chunks.get(&coords)
    }

    /// Gets mutable reference to chunk at specified coordinates.
    fn get_or_create_chunk(&mut self, coords: ChunkCoords) -> &mut Chunk {
        self.chunks.entry(coords).or_insert_with(|| {
            let chunk_size = self.chunk_size.x * self.chunk_size.y * self.chunk_size.z;
            Chunk(vec![TerrainPiece::Empty; chunk_size as usize])
        })
    }

    // Converts global terrain piece coord and converts it into the chunk and index within the chunk
    fn to_indices(&self, coords: Coords) -> (ChunkCoords, usize) {
        let chunk_width = self.chunk_size.x as i32;
        let chunk_height = self.chunk_size.y as i32;
        let chunk_depth = self.chunk_size.z as i32;
        let chunk_coords = ChunkCoords {
            x: div(coords.x, chunk_width),
            y: div(coords.y, chunk_height),
            z: div(coords.z, chunk_depth)
        };
        let chunk_x = modulo(coords.x, chunk_width) as u32;
        let chunk_y = modulo(coords.y, chunk_height) as u32;
        let chunk_z = modulo(coords.z, chunk_depth) as u32;
        let idx = self.chunk_size.x * (self.chunk_size.y*chunk_z + chunk_y) + chunk_x;
        (chunk_coords, idx as usize)
    }

    /// Converts global coords to chunk coords
    fn to_chunk_coords(&self, coords: Coords) -> ChunkCoords {
        let chunk_width = self.chunk_size.x as i32;
        let chunk_height = self.chunk_size.y as i32;
        let chunk_depth = self.chunk_size.z as i32;
        ChunkCoords {
            x: div(coords.x, chunk_width),
            y: div(coords.y, chunk_height),
            z: div(coords.z, chunk_depth)
        }
    }
}

/// One piece of terrain
#[derive(Debug,Copy, Clone, Eq, PartialEq)]
pub enum TerrainPiece {
    Empty,
    Cuboid,
    Slope
}

/// Reference to terrain piece with context
#[derive(Debug,Copy, Clone, Eq, PartialEq)]
pub struct TerrainPieceRef<'terrain> {
    /// Terrain piece referenced
    pub piece: &'terrain TerrainPiece,

    /// Global coordinates of the terrain piece being iterated over
    pub coords: Coords
}

pub struct ChunkIter<'terrain> {
    terrain: &'terrain Terrain,
    pos: ChunkCoords,
    min: ChunkCoords,
    max: ChunkCoords,
}

impl<'terrain> ChunkIter<'terrain> {
    fn next_pos(pos: &ChunkCoords, min: &ChunkCoords, max: &ChunkCoords) -> ChunkCoords {
        let mut pos = *pos;
        pos.x += 1;
        if pos.x >= max.x {
            pos.x = min.x;
            pos.y += 1;
            if pos.y >= max.y {
                pos.y = min.y;
                pos.z += 1;
            }
        }
        pos
    }
}

impl<'terrain> Iterator for ChunkIter<'terrain> {
    type Item = ChunkRef<'terrain>;
    fn next(&mut self) -> Option<Self::Item> {

        // Quits early if past bounds
        if self.pos.z >= self.max.z {
            return None;
        }

        // Gets chunk at current location and updates position
        let (mut pos, min, max) = (self.pos, self.min, self.max);
        println!("Trying: {:?}", pos);
        let mut chunk = self.terrain.get_chunk(pos);
        while chunk.is_none() {
            pos = Self::next_pos(&pos, &min, &max);
            if pos.z >= self.max.z { return None; }
            println!("Trying inner: {:?}", pos);
            chunk = self.terrain.get_chunk(pos);
            println!("None inner: {}", chunk.is_none());
        }
        self.pos = Self::next_pos(&pos, &min, &max);

        // Unbounds reference and returns next chunk
        unsafe {
            let result_ptr = chunk.unwrap() as *const Chunk;
            Some(ChunkRef {
                chunk: &*result_ptr,
                position: Coords::new(
                    pos.x * self.terrain.chunk_size.x as i32,
                    pos.y * self.terrain.chunk_size.y as i32,
                    pos.z * self.terrain.chunk_size.z as i32
                ),
                size: self.terrain.chunk_size
            })
        }
    }
}

/// Chunk of terrain pieces
pub struct Chunk(Vec<TerrainPiece>);

#[derive(Copy, Clone)]
pub struct ChunkRef<'terrain> {
    pub chunk: &'terrain Chunk,
    pub position: Coords,
    pub size: UVec3
}

impl<'terrain> ChunkRef<'terrain> {
    fn intersect(&self, min: Coords, max: Coords) -> (Coords, Coords) {
        let this_min = self.position;
        let this_max = Coords::new(
            self.position.x + self.size.x as i32,
            self.position.y + self.size.y as i32,
            self.position.z + self.size.z as i32
        );
        let result_min = this_min.max(min);
        let mut result_max = this_max.min(max);
        if result_max.x < result_min.x {
            result_max.x = 0;
        }
        if result_max.y < result_min.y {
            result_max.y = 0;
        }
        if result_max.z < result_min.z {
            result_max.z = 0;
        }
        (
            result_min,
            result_max
        )
    }
}

pub struct ChunkRefIter<'terrain> {
    chunk: ChunkRef<'terrain>,
    min: UVec3,
    max: UVec3,
    pos: UVec3
}

impl<'terrain> ChunkRef<'terrain> {
    pub fn iter_pieces(self, min: UVec3, max: UVec3) -> ChunkRefIter<'terrain> {
        ChunkRefIter {
            chunk: self,
            min,
            max,
            pos: min
        }
    }
}

impl<'terrain> ChunkRefIter<'terrain> {
    pub fn new(chunk: ChunkRef<'terrain>, min: UVec3, max: UVec3) -> Self {
        Self {
            chunk,
            min,
            max,
            pos: min
        }
    }
}

impl<'terrain> Iterator for ChunkRefIter<'terrain> {
    type Item = TerrainPieceRef<'terrain>;
    fn next(&mut self) -> Option<Self::Item> {

        // Quits early if at end
        if self.pos.z >= self.max.z {
            return None;
        }

        // Gets current piece
        let min = &self.min;
        let chunk = &self.chunk;
        let size = &chunk.size;
        let idx = self.pos.z * (size.y + size.x) + self.pos.y * size.x + self.pos.x;
        let piece = &self.chunk.chunk.0[idx as usize];
        println!("Piece idx: {:?}, pos: {:?}, piece: {:?}", idx, self.pos, piece);

        // Advances position
        self.pos.x += 1;
        if self.pos.x >= self.max.x {
            self.pos.x = min.x;
            self.pos.y += 1;
            if self.pos.y >= self.max.y {
                self.pos.y = min.y;
                self.pos.z += 1;
            }
        }

        // Done
        Some(TerrainPieceRef {
            piece,
            coords: Coords::new(
                chunk.position.x + self.pos.x as i32,
                chunk.position.y + self.pos.y as i32,
                chunk.position.z + self.pos.z as i32
            )
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub y: i32,
    pub z: i32
}
impl ChunkCoords {
    pub fn new(x: i32, y: i32, z: i32) -> Self { Self { x, y, z} }
}

/// Global coordinates of terrain
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
    pub z: i32
}
impl Coords {
    pub fn new(x: i32, y: i32, z: i32) -> Self { Self { x, y, z} }
    pub fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z)
        )
    }
    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z)
        )
    }
    pub fn min_max(self, other: Self) -> (Self, Self) {
        let (min_x, max_x) = if self.x < other.x { (self.x, other.x) } else { (other.x, self.x) };
        let (min_y, max_y) = if self.y < other.y { (self.y, other.y) } else { (other.y, self.y) };
        let (min_z, max_z) = if self.z < other.z { (self.z, other.z) } else { (other.z, self.z) };
        (
            Coords {
                x: min_x,
                y: min_y,
                z: min_z
            },
            Coords {
                x: max_x,
                y: max_y,
                z: max_z
            }
        )
    }
}

/// Represents a selection of [`TerrainPiece`] instances across a [`Terrain`] instance.
/// Points a and b are inclusive
pub struct Selection {
    pub point_a: Coords,
    pub point_b: Coords
}
impl Selection {
    pub fn min_max(&self) -> (Coords, Coords) {
        self.point_a.min_max(self.point_b)
    }
}

fn div(a: i32, b: i32) -> i32 {
    if a >= 0 { a/b }
    else { (a-b) / b }
}

fn modulo(a: i32, b: i32) -> i32 {
    ((a%b) + b) % b
}


#[test]
fn test_div() {
    assert_eq!(0, div(1, 2));
    assert_eq!(1, div(2, 2));
    assert_eq!(-1, div(-1, 4));
    assert_eq!(-1, div(-3, 4));
    assert_eq!(-2, div(-4, 4));
}

#[test]
fn test_modulo() {
    assert_eq!(5, modulo(5, 10));
    assert_eq!(9, modulo(9, 10));
    assert_eq!(0, modulo(10, 10));
    assert_eq!(1, modulo(11, 10));
    assert_eq!(9, modulo(-1, 10));
}

#[test]
fn test_insertion() {
    let mut terrain = Terrain::new(
        Vec3::new(32.0, 32.0, 32.0),
        UVec3::new(16, 16, 16)
    );
    let piece = terrain.get_or_create_mut(Coords::new(0, 0, 0));
    *piece = TerrainPiece::Cuboid;
    let piece = terrain.get_or_create_mut(Coords::new(10, 11, 12));
    *piece = TerrainPiece::Slope;
    let piece = terrain.get_or_create_mut(Coords::new(17, -18, 19));
    *piece = TerrainPiece::Cuboid;
    assert_eq!(TerrainPiece::Empty, *terrain.get_or_create_mut(Coords::new(1, 0, 0)));
    assert_eq!(TerrainPiece::Empty, *terrain.get_or_create_mut(Coords::new(0, 1, 0)));
    assert_eq!(TerrainPiece::Empty, *terrain.get_or_create_mut(Coords::new(0, 0, 1)));
    assert_eq!(TerrainPiece::Cuboid, *terrain.get_or_create_mut(Coords::new(0, 0, 0)));
    assert_eq!(TerrainPiece::Slope, *terrain.get_or_create_mut(Coords::new(10, 11, 12)));
    assert_eq!(TerrainPiece::Cuboid, *terrain.get_or_create_mut(Coords::new(17, -18, 19)));
    assert_eq!(TerrainPiece::Empty, *terrain.get_or_create_mut(Coords::new(-100, -101, 102)));
}

#[test]
fn test_insertion_and_get() {
    let mut terrain = Terrain::new(
        Vec3::new(32.0, 32.0, 32.0),
        UVec3::new(16, 16, 16)
    );
    let piece = terrain.get_or_create_mut(Coords::new(0, 0, 0));
    *piece = TerrainPiece::Cuboid;
    let piece = terrain.get_or_create_mut(Coords::new(10, 11, 12));
    *piece = TerrainPiece::Slope;
    let piece = terrain.get_or_create_mut(Coords::new(17, -18, 19));
    *piece = TerrainPiece::Cuboid;
    assert_eq!(Some(&TerrainPiece::Empty), terrain.get(Coords::new(1, 0, 0)));
    assert_eq!(Some(&TerrainPiece::Empty), terrain.get(Coords::new(0, 1, 0)));
    assert_eq!(Some(&TerrainPiece::Empty), terrain.get(Coords::new(0, 0, 1)));
    assert_eq!(Some(&TerrainPiece::Cuboid), terrain.get(Coords::new(0, 0, 0)));
    assert_eq!(Some(&TerrainPiece::Slope), terrain.get(Coords::new(10, 11, 12)));
    assert_eq!(Some(&TerrainPiece::Cuboid), terrain.get(Coords::new(17, -18, 19)));
    assert_eq!(None, terrain.get(Coords::new(-100, -101, 102)));
}

#[test]
fn test_chunk_iter() {

    #[derive(Debug, Eq, PartialEq)]
    struct ChunkInfo {
        pos: Coords,
        size: UVec3
    }

    let mut terrain = Terrain::new(
        Vec3::new(32.0, 32.0, 32.0),
        UVec3::new(16, 16, 16)
    );
    terrain.get_or_create_mut(Coords::new(0, 0, 0));
    terrain.get_or_create_mut(Coords::new(-1, 0, 1));
    terrain.get_or_create_mut(Coords::new(0, 0, -1));

    let actual: Vec<ChunkInfo> = terrain.iter_chunks(
        ChunkCoords::new(-2, 0, -1),
        ChunkCoords::new(1, 1, 2)
    )
    .map(|chunk| ChunkInfo {
        pos: chunk.position,
        size: chunk.size
    })
    .collect();
    
    let expected = vec![
        ChunkInfo { pos: Coords { x: 0, y: 0, z: -16 }, size: UVec3::new(16, 16, 16) },
        ChunkInfo { pos: Coords { x: -16, y: 0, z: 0 }, size: UVec3::new(16, 16, 16) },
        ChunkInfo { pos: Coords { x: 0, y: 0, z: 0 }, size: UVec3::new(16, 16, 16) }
    ];
    assert_eq!(expected, actual);
}

#[test]
fn test_selection() {
    let mut terrain = Terrain::new(
        Vec3::new(32.0, 32.0, 32.0),
        UVec3::new(16, 16, 16)
    );
    let piece = terrain.get_or_create_mut(Coords::new(5, 5, 5));
    *piece = TerrainPiece::Cuboid;
    
    let actual: Vec<TerrainPiece> = terrain
        .iter_pieces(Coords::new(4, 4, 4), Coords::new(6, 6, 6))
        .map(|piece| *piece.piece)
        .collect();
    
    let expected = vec![TerrainPiece::Cuboid];
    assert_eq!(expected, actual);
}