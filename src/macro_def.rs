#[allow(unused_imports)]
pub(crate) use super::num_mask;

/// 根据寄存器位域的宽度和偏移，返回 umask 和 需要设置的值
/// ```rust
/// let (umask, val) = macro_def::field_umask_val_make(mode, offset, width)
/// ```
#[macro_export]
macro_rules! field_umask_val_make {
    ($va: ident, $oft:expr, $widt: expr) => {{
        let va = &$va;
        let mask = ((0x01 << $widt) - 1) << $oft;
        let val = ((*va as u32) << $oft) & mask;
        (!mask, val)
    }};
}

/// 生成一个枚举
#[macro_export]
macro_rules! enum_impl_from_make {
    (
        $(#[$meta: meta])*
        $vis: vis enum $name: ident <$ty: ty>{
            $(
                $(#[$item_meta: meta])*
                $item: ident -> $val: expr,
            )*}
    ) => {
        $(#[$meta])*
        $vis enum $name{
            $(
                $(#[$item_meta])*
                $item = $val
            ),*
        }
    };
}

/// 生成一个 $ty 类型的mask
/// ```rust
/// let x = num_mask!(u32; 1, 2 3);  // x = 0b1110
/// let y = num_mask!(u32;)          // y = 0b0000
#[macro_export]
#[allow(unused_imports)]
macro_rules! num_mask {
    (
        $ty: ty;
        $(
            $num: expr
        ),*
    ) => {{
        let mut x: $ty = 0x00;
        $(
            x = x | (0x01 << $num);
        )*
        x
    }};
}
