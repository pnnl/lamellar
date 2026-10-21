use crate::array::{LamellarByteArray, ScalarType};

/// Runtime tag for the 7 builtin reduction operations.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum BuiltinOp {
    Sum,
    Prod,
    Max,
    Min,
    And,
    Or,
    Xor,
}

/// Selects the closure implementing `$op` for scalar type `$ty`. The op set depends
/// on the type class: ints get arithmetic + bitwise, floats arithmetic only,
/// bool bitwise only.
macro_rules! builtin_op_closure {
    (@arith $op:expr, $ty:ty, $msg:literal, [$($extra:tt)*]) => {{
        let closure: fn($ty, $ty) -> $ty = match $op {
            BuiltinOp::Sum  => |a, b| a + b,
            BuiltinOp::Prod => |a, b| a * b,
            BuiltinOp::Max  => |a, b| if a > b { a } else { b },
            BuiltinOp::Min  => |a, b| if a < b { a } else { b },
            $($extra)*
            _ => panic!($msg),
        };
        closure
    }};
    (ints $op:expr, $ty:ty, $msg:literal) => {
        builtin_op_closure!(@arith $op, $ty, $msg, [
            BuiltinOp::And  => |a, b| a & b,
            BuiltinOp::Or   => |a, b| a | b,
            BuiltinOp::Xor  => |a, b| a ^ b,
        ])
    };
    (floats $op:expr, $ty:ty, $msg:literal) => {
        builtin_op_closure!(@arith $op, $ty, $msg, [])
    };
    (bools $op:expr, $ty:ty, $msg:literal) => {{
        let closure: fn($ty, $ty) -> $ty = match $op {
            BuiltinOp::And  => |a, b| a & b,
            BuiltinOp::Or   => |a, b| a | b,
            BuiltinOp::Xor  => |a, b| a ^ b,
            _ => panic!($msg),
        };
        closure
    }};
}

/// Generates `reduce_scalar_local` and `merge_scalar`. Types are grouped by class
/// so each class gets its own set of supported ops; all arms land in a single
/// `match self.scalar_type`.
macro_rules! impl_reduce_scalar_type_match {
    (
        ints: [$($int:ident),* $(,)?],
        floats: [$($float:ident),* $(,)?],
        bools: [$($bool:ident),* $(,)?] $(,)?
    ) => {
        #[allow(unreachable_patterns)] // `_` arm is dead for int types (all 7 ops handled)
        async fn reduce_scalar_local(&self) -> Vec<u8> {
            match self.scalar_type {
                $( ScalarType::$int => impl_reduce_scalar_type_match!(@reduce self, ints, $int), )*
                $( ScalarType::$float => impl_reduce_scalar_type_match!(@reduce self, floats, $float), )*
                $( ScalarType::$bool => impl_reduce_scalar_type_match!(@reduce self, bools, $bool), )*
            }
        }

        #[allow(unreachable_patterns)]
        async fn merge_scalar(&self, left: Vec<u8>, right: Vec<u8>) -> Vec<u8> {
            if left.is_empty() && right.is_empty() {
                Vec::new()
            } else if left.is_empty() {
                right
            } else if right.is_empty() {
                left
            } else {
                match self.scalar_type {
                    $( ScalarType::$int => impl_reduce_scalar_type_match!(@merge self, left, right, ints, $int), )*
                    $( ScalarType::$float => impl_reduce_scalar_type_match!(@merge self, left, right, floats, $float), )*
                    $( ScalarType::$bool => impl_reduce_scalar_type_match!(@merge self, left, right, bools, $bool), )*
                }
            }
        }
    };
    (@reduce $this:ident, $class:ident, $ty:ident) => {{
        let local = $this.data.local_data::<$ty>().await;
        let closure = builtin_op_closure!($class $this.op, $ty, "reduce operation not supported");
        match local.reduce(closure) {
            Some(v) => crate::serialize(&v, false).expect("reduce_scalar_local ser res"),
            None => Vec::new(),
        }
    }};
    (@merge $this:ident, $left:ident, $right:ident, $class:ident, $ty:ident) => {{
        let left = crate::deserialize::<$ty>(&$left, false).expect("merge_scalar des left");
        let right = crate::deserialize::<$ty>(&$right, false).expect("merge_scalar des right");
        let closure = builtin_op_closure!($class $this.op, $ty, "merge operation not supported");
        let res = closure(left, right);
        crate::serialize(&res, false).expect("merge_scalar ser res")
    }};
}

/// Single AM struct used for ALL builtin reductions on ALL primitive types.
/// The array type is erased into `LamellarByteArray`; `scalar_type` and `op` tag
/// which concrete type and operation to dispatch at runtime.
#[doc(hidden)]
#[lamellar_impl::AmDataRT(Clone)]
pub(crate) struct ScalarBuiltinReductionAm {
    pub data: LamellarByteArray,
    pub start_pe: usize,
    pub end_pe: usize,
    pub scalar_type: ScalarType,
    pub op: BuiltinOp,
}

impl ScalarBuiltinReductionAm {
    impl_reduce_scalar_type_match! {
        ints: [u8, u16, u32, u64, usize, u128, i8, i16, i32, i64, isize, i128],
        floats: [f32, f64],
        bools: [bool],
    }
}

#[lamellar_impl::rt_am]
impl LamellarAM for ScalarBuiltinReductionAm {
    async fn exec(&self) -> Vec<u8> {
        if self.start_pe == self.end_pe {
            self.reduce_scalar_local().await
        } else {
            let mid_pe = (self.start_pe + self.end_pe) / 2;
            let left = __lamellar_team.spawn_am_pe(
                self.start_pe,
                ScalarBuiltinReductionAm {
                    data: self.data.clone(),
                    start_pe: self.start_pe,
                    end_pe: mid_pe,
                    scalar_type: self.scalar_type,
                    op: self.op,
                },
            );
            let right = __lamellar_team.spawn_am_pe(
                mid_pe + 1,
                ScalarBuiltinReductionAm {
                    data: self.data.clone(),
                    start_pe: mid_pe + 1,
                    end_pe: self.end_pe,
                    scalar_type: self.scalar_type,
                    op: self.op,
                },
            );
            let left_bytes = left.await;
            let right_bytes = right.await;
            self.merge_scalar(left_bytes, right_bytes).await
        }
    }
}

impl ScalarBuiltinReductionAm {
    pub(crate) fn new(data: LamellarByteArray, scalar_type: ScalarType, op: BuiltinOp) -> Self {
        let num_pes = data.team().num_pes() - 1;
        Self {
            data,
            start_pe: 0,
            end_pe: num_pes,
            scalar_type,
            op,
        }
    }
}
