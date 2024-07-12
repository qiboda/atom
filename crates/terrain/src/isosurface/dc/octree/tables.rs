/*
  USING Right HANDED SYSTEM like bevy engine

  如果没有做说明，默认排序为x,y,z，从小到大, 从右到左。
*/

/*

 Vertex and Edge Index Map

       2-------1------3
      /.             /|
     10.           11 |
    /  4           /  5
   /   .          /   |     ^ Y
  6-------3------7    |     |
  |    0 . . 0 . |. . 1     --> X
  |   .          |   /     /
  6  8           7  9     / z
  | .            | /     |/
  |.             |/
  4-------2------5

*/

use bevy::reflect::Reflect;
use strum::{EnumCount, FromRepr};
use strum_macros::EnumIter;

bitflags::bitflags! {
    /**
     *  If Vertex or edge or face is axis left/buttom/back, then the value is 0, other than 1.
     */
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AxisValue: u8 {
        const Zero = 0b000;
        const One = 0b001;
    }
}

type XAxisValue = AxisValue;
type YAxisValue = AxisValue;
type ZAxisValue = AxisValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, FromRepr, Reflect)]
pub enum AxisType {
    XAxis = 0,
    YAxis = 1,
    ZAxis = 2,
}

const fn get_axis_value(
    x_axis_value: XAxisValue,
    y_axis_value: YAxisValue,
    z_axis_value: ZAxisValue,
) -> isize {
    (x_axis_value.bits() + (y_axis_value.bits() << 1) + (z_axis_value.bits() << 2)) as isize
}

