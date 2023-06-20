/*
  USING LEFT HANDED SYSTEM (... i know...)

    Data tables for marching squares


    Vertices have the following IDs
   0        2
    --------
    |      |
    |      |       ^-Z
    |      |       |
    --------       --> X
   1        3

    (divided by two from d[i] indices, since we're
     looking at a flat ASDF and we don't care about z)


    Edges are numbered as follows:
       0
    --------
    |      |
  2 |      | 3     ^-Z
    |      |       |
    --------       --> X
       1

*/

#[derive(EnumIter, EnumCount, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face2DVertex {
    LeftUp = 0,
    LeftDown = 1,
    RightUp = 2,
    RightDown = 3,
}

#[derive(EnumIter, EnumCount, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face2DEdge {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

// 0        2
//  --------
//  |      |
//  |      |       ^-Z
//  |      |       |
//  --------       --> X
// 1        3
//

// For a given set of filled corners, this array defines
// the cell edges from which we draw interior edges
#[rustfmt::skip]
pub const EDGE_MAP: [[[Option<Face2DEdge>; 2]; 2]; 16] = [
    [[None, None], [None, None]],                                                                          // ----
    [[Some(Face2DEdge::Up), Some(Face2DEdge::Left)], [None, None]],                                        // ---0
    [[Some(Face2DEdge::Left), Some(Face2DEdge::Down)], [None, None]],                                      // --1-
    [[Some(Face2DEdge::Up), Some(Face2DEdge::Down)], [None, None]],                                        // --10
    [[Some(Face2DEdge::Right), Some(Face2DEdge::Up)], [None, None]],                                       // -2--
    [[Some(Face2DEdge::Right), Some(Face2DEdge::Left)], [None, None]],                                     // -2-0
    [[Some(Face2DEdge::Right), Some(Face2DEdge::Up)], [Some(Face2DEdge::Left), Some(Face2DEdge::Down)]],   // -21- //ambig
    [[Some(Face2DEdge::Right), Some(Face2DEdge::Down)], [None, None]],                                     // -210
    [[Some(Face2DEdge::Down), Some(Face2DEdge::Right)], [None, None]],                                     // 3---
    [[Some(Face2DEdge::Down), Some(Face2DEdge::Right)], [Some(Face2DEdge::Up), Some(Face2DEdge::Left)]],   // 3--0 //ambig // 可以通过法线来判断
    [[Some(Face2DEdge::Left), Some(Face2DEdge::Right)], [None, None]],                                     // 3-1-
    [[Some(Face2DEdge::Up), Some(Face2DEdge::Right)], [None, None]],                                       // 3-10
    [[Some(Face2DEdge::Down), Some(Face2DEdge::Up)], [None, None]],                                        // 32--
    [[Some(Face2DEdge::Down), Some(Face2DEdge::Left)], [None, None]],                                      // 32-0
    [[Some(Face2DEdge::Left), Some(Face2DEdge::Up)], [None, None]],                                        // 321-
    [[None, None], [None, None]],                                                                          // 3210
];

// Indexed by edge number, returns vertex index
//
pub const VERTEX_MAP: [[i8; 2]; 4] = [[0, 2], [3, 1], [1, 0], [2, 3]];

/*

 Vertex and Edge Index Map

       2-------0------6
      /.             /|
     10.           11 |
    /  2           /  3
   /   .          /   |     ^ Y
  3-------5------7    |     |
  |    0 . . 1 . |. . 4     --> X
  |   .          |   /     /
  6  8           7  9     / z
  | .            | /     |/
  |.             |/
  1-------4------5


     Face Index Map

         -----
         | 0 |
     -----------------     ^ -z
     | 5 | 2 | 4 | 3 |     |
     -----------------     ---> x
         | 1 |
         -----

     Face Index Layout
       o--------------o
      /.             /|
     / .            / |
    /  .    3      /  |
   /   .      (0) /   |     ^ Y
  o--------------o  5 |     |
  |(4) . . . . . |. . o     --> X
  |   .   1      |   /
  |  .           |  /
  | .      (2)   | /
  |.             |/
  o--------------o



 Cell Point and Subcell Layout

     (2)o--------------o(6)
       /.             /|
      / .            / |
     /  .           /  |
    /   .          /   |     ^ Y
(3)o--------------o(7) |     |
   | (0). . . . . |. . o(4)  --> X
   |   .          |   /
   |  .           |  /
   | .            | /
   |.             |/
(1)o--------------o(5)

*/

use strum::EnumCount;
use strum_macros::EnumIter;

// also face
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, EnumCount)]
pub enum FaceIndex {
    Back = 0,
    Front = 1,
    Bottom = 2,
    Top = 3,
    Left = 4,
    Right = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter, EnumCount)]
