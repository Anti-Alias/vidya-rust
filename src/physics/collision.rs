use bevy::utils::HashMap;

/// One piece of terrain
#[derive(Debug,Copy, Clone, Eq, PartialEq)]
pub enum TerrainPiece {
    Empty,
    Cuboid,
    Slope
}

/// Reference to a [`TerrainPiece`] with context
pub struct TerrainPieceRef<'t> {
    pub piece: &'t mut TerrainPiece,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    terrain: &'t Terrain,
}

/// Chunk of terrain pieces
pub struct TerrainChunk(Vec<TerrainPiece>);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TerrainChunkCoords {
    pub x: i32,
    pub y: i32,
    pub z: i32
}
impl TerrainChunkCoords {
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

/// All of the terrain in a [`World`] at a given time as a resource.
pub struct Terrain {
    chunks: HashMap<TerrainChunkCoords, TerrainChunk>,
    piece_width: f32,
    piece_height: f32,
    piece_depth: f32,
    chunk_width: u32,
    chunk_height: u32,
    chunk_depth: u32
}
impl Terrain {

    /// Constructs a new [`Terrain`] instance.
    pub fn new(
        piece_width: f32,
        piece_height: f32,
        piece_depth: f32,
        chunk_width: u32,
        chunk_height: u32,
        chunk_depth: u32
    ) -> Self {
        if piece_width <= 0.0 || piece_height <= 0.0 || piece_depth <= 0.0 {
            panic!("Invalid piece size");
        };
        if chunk_width == 0 || chunk_height == 0 || chunk_depth == 0 {
            panic!("Invalid chunk size");
        };
        Self {
            chunks: HashMap::default(),
            piece_width,
            piece_height,
            piece_depth,
            chunk_width,
            chunk_height,
            chunk_depth
        }
    }

    /// Adds a single terrain piece using global coordinates
    pub fn get_terrain_piece(&mut self, coords: Coords) -> &mut TerrainPiece {

        let chunk_width = self.chunk_width as i32;
        let chunk_height = self.chunk_height as i32;
        let chunk_depth = self.chunk_depth as i32;

        // Gets coordinates of chunk
        let chunk_coords = TerrainChunkCoords {
            x: coords.x / chunk_width,
            y: coords.y / chunk_height,
            z: coords.z / chunk_depth
        };

        // Gets coordinates in chunk
        let chunk_x = modulo(coords.x, chunk_width) as u32;
        let chunk_y = modulo(coords.y, chunk_height) as u32;
        let chunk_z = modulo(coords.z, chunk_depth) as u32;

        // Gets or creates chunk and returns reference to slot
        let idx = self.chunk_width * (self.chunk_height*chunk_z + chunk_y) + chunk_x;
        let chunk = self.get_or_create_chunk(chunk_coords);
        &mut chunk.0[idx as usize]
    }

    pub fn iter_mut(&mut self) {

    }

    /// Gets reference to chunk at specified coordinates.
    fn get_chunk(&mut self, coords: TerrainChunkCoords) -> Option<&mut TerrainChunk> {
        self.chunks.get_mut(&coords)
    }

    /// Gets mutable reference to chunk at specified coordinates.
    fn get_chunk_mut(&mut self, coords: TerrainChunkCoords) -> Option<&mut TerrainChunk> {
        self.chunks.get_mut(&coords)
    }

    /// Gets mutable reference to chunk at specified coordinates.
    fn get_or_create_chunk(&mut self, coords: TerrainChunkCoords) -> &mut TerrainChunk {
        self.chunks.entry(coords).or_insert_with(|| {
            let chunk_size = self.chunk_width * self.chunk_height * self.chunk_depth;
            TerrainChunk(vec![TerrainPiece::Empty; chunk_size as usize])
        })
    }

    
}

fn modulo(a: i32, b: i32) -> i32 {
    ((a%b) + b) % b
}


#[test]
fn test_insertion() {
    let mut terrain = Terrain::new(
        32.0,
        32.0,
        32.0,
        16,
        16,
        16
    );
    let piece = terrain.get_terrain_piece(Coords::new(0, 0, 0));
    *piece = TerrainPiece::Cuboid;
    let piece = terrain.get_terrain_piece(Coords::new(10, 11, 12));
    *piece = TerrainPiece::Slope;
    let piece = terrain.get_terrain_piece(Coords::new(17, -18, 19));
    *piece = TerrainPiece::Cuboid;
    assert_eq!(TerrainPiece::Empty, *terrain.get_terrain_piece(Coords::new(1, 0, 0)));
    assert_eq!(TerrainPiece::Empty, *terrain.get_terrain_piece(Coords::new(0, 1, 0)));
    assert_eq!(TerrainPiece::Empty, *terrain.get_terrain_piece(Coords::new(0, 0, 1)));
    assert_eq!(TerrainPiece::Cuboid, *terrain.get_terrain_piece(Coords::new(0, 0, 0)));
    assert_eq!(TerrainPiece::Slope, *terrain.get_terrain_piece(Coords::new(10, 11, 12)));
    assert_eq!(TerrainPiece::Cuboid, *terrain.get_terrain_piece(Coords::new(17, -18, 19)));
}