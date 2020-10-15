use storage_proofs_core::merkle::LCTree;
use typenum::{U0, U2, U8};

use crate::{
    constants::{
        SECTOR_SIZE_16_KIB, SECTOR_SIZE_16_MIB, SECTOR_SIZE_1_GIB, SECTOR_SIZE_2_KIB,
        SECTOR_SIZE_32_GIB, SECTOR_SIZE_32_KIB, SECTOR_SIZE_4_KIB, SECTOR_SIZE_512_MIB,
        SECTOR_SIZE_64_GIB, SECTOR_SIZE_8_MIB,
    },
    types::DefaultTreeHasher,
};

// Generic shapes
pub type SectorShapeBase = LCTree<DefaultTreeHasher, U8, U0, U0>;
pub type SectorShapeSub2 = LCTree<DefaultTreeHasher, U8, U2, U0>;
pub type SectorShapeSub8 = LCTree<DefaultTreeHasher, U8, U8, U0>;
pub type SectorShapeTop2 = LCTree<DefaultTreeHasher, U8, U8, U2>;

// Specific size constants by shape
pub type SectorShape2KiB = SectorShapeBase;
pub type SectorShape8MiB = SectorShapeBase;
pub type SectorShape512MiB = SectorShapeBase;

pub type SectorShape4KiB = SectorShapeSub2;
pub type SectorShape16MiB = SectorShapeSub2;
pub type SectorShape1GiB = SectorShapeSub2;

pub type SectorShape16KiB = SectorShapeSub8;
pub type SectorShape32GiB = SectorShapeSub8;

pub type SectorShape32KiB = SectorShapeTop2;
pub type SectorShape64GiB = SectorShapeTop2;

pub fn is_sector_shape_base(sector_size: u64) -> bool {
    match sector_size {
        SECTOR_SIZE_2_KIB | SECTOR_SIZE_8_MIB | SECTOR_SIZE_512_MIB => true,
        _ => false,
    }
}

pub fn is_sector_shape_sub2(sector_size: u64) -> bool {
    match sector_size {
        SECTOR_SIZE_4_KIB | SECTOR_SIZE_16_MIB | SECTOR_SIZE_1_GIB => true,
        _ => false,
    }
}

pub fn is_sector_shape_sub8(sector_size: u64) -> bool {
    match sector_size {
        SECTOR_SIZE_16_KIB | SECTOR_SIZE_32_GIB => true,
        _ => false,
    }
}

pub fn is_sector_shape_top2(sector_size: u64) -> bool {
    match sector_size {
        SECTOR_SIZE_32_KIB | SECTOR_SIZE_64_GIB => true,
        _ => false,
    }
}

/// Calls a function with the type hint of the sector shape matching the provided sector.
/// Panics if provided with an unknown sector size.
#[macro_export]
macro_rules! with_shape_enum {
    ($size:expr, $f:ident) => {
        with_shape_enum!($size, $f,)
    };
    ($size:expr, $f:ident, $($args:expr,)*) => {
        #[allow(unreachable_patterns)]
        match $size {
            $crate::types::SectorSize::KiB2 => {
              $f::<$crate::types::SectorShape2KiB>($($args),*)
            },
            $crate::types::SectorSize::KiB4 => {
              $f::<$crate::types::SectorShape4KiB>($($args),*)
            },
            $crate::types::SectorSize::KiB16 => {
              $f::<$crate::types::SectorShape16KiB>($($args),*)
            },
            $crate::types::SectorSize::KiB32 => {
              $f::<$crate::types::SectorShape32KiB>($($args),*)
            },
            $crate::types::SectorSize::MiB8 => {
              $f::<$crate::types::SectorShape8MiB>($($args),*)
            },
            $crate::types::SectorSize::MiB16 => {
              $f::<$crate::types::SectorShape16MiB>($($args),*)
            },
            $crate::types::SectorSize::MiB512 => {
              $f::<$crate::types::SectorShape512MiB>($($args),*)
            },
            $crate::types::SectorSize::GiB1=> {
              $f::<$crate::types::SectorShape1GiB>($($args),*)
            },
            $crate::types::SectorSize::GiB32 => {
              $f::<$crate::types::SectorShape32GiB>($($args),*)
            },
            $crate::types::SectorSize::GiB64 => {
              $f::<$crate::types::SectorShape64GiB>($($args),*)
            },
            _ => panic!("unsupported sector size: {:?}", $size),
        }
    };
    ($size:expr, $f:ident, $($args:expr),*) => {
        with_shape_enum!($size, $f, $($args,)*)
    };
}

