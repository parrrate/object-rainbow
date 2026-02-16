#[macro_export]
#[doc(hidden)]
macro_rules! unpack_where {
    ({$($a:tt)*} {$($tr:tt)*} {$ty:ty} {$($w:tt)*} {}) => {
        const _: () = {
            fn _inner<$($a)* X: $($tr)*>() $($w)* {}
            fn _outer<$($a)*>() $($w)* {
                _inner::<$($a)* $ty>()
            }
        };
    };
    ({$($a:tt)*} {$($tr:tt)*} {$ty:ty} {$($w:tt)*} $ww:tt $($www:tt)*) => {
        $crate::unpack_where!{ {$($a)*} {$($tr)*} {$ty} {$($w)* $ww} $($www)* }
    };
}

#[macro_export]
macro_rules! assert_impl {
    (impl $(<$($a:ident),*>)? $tr:ident$(<$($tra:ident),*>)? for $ty:ty where $($w:tt)*) => {
        $crate::unpack_where! { {$($($a,)*)?} {$tr$(<$($tra),*>)?} {$ty} {} where $($w)* }
    };
    (impl $(<$($a:ident),*>)? $tr:ident$(<$($tra:ident),*>)? for $ty:ty {}) => {
        $crate::unpack_where! { {$($($a,)*)?} {$tr$(<$($tra),*>)?} {$ty} {} {} }
    };
}

assert_impl!(
    impl<T> Clone for T where T: Copy {}
);

assert_impl!(
    impl Copy for i64 {}
);
