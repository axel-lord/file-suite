macro_rules! conv_or_reply {
    ($reply:ident, $expr:expr, $ty:ty) => {
        match <$ty>::try_from($expr) {
            Ok(val) => val,
            Err(err) => {
                ::log::error!(
                    "could not convert {} {} to type {}\n{err}",
                    stringify!($expr),
                    $expr,
                    stringify!($ty)
                );
                return $reply.error(::libc::EIO);
            }
        }
    };
}

pub(crate) use conv_or_reply;