pub enum EdgeIndex {
    XAxisTopBack = 0,
    XAxisBottomBack = 1,
    LeftYAxisBack = 2,
    RightYAxisBack = 3,
    XAxisBottomFront = 4,
    XAxisTopFront = 5,
    LeftYAxisFront = 6,
    RightYAxisFront = 7,
    LeftBottomZAxis = 8,
    RightBottomZAxis = 9,
    LeftTopZAxis = 10,
    RightTopZAxis = 11,
}

// x, y, z => xyz
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, EnumCount)]
pub enum VertexPoint {
    LeftBottomBack = 0,
    LeftBottomFront = 1,
    LeftTopBack = 2,
    LeftTopFront = 3,
    RightBottomBack = 4,
    RightBottomFront = 5,
    RightTopBack = 6,
    RightTopFront = 7,
}

// x, y, z => xyz
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, EnumCount)]
pub enum SubCellIndex {
    LeftBottomBack = 0,
    LeftBottomFront = 1,
    LeftTopBack = 2,
    LeftTopFront = 3,
    RightBottomBack = 4,
    RightBottomFront = 5,
    RightTopBack = 6,
    RightTopFront = 7,
}

// x, y, z => xyz
#[derive(Debug, Clone, Copy, PartialEq, EnumIter, EnumCount)]
pub enum Direction {
    XAxis = 0,
    YAxis = 1,
    ZAxis = 2,
}

//----------------------------------------------

// FACE_VERTEX[i] gives the four vertices (as 3-bit corner indices)
// that define face i on an octree cell
//
pub const FACE_VERTEX: [[VertexPoint; Face2DVertex::COUNT]; FaceIndex::COUNT] = [
    [
        VertexPoint::LeftTopBack,
        VertexPoint::LeftBottomBack,
        VertexPoint::RightTopBack,
        VertexPoint::RightBottomBack,
    ], // face 0
    [
        VertexPoint::LeftBottomFront,
        VertexPoint::LeftTopFront,
        VertexPoint::RightBottomFront,
        VertexPoint::RightTopFront,
    ], // face 1
    [
        VertexPoint::LeftBottomBack,
        VertexPoint::LeftBottomFront,
        VertexPoint::RightBottomBack,
        VertexPoint::RightBottomFront,
    ], // face 2
    [
        VertexPoint::RightTopBack,
        VertexPoint::RightTopFront,
        VertexPoint::LeftTopBack,
        VertexPoint::LeftTopFront,
    ], // face 3
    [
        VertexPoint::LeftTopBack,
        VertexPoint::LeftTopFront,
        VertexPoint::LeftBottomBack,
        VertexPoint::LeftBottomFront,
    ], // face 4
    [
        VertexPoint::RightBottomBack,
        VertexPoint::RightBottomFront,
        VertexPoint::RightTopBack,
        VertexPoint::RightTopFront,
    ], // face 5
];

// Given an edge it gives back the two
// cell vertices that connect it
// @ at pos 0 and 1 - the order will always
// be in the positive direction...
//
pub const EDGE_VERTICES: [[VertexPoint; 2]; EdgeIndex::COUNT] = [
    [VertexPoint::LeftTopBack, VertexPoint::RightTopBack], // edge 0
    [VertexPoint::LeftBottomBack, VertexPoint::RightBottomBack], // edge 1
    [VertexPoint::LeftBottomBack, VertexPoint::LeftTopBack], // edge 2
    [VertexPoint::RightBottomBack, VertexPoint::RightTopBack], // edge 3
    [VertexPoint::LeftBottomFront, VertexPoint::RightBottomFront], // edge 4
    [VertexPoint::LeftTopFront, VertexPoint::RightTopFront], // edge 5
    [VertexPoint::LeftBottomFront, VertexPoint::LeftTopFront], // edge 6
    [VertexPoint::RightBottomFront, VertexPoint::RightTopFront], // edge 7
    [VertexPoint::LeftBottomBack, VertexPoint::LeftBottomFront], // edge 8
    [VertexPoint::RightBottomBack, VertexPoint::RightBottomFront], // edge 9
    [VertexPoint::LeftTopBack, VertexPoint::LeftTopFront], // edge 10
    [VertexPoint::RightTopBack, VertexPoint::RightTopFront], // edge 11
];

pub const EDGE_DIRECTION: [Direction; EdgeIndex::COUNT] = [
    Direction::XAxis,
    Direction::XAxis,
    Direction::YAxis,
    Direction::YAxis,
    Direction::XAxis,
    Direction::XAxis,
    Direction::YAxis,
    Direction::YAxis,
    Direction::ZAxis,
    Direction::ZAxis,
    Direction::ZAxis,
    Direction::ZAxis,
];

pub const FACE_DIRECTION: [Direction; FaceIndex::COUNT] = [
    Direction::ZAxis,
    Direction::ZAxis,
    Direction::YAxis,
    Direction::YAxis,
    Direction::XAxis,
    Direction::XAxis,
];