/// Calls a function with the type hint of the sector shape matching the provided sector.
/// Panics if provided with an unknown sector size.
#[macro_export]
macro_rules! with_shape {
    ($size:expr, $f:ident) => {
        with_shape!($size, $f,)
    };
    ($size:expr, $f:ident, $($args:expr,)*) => {{
        use std::convert::TryInto;
        let e: $crate::types::SectorSize = $size.try_into().expect("unsupported sector size");

        with_shape_enum!(e, $f, $($args), *)
    }};
    ($size:expr, $f:ident, $($args:expr),*) => {
        with_shape!($size, $f, $($args,)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    use generic_array::typenum::Unsigned;
    use storage_proofs_core::merkle::MerkleTreeTrait;

    fn canonical_shape(sector_size: u64) -> (usize, usize, usize) {
        // This could perhaps be cleaned up, but I think it expresses the intended constraints
        // and is consistent with our current hard-coded size->shape mappings.
        assert_eq!(sector_size.count_ones(), 1);
        let log_byte_size = sector_size.trailing_zeros();
        let log_nodes = log_byte_size - 5; // 2^5 = 32-byte nodes

        let max_tree_log = 3; // Largest allowable arity. The optimal shape.

        let log_max_base = 27; // 4 GiB worth of nodes
        let log_base = max_tree_log; // Base must be oct trees.x
        let log_in_base = u32::min(log_max_base, (log_nodes / log_base) * log_base); // How many nodes in base?

        let log_upper = log_nodes - log_in_base; // Nodes in sub and upper combined.
        let log_rem = log_upper % max_tree_log; // Remainder after filling optimal trees.

        let (log_sub, log_top) = {
            // Are the upper trees empty?
            if log_upper > 0 {
                // Do we need a remainder tree?
                if log_rem == 0 {
                    (Some(max_tree_log), None) // No remainder tree, fill the sub tree optimall.y
                } else {
                    // Need a remainder tree.

                    // Do we have room for another max tree?
                    if log_upper > max_tree_log {
                        // There is room. Use the sub tree for as much overflow as we can fit optimally.
                        // And put the rest in the top tree.
                        (Some(max_tree_log), Some(log_rem))
                    } else {
                        // Can't fit another max tree.
                        // Just put the remainder in the sub tree.
                        (Some(log_rem), None)
                    }
                }
            } else {
                // Upper trees are empty.
                (None, None)
            }
        };

        let base = 1 << log_base;
        let sub = if let Some(l) = log_sub { 1 << l } else { 0 };
        let top = if let Some(l) = log_top { 1 << l } else { 0 };

        (base, sub, top)
    }

    fn arities_to_usize<Tree: MerkleTreeTrait>() -> (usize, usize, usize) {
        (
            Tree::Arity::to_usize(),
            Tree::SubTreeArity::to_usize(),
            Tree::TopTreeArity::to_usize(),
        )
    }

    #[test]
    fn test_with_shape_macro() {
        test_with_shape_macro_aux(SECTOR_SIZE_2_KIB);
        test_with_shape_macro_aux(SECTOR_SIZE_4_KIB);
        test_with_shape_macro_aux(SECTOR_SIZE_8_MIB);
        test_with_shape_macro_aux(SECTOR_SIZE_16_MIB);
        test_with_shape_macro_aux(SECTOR_SIZE_512_MIB);
        test_with_shape_macro_aux(SECTOR_SIZE_1_GIB);
        test_with_shape_macro_aux(SECTOR_SIZE_32_GIB);
        test_with_shape_macro_aux(SECTOR_SIZE_64_GIB);
    }

    fn test_with_shape_macro_aux(sector_size: u64) {
        let expected = canonical_shape(sector_size);
        let arities = with_shape!(sector_size, arities_to_usize);
        assert_eq!(
            arities, expected,
            "Wrong shape for sector size {}: have {:?} but need {:?}.",
            sector_size, arities, expected
        );
    }
}