/**
 * Edge Index Map
 *
 * Axis is edge direction
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Hash)]
pub enum EdgeIndex {
    #[allow(clippy::identity_op, clippy::erasing_op)]
    XAxisY0Z0 = 4 * 0 + 1 * 0 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    XAxisY1Z0 = 4 * 0 + 1 * 1 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    XAxisY0Z1 = 4 * 0 + 1 * 0 + 2 * 1,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    XAxisY1Z1 = 4 * 0 + 1 * 1 + 2 * 1,

    #[allow(clippy::identity_op, clippy::erasing_op)]
    YAxisX0Z0 = 4 * 1 + 1 * 0 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    YAxisX1Z0 = 4 * 1 + 1 * 1 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    YAxisX0Z1 = 4 * 1 + 1 * 0 + 2 * 1,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    YAxisX1Z1 = 4 * 1 + 1 * 1 + 2 * 1,

    #[allow(clippy::identity_op, clippy::erasing_op)]
    ZAxisX0Y0 = 4 * 2 + 1 * 0 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    ZAxisX1Y0 = 4 * 2 + 1 * 1 + 2 * 0,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    ZAxisX0Y1 = 4 * 2 + 1 * 0 + 2 * 1,
    #[allow(clippy::identity_op, clippy::erasing_op)]
    ZAxisX1Y1 = 4 * 2 + 1 * 1 + 2 * 1,
}

// x, y, z => xyz
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, Hash)]
pub enum VertexIndex {
    X0Y0Z0 = get_axis_value(XAxisValue::Zero, YAxisValue::Zero, ZAxisValue::Zero),
    X1Y0Z0 = get_axis_value(XAxisValue::One, YAxisValue::Zero, ZAxisValue::Zero),
    X0Y1Z0 = get_axis_value(XAxisValue::Zero, YAxisValue::One, ZAxisValue::Zero),
    X1Y1Z0 = get_axis_value(XAxisValue::One, YAxisValue::One, ZAxisValue::Zero),
    X0Y0Z1 = get_axis_value(XAxisValue::Zero, YAxisValue::Zero, ZAxisValue::One),
    X1Y0Z1 = get_axis_value(XAxisValue::One, YAxisValue::Zero, ZAxisValue::One),
    X0Y1Z1 = get_axis_value(XAxisValue::Zero, YAxisValue::One, ZAxisValue::One),
    X1Y1Z1 = get_axis_value(XAxisValue::One, YAxisValue::One, ZAxisValue::One),
}

impl VertexIndex {
    pub fn to_array(&self) -> [u32; 3] {
        match self {
            VertexIndex::X0Y0Z0 => [0, 0, 0],
            VertexIndex::X1Y0Z0 => [1, 0, 0],
            VertexIndex::X0Y1Z0 => [0, 1, 0],
            VertexIndex::X1Y1Z0 => [1, 1, 0],
            VertexIndex::X0Y0Z1 => [0, 0, 1],
            VertexIndex::X1Y0Z1 => [1, 0, 1],
            VertexIndex::X0Y1Z1 => [0, 1, 1],
            VertexIndex::X1Y1Z1 => [1, 1, 1],
        }
    }

    pub fn from_repr(repr: usize) -> Option<Self> {
        match repr {
            0 => Some(VertexIndex::X0Y0Z0),
            1 => Some(VertexIndex::X1Y0Z0),
            2 => Some(VertexIndex::X0Y1Z0),
            3 => Some(VertexIndex::X1Y1Z0),
            4 => Some(VertexIndex::X0Y0Z1),
            5 => Some(VertexIndex::X1Y0Z1),
            6 => Some(VertexIndex::X0Y1Z1),
            7 => Some(VertexIndex::X1Y1Z1),
            _ => None,
        }
    }
}

/// 边连接的顶点
pub const EDGE_VERTEX_PAIRS: [[VertexIndex; 2]; EdgeIndex::COUNT] = [
    // x axis
    [VertexIndex::X0Y0Z0, VertexIndex::X1Y0Z0],
    [VertexIndex::X0Y1Z0, VertexIndex::X1Y1Z0],
    [VertexIndex::X0Y0Z1, VertexIndex::X1Y0Z1],
    [VertexIndex::X0Y1Z1, VertexIndex::X1Y1Z1],
    // y axis
    [VertexIndex::X0Y0Z0, VertexIndex::X0Y1Z0],
    [VertexIndex::X1Y0Z0, VertexIndex::X1Y1Z0],
    [VertexIndex::X0Y0Z1, VertexIndex::X0Y1Z1],
    [VertexIndex::X1Y0Z1, VertexIndex::X1Y1Z1],
    // z axis
    [VertexIndex::X0Y0Z0, VertexIndex::X0Y0Z1],
    [VertexIndex::X1Y0Z0, VertexIndex::X1Y0Z1],
    [VertexIndex::X0Y1Z0, VertexIndex::X0Y1Z1],
    [VertexIndex::X1Y1Z0, VertexIndex::X1Y1Z1],
];

pub type SubCellIndex = VertexIndex;

//  Cell Point and Subcell Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
//(110)o------------o(111)|     |
//    |(000)o. . . . |. . o(001)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (100)o--------------o(101)
//
// 连接两个Cell的Face，根据方向轴，找到分解的四个面的两侧的SubCellIndex
// 在输入轴方向，从小到大，连接的SubCellIndex， ==0表示在输入轴的左边， ==1表示在输入轴的右边
pub const FACES_SUBCELLS_NEIGHBOUR_PAIRS: [[(SubCellIndex, SubCellIndex); 4]; AxisType::COUNT] = [
    // x axis
    [
        (SubCellIndex::X1Y0Z0, SubCellIndex::X0Y0Z0),
        (SubCellIndex::X1Y1Z0, SubCellIndex::X0Y1Z0),
        (SubCellIndex::X1Y0Z1, SubCellIndex::X0Y0Z1),
        (SubCellIndex::X1Y1Z1, SubCellIndex::X0Y1Z1),
    ],
    // y axis
    [
        (SubCellIndex::X0Y1Z0, SubCellIndex::X0Y0Z0),
        (SubCellIndex::X1Y1Z0, SubCellIndex::X1Y0Z0),
        (SubCellIndex::X0Y1Z1, SubCellIndex::X0Y0Z1),
        (SubCellIndex::X1Y1Z1, SubCellIndex::X1Y0Z1),
    ],
    // z axis
    [
        (SubCellIndex::X0Y0Z1, SubCellIndex::X0Y0Z0),
        (SubCellIndex::X1Y0Z1, SubCellIndex::X1Y0Z0),
        (SubCellIndex::X0Y1Z1, SubCellIndex::X0Y1Z0),
        (SubCellIndex::X1Y1Z1, SubCellIndex::X1Y1Z0),
    ],
];

//  Cell Point and Subcell Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |     ^ Y
//     /   .          /   |     |
//(110)o-------------o(111)     |
//    |(000)o. . . . |. . o(001)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (100)o--------------o(101)
//
// 在一个Cell中，坐标轴垂直的内部的四个面，所对应的SubCellIndex
// 根据输入轴，找到面相邻的四个SubCellIndex
// 轴垂直穿过的Cell中间内部的面, 负半轴的SubCellIndex,在tuple的左边, 正半轴的SubCellIndex,在tuple的右边
pub const SUBCELL_FACES_NEIGHBOUR_PAIRS: [[(SubCellIndex, SubCellIndex); 4]; AxisType::COUNT] = [
    // x axis
    [
        (SubCellIndex::X0Y0Z0, SubCellIndex::X1Y0Z0),
        (SubCellIndex::X0Y1Z0, SubCellIndex::X1Y1Z0),
        (SubCellIndex::X0Y0Z1, SubCellIndex::X1Y0Z1),
        (SubCellIndex::X0Y1Z1, SubCellIndex::X1Y1Z1),
    ],
    // y axis
    [
        (SubCellIndex::X0Y0Z0, SubCellIndex::X0Y1Z0),
        (SubCellIndex::X1Y0Z0, SubCellIndex::X1Y1Z0),
        (SubCellIndex::X0Y0Z1, SubCellIndex::X0Y1Z1),
        (SubCellIndex::X1Y0Z1, SubCellIndex::X1Y1Z1),
    ],
    // z axis
    [
        (SubCellIndex::X0Y0Z0, SubCellIndex::X0Y0Z1),
        (SubCellIndex::X1Y0Z0, SubCellIndex::X1Y0Z1),
        (SubCellIndex::X0Y1Z0, SubCellIndex::X0Y1Z1),
        (SubCellIndex::X1Y1Z0, SubCellIndex::X1Y1Z1),
    ],
];

//  Cell Point and Subcell Layout
//
//      (010)o--------------o(011)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |         ^ Y
// (110)o--------------o(111)      |
//    |(000)o. . . . |. . o(001)     --> X
//    |   .          |   /        /
//    |  .           |  /        /
//    | .            | /        z
//    |.             |/
// (100)o--------------o(101)
//
// 根据边的朝向，获取负半轴和正半轴的四个SubCellIndex.
// cell index 排列顺序间 EdgeCells的注释。
pub const SUBCELL_EDGES_NEIGHBOUR_PAIRS: [[(
    SubCellIndex,
    SubCellIndex,
    SubCellIndex,
    SubCellIndex,
); 2]; AxisType::COUNT] = [
    // two group indices is x axis
    [
        (
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X0Y1Z1,
        ),
        (
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y1Z1,
        ),
    ],
    //  Cell Point and Subcell Layout
    //
    //      (010)o--------------o(011)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (110)o--------------o(111) |     |
    //    |(000)o. . . . |. . o(001)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (100)o--------------o(101)
    //
    // two group indices is y axis
    [
        (
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X1Y0Z1,
        ),
        (
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X1Y1Z1,
        ),
    ],
    //  Cell Point and Subcell Layout
    //
    //      (010)o--------------o(011)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (110)o--------------o(111) |     |
    //    |(000)o. . . . |. . o(001)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (100)o--------------o(101)
    //
    [
        (
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X0Y1Z0,
        ),
        (
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X0Y1Z1,
        ),
    ],
];

//  Face Index Layout
//
//         o--------------o
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
//    o--------------o    |     |
//    |    o. . . . . |. . o      --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
//    o--------------o
//
// order is from x to y to z
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, FromRepr, Reflect)]
pub enum FaceIndex {
    Left = 0,
    Right = 1,
    Bottom = 2,
    Top = 3,
    Back = 4,
    Front = 5,
}

//  Cell Point and Subcell Layout
//
//      (2)o--------------o(3)
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// (6)o--------------o(7) |     |
//    | (0)o. . . . . |. . o(1)  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// (4)o--------------o(5)
//
//
/// @brief Cell neighbour table
///  找到一个subcell在FaceIndex的方向是否有邻居subcell,以及相邻的Subcell的位置
///
pub const NEIGHBOUR_ADDRESS_TABLE: [[(SubCellIndex, bool); SubCellIndex::COUNT]; FaceIndex::COUNT] = [
    // left
    [
        (SubCellIndex::X1Y0Z0, false),
        (SubCellIndex::X0Y0Z0, true),
        (SubCellIndex::X1Y1Z0, false),
        (SubCellIndex::X0Y1Z0, true),
        (SubCellIndex::X1Y0Z1, false),
        (SubCellIndex::X0Y0Z1, true),
        (SubCellIndex::X1Y1Z1, false),
        (SubCellIndex::X0Y1Z1, true),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // right
    [
        (SubCellIndex::X1Y0Z0, true),
        (SubCellIndex::X0Y0Z0, false),
        (SubCellIndex::X1Y1Z0, true),
        (SubCellIndex::X0Y1Z0, false),
        (SubCellIndex::X1Y0Z1, true),
        (SubCellIndex::X0Y0Z1, false),
        (SubCellIndex::X1Y1Z1, true),
        (SubCellIndex::X0Y1Z1, false),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // Bottom
    [
        (SubCellIndex::X0Y1Z0, false),
        (SubCellIndex::X1Y1Z0, false),
        (SubCellIndex::X0Y0Z0, true),
        (SubCellIndex::X1Y0Z0, true),
        (SubCellIndex::X0Y1Z1, false),
        (SubCellIndex::X1Y1Z1, false),
        (SubCellIndex::X0Y0Z1, true),
        (SubCellIndex::X1Y0Z1, true),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // Top
    [
        (SubCellIndex::X0Y1Z0, true),
        (SubCellIndex::X1Y1Z0, true),
        (SubCellIndex::X0Y0Z0, false),
        (SubCellIndex::X1Y0Z0, false),
        (SubCellIndex::X0Y1Z1, true),
        (SubCellIndex::X1Y1Z1, true),
        (SubCellIndex::X0Y0Z1, false),
        (SubCellIndex::X1Y0Z1, false),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // Back
    [
        (SubCellIndex::X0Y0Z1, false),
        (SubCellIndex::X1Y0Z1, false),
        (SubCellIndex::X0Y1Z1, false),
        (SubCellIndex::X1Y1Z1, false),
        (SubCellIndex::X0Y0Z0, true),
        (SubCellIndex::X1Y0Z0, true),
        (SubCellIndex::X0Y1Z0, true),
        (SubCellIndex::X1Y1Z0, true),
    ],
    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    // Front
    [
        (SubCellIndex::X0Y0Z1, true),
        (SubCellIndex::X1Y0Z1, true),
        (SubCellIndex::X0Y1Z1, true),
        (SubCellIndex::X1Y1Z1, true),
        (SubCellIndex::X0Y0Z0, false),
        (SubCellIndex::X1Y0Z0, false),
        (SubCellIndex::X0Y1Z0, false),
        (SubCellIndex::X1Y1Z0, false),
    ],
];

pub const FACE_TO_SUB_EDGES_AXIS_TYPE: [[AxisType; 2]; AxisType::COUNT] = [
    // face is x axis
    [AxisType::YAxis, AxisType::ZAxis],
    // face is y axis
    [AxisType::XAxis, AxisType::ZAxis],
    // face is z axis
    [AxisType::XAxis, AxisType::YAxis],
];

//      (2)o--------------o(3)
//        /.             /|                    z ---------->
//       / .            / |                  |--------------|  /\
//      /  .           /  |                  |       |      |  |
//     /   .          /   |     ^ Y          |       1      |  |
// (6)o--------------o(7) |     |            |- -1 --x-- 1--|  |
//    | (0)o. . . . . |. . o(1)  --> X       |      -1      |  |
//    |   .          |   /     /             |       |      |  |
//    |  .           |  /     /              |--------------|  x
//    | .            | /     z
//    |.             |/
// (4)o--------------o(5)
//
// 垂直于轴的面上的四个边所相邻的SubCellIndex
// face axis => 边的轴向 =》边的轴值 => 共享边的SubCellIndex, 以及SubCellIndex属于Face左侧还是右侧的Cell。
// 需要注意，因为Face是连接两个Cell的，所以AxisValue::Zero表示在右侧的Cell，AxisValue::One表示在左侧的Cell, 与正常值相反。
#[allow(clippy::type_complexity)]
pub const FACES_TO_SUB_EDGES_CELLS: [[[[(SubCellIndex, AxisValue); 4]; 2]; 2]; AxisType::COUNT] = [
    // 面是 x axis
    [
        // 边是 y axis
        [
            [
                // 1 0 5 4
                (SubCellIndex::X1Y0Z0, AxisValue::Zero),
                (SubCellIndex::X0Y0Z0, AxisValue::One),
                (SubCellIndex::X1Y0Z1, AxisValue::Zero),
                (SubCellIndex::X0Y0Z1, AxisValue::One),
            ],
            [
                (SubCellIndex::X1Y1Z0, AxisValue::Zero),
                (SubCellIndex::X0Y1Z0, AxisValue::One),
                (SubCellIndex::X1Y1Z1, AxisValue::Zero),
                (SubCellIndex::X0Y1Z1, AxisValue::One),
            ],
        ],
        // 边是 z axis
        [
            [
                // 0 1 2 3
                (SubCellIndex::X0Y0Z0, AxisValue::One),
                (SubCellIndex::X1Y0Z0, AxisValue::Zero),
                (SubCellIndex::X0Y1Z0, AxisValue::One),
                (SubCellIndex::X1Y1Z0, AxisValue::Zero),
            ],
            [
                (SubCellIndex::X0Y0Z1, AxisValue::One),
                (SubCellIndex::X1Y0Z1, AxisValue::Zero),
                (SubCellIndex::X0Y1Z1, AxisValue::One),
                (SubCellIndex::X1Y1Z1, AxisValue::Zero),
            ],
        ],
    ],
    //      [2]o--------------o[3]
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // [6]o--------------o[7] |     |
    //    | [0]o. . . . . |. . o[1]  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // [4]o--------------o[5]
    //
    //面是  y axis
    [
        //边是  x axis
        [
            [   // 2 6 0 4
                (SubCellIndex::X0Y1Z0, AxisValue::Zero),
                (SubCellIndex::X0Y1Z1, AxisValue::Zero),
                (SubCellIndex::X0Y0Z0, AxisValue::One),
                (SubCellIndex::X0Y0Z1, AxisValue::One),
            ],
            [
                (SubCellIndex::X1Y1Z0, AxisValue::Zero),
                (SubCellIndex::X1Y1Z1, AxisValue::Zero),
                (SubCellIndex::X1Y0Z0, AxisValue::One),
                (SubCellIndex::X1Y0Z1, AxisValue::One),
            ],
        ],
        // 边是 z axis
        [
            [
                // 3 2 1 0
                (SubCellIndex::X1Y1Z0, AxisValue::Zero),
                (SubCellIndex::X0Y1Z0, AxisValue::Zero),
                (SubCellIndex::X1Y0Z0, AxisValue::One),
                (SubCellIndex::X0Y0Z0, AxisValue::One),
            ],
            [
                (SubCellIndex::X1Y1Z1, AxisValue::Zero),
                (SubCellIndex::X0Y1Z1, AxisValue::Zero),
                (SubCellIndex::X1Y0Z1, AxisValue::One),
                (SubCellIndex::X0Y0Z1, AxisValue::One),
            ],
        ],
    ],
    //      [2]o--------------o[3]
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // [6]o--------------o[7] |     |
    //    | [0]o. . . . . |. . o[1]  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // [4]o--------------o[5]
    //
    //面是 z axis
    [
        // 边是 x axis
        [
            [
                // 4 0 6 2
                (SubCellIndex::X0Y0Z1, AxisValue::Zero),
                (SubCellIndex::X0Y0Z0, AxisValue::One),
                (SubCellIndex::X0Y1Z1, AxisValue::Zero),
                (SubCellIndex::X0Y1Z0, AxisValue::One),
            ],
            [
                (SubCellIndex::X1Y0Z1, AxisValue::Zero),
                (SubCellIndex::X1Y0Z0, AxisValue::One),
                (SubCellIndex::X1Y1Z1, AxisValue::Zero),
                (SubCellIndex::X1Y1Z0, AxisValue::One),
            ],
        ],
        // 边是 y axis
        [
            [
                //  4 5 0 1
                (SubCellIndex::X0Y0Z1, AxisValue::Zero),
                (SubCellIndex::X1Y0Z1, AxisValue::Zero),
                (SubCellIndex::X0Y0Z0, AxisValue::One),
                (SubCellIndex::X1Y0Z0, AxisValue::One),
            ],
            [
                (SubCellIndex::X0Y1Z1, AxisValue::Zero),
                (SubCellIndex::X1Y1Z1, AxisValue::Zero),
                (SubCellIndex::X0Y1Z0, AxisValue::One),
                (SubCellIndex::X1Y1Z0, AxisValue::One),
            ],
        ],
    ],
];

//      [2]o--------------o[3]
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// [6]o--------------o[7] |     |
//    | [0]o. . . . . |. . o[1]  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// [4]o--------------o[5]
//
// four edges axis in a face axis
pub const FACE_TO_EDGE_AXIS: [[AxisType; 4]; AxisType::COUNT] = [
    // x axis
    [
        AxisType::YAxis,
        AxisType::YAxis,
        AxisType::ZAxis,
        AxisType::ZAxis,
    ],
    // y axis
    [
        AxisType::XAxis,
        AxisType::XAxis,
        AxisType::ZAxis,
        AxisType::ZAxis,
    ],
    // z axis
    [
        AxisType::XAxis,
        AxisType::XAxis,
        AxisType::YAxis,
        AxisType::YAxis,
    ],
];

//      [2]o--------------o[3]
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// [6]o--------------o[7] |     |
//    | [0]o. . . . . |. . o[1]  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// [4]o--------------o[5]
//
// 不同方向的边，切割为两个子边，每个子边的4个SubCellIndex,
// 因为边是跨越4个Cell的，所以SubCellIndex是4个不同的Cell的。
pub const SUBEDGE_CELLS: [[[SubCellIndex; 4]; 2]; AxisType::COUNT] = [
    // edge is x axis
    [
        // 负半轴
        [   // 6 2 4 0
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X0Y0Z0,
        ],
        // 正半轴
        [
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X1Y0Z0,
        ],
    ],
    // edge is y axis
    [
        // 5 4 1 0
        // 负半轴
        [
            SubCellIndex::X1Y0Z1,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X1Y0Z0,
            SubCellIndex::X0Y0Z0,
        ],
        // 正半轴
        [
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X0Y1Z0,
        ],
    ],
    // edge is z axis
    [
        // 2 3 0 1
        // 负半轴
        [
            SubCellIndex::X0Y1Z0,
            SubCellIndex::X1Y1Z0,
            SubCellIndex::X0Y0Z0,
            SubCellIndex::X1Y0Z0,
        ],
        // 正半轴
        [
            SubCellIndex::X0Y1Z1,
            SubCellIndex::X1Y1Z1,
            SubCellIndex::X0Y0Z1,
            SubCellIndex::X1Y0Z1,
        ],
    ],
];

//      [2]o--------------o[3]
//        /.             /|
//       / .            / |
//      /  .           /  |
//     /   .          /   |     ^ Y
// [6]o--------------o[7] |     |
//    | [0]o. . . . . |. . o[1]  --> X
//    |   .          |   /     /
//    |  .           |  /     /
//    | .            | /     z
//    |.             |/
// [4]o--------------o[5]
//
// 因为边是跨越4个Cell的，所以SubCellIndex是4个不同的Cell的。
pub const EDGE_CELLS_VERTICES: [[[SubCellIndex; 2]; 4]; AxisType::COUNT] = [
    // edge is x axis
    [
        // 负方向到正方向
        // 6 7
        [SubCellIndex::X0Y1Z1, SubCellIndex::X1Y1Z1],
        // 2 3
        [SubCellIndex::X0Y1Z0, SubCellIndex::X1Y1Z0],
        // 4 5
        [SubCellIndex::X0Y0Z1, SubCellIndex::X1Y0Z1],
        // 0 1
        [SubCellIndex::X0Y0Z0, SubCellIndex::X1Y0Z0],
    ],
    // edge is y axis
    [
        // 5 7
        [SubCellIndex::X1Y0Z1, SubCellIndex::X1Y1Z1],
        // 4 6
        [SubCellIndex::X0Y0Z1, SubCellIndex::X0Y1Z1],
        // 1 3
        [SubCellIndex::X1Y0Z0, SubCellIndex::X1Y1Z0],
        // 0 2
        [SubCellIndex::X0Y0Z0, SubCellIndex::X0Y1Z0],
    ],
    // edge is z axis
    [
        // 2 6
        [SubCellIndex::X0Y1Z0, SubCellIndex::X0Y1Z1],
        // 3 7
        [SubCellIndex::X1Y1Z0, SubCellIndex::X1Y1Z1],
        // 0 4
        [SubCellIndex::X0Y0Z0, SubCellIndex::X0Y0Z1],
        // 1 5
        [SubCellIndex::X1Y0Z0, SubCellIndex::X1Y0Z1],
    ],
];
