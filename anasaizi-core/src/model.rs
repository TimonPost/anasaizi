mod mesh;
mod object;

use crate::math::PosOnlyVertex;
use nalgebra::Vector3;

pub use object::Object;

pub fn square_vertices() -> [PosOnlyVertex; 6] {
    return [
        PosOnlyVertex {
            pos: Vector3::new(1.0, 1.0, 0.0),
        },
        PosOnlyVertex {
            pos: Vector3::new(-1.0, -1.0, 0.0),
        },
        PosOnlyVertex {
            pos: Vector3::new(-1.0, 1.0, 0.0),
        },
        PosOnlyVertex {
            pos: Vector3::new(-1.0, -1.0, 0.0),
        },
        PosOnlyVertex {
            pos: Vector3::new(1.0, 1.0, 0.0),
        },
        PosOnlyVertex {
            pos: Vector3::new(1.0, -1.0, 0.0),
        },
    ];
}

pub fn square_indices() -> [u32; 6] {
    return [0, 1, 2, 3, 4, 5];
}