/// The table for face twins.
/// Twins是两个相邻的Cell的重叠的面，
pub const FACE_TWIN_TABLE: [[FaceIndex; 2]; FaceIndex::COUNT] = [
    [FaceIndex::Back, FaceIndex::Front],
    [FaceIndex::Bottom, FaceIndex::Top],
    [FaceIndex::Left, FaceIndex::Right],
    [FaceIndex::Right, FaceIndex::Left],
    [FaceIndex::Top, FaceIndex::Bottom],
    [FaceIndex::Front, FaceIndex::Back],
];

/// @brief Cell neighbour table
/// See: 'Cell Point and Subcell Layout' in tables header
///
/// 在对应的方向上，当前Cell的ID，对应的相邻Cell的ID
pub const NEIGHBOUR_ADDRESS_TABLE: [[SubCellIndex; SubCellIndex::COUNT]; Direction::COUNT] = [
    [
        SubCellIndex::LeftBottomFront,
        SubCellIndex::LeftBottomBack,
        SubCellIndex::LeftTopFront,
        SubCellIndex::LeftTopBack,
        SubCellIndex::RightBottomFront,
        SubCellIndex::RightBottomBack,
        SubCellIndex::RightTopFront,
        SubCellIndex::RightTopBack,
    ], // Z (BACK & FRONT) NEIGHBOUR
    [
        SubCellIndex::LeftTopBack,
        SubCellIndex::LeftTopFront,
        SubCellIndex::LeftBottomBack,
        SubCellIndex::LeftBottomFront,
        SubCellIndex::RightTopBack,
        SubCellIndex::RightTopFront,
        SubCellIndex::RightBottomBack,
        SubCellIndex::RightBottomFront,
    ], // Y (TOP & BOTTOM) NEIGHBOUR
    [
        SubCellIndex::RightBottomBack,
        SubCellIndex::RightBottomFront,
        SubCellIndex::RightTopBack,
        SubCellIndex::RightTopFront,
        SubCellIndex::LeftBottomBack,
        SubCellIndex::LeftBottomFront,
        SubCellIndex::LeftTopBack,
        SubCellIndex::LeftTopFront,
    ], // X (LEFT & RIGHT) NEIGHBOUR
];

/// @brief neighbour face table between sub cell
pub const FACE_RELATIONSHIP_TABLE: [[FaceIndex; 3]; SubCellIndex::COUNT] = [
    [FaceIndex::Back, FaceIndex::Bottom, FaceIndex::Left],
    [FaceIndex::Front, FaceIndex::Bottom, FaceIndex::Left],
    [FaceIndex::Back, FaceIndex::Top, FaceIndex::Left],
    [FaceIndex::Front, FaceIndex::Top, FaceIndex::Left],
    [FaceIndex::Back, FaceIndex::Bottom, FaceIndex::Right],
    [FaceIndex::Front, FaceIndex::Bottom, FaceIndex::Right],
    [FaceIndex::Back, FaceIndex::Top, FaceIndex::Right],
    [FaceIndex::Front, FaceIndex::Top, FaceIndex::Right],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount)]
pub enum SubFaceIndex {
    LeftUp = 0,
    RightUp = 1,
    LeftDown = 2,
    RightDown = 3,
}

//
//      2-------0------6
//     /.             /|
//    10.           11 |
//   /  2           /  3
//  /   .          /   |     ^ Y
// 3-------5------7    |     |
// |    0 . . 1 . |. . 4     --> X
// |   .          |   /     /
// 6  8           7  9     / z
// | .            | /     |/
// |.             |/
// 1-------4------5
//

/// @brief subcell的Face在父Cell的Face的位置。
/// 从Face的正面看去，SubFace(SubFaceIndex)的顺序
///   _______________
///  |       |      |
///  |   0   |  1   |
///  |_______|______|
///  |       |      |
///  |   2   |  3   |
///  |_______|______|
pub const SUB_FACE_TABLE: [[SubFaceIndex; 3]; SubCellIndex::COUNT] = [
    [
        SubFaceIndex::RightDown,
        SubFaceIndex::LeftDown,
        SubFaceIndex::LeftDown,
    ],
    [
        SubFaceIndex::LeftDown,
        SubFaceIndex::LeftUp,
        SubFaceIndex::RightDown,
    ],
    [
        SubFaceIndex::RightUp,
        SubFaceIndex::LeftUp,
        SubFaceIndex::LeftUp,
    ],
    [
        SubFaceIndex::LeftUp,
        SubFaceIndex::LeftDown,
        SubFaceIndex::RightUp,
    ],
    [
        SubFaceIndex::LeftDown,
        SubFaceIndex::RightDown,
        SubFaceIndex::RightDown,
    ],
    [
        SubFaceIndex::RightDown,
        SubFaceIndex::RightUp,
        SubFaceIndex::LeftDown,
    ],
    [
        SubFaceIndex::LeftUp,
        SubFaceIndex::RightUp,
        SubFaceIndex::RightUp,
    ],
    [
        SubFaceIndex::RightUp,
        SubFaceIndex::RightDown,
        SubFaceIndex::LeftUp,
    ],
];
