/// A not at all necessary macro for fun, brought to you by tears.

#[macro_export]
macro_rules! raw_packet {
    // process outputs
    (@internal -> ()) => {
        Vec::<u8>::new()
    };

    (@internal -> (,$($items:tt),+)) => {
        vec![$($items),+]
    };

    // process inputs
    (@internal (u32_le $num:literal)$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        raw_packet!(@internal $($($tail),*)? -> (
            $(,$($items),*)?,
            (($num & 0xff) as u8),
            (($num >> 8 & 0xff) as u8),
            (($num >> 16 & 0xff) as u8),
            (($num >> 24 & 0xff) as u8)
        ))
    };

    (@internal (u32_be $num:literal)$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        raw_packet!(@internal $($($tail),*)? -> (
            $(,$($items),*)?,
            (($num >> 24 & 0xff) as u8),
            (($num >> 16 & 0xff) as u8),
            (($num >> 8 & 0xff) as u8),
            (($num & 0xff) as u8)
        ))
    };

    (@internal (u32_ne $num:literal)$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        if cfg!(target_endian="big") {
            raw_packet!(@internal (u32_be $num)$(, $($tail),*)? -> ($(,$($items),*)?))
        } else {
            raw_packet!(@internal (u32_le $num)$(, $($tail),*)? -> ($(,$($items),*)?))
        }
    };

    (@internal [be2le_4 $b1:literal, $b2:literal, $b3:literal, $b4:literal]$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        raw_packet!(@internal $($($tail),*)? -> (
            $(,$($items),*)?,
            $b4,
            $b3,
            $b2,
            $b1
        ))
    };

    (@internal [be2ne_4 $b1:literal, $b2:literal, $b3:literal, $b4:literal]$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        if cfg!(target_endian="big") {
            raw_packet!(@internal $($($tail),*)? -> (
                $(,$($items),*)?,
                $b4,
                $b3,
                $b2,
                $b1
            ))
        } else {
            raw_packet!(@internal $($($tail),*)? -> (
                $(,$($items),*)?,
                $b1,
                $b2,
                $b3,
                $b4
            ))
        }
    };

    (@internal magic$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        raw_packet!(@internal $($($tail),*)? -> ($(,$($items),*)?, b'm', b'a', b'g', b'i', b'c'))
    };

    (@internal $head:tt$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        raw_packet!(@internal $($($tail),*)? -> ($(,$($items),*)?, $head))
    };

    // API
    ($($val:tt),*) => {
        raw_packet!(@internal $($val),* -> ())
    };
}

#[macro_export]
macro_rules! raw_packet_wbody {
    (header: [$($val:tt),*], body: $body:tt) => {
        {
            let mut vec: Vec<u8> = raw_packet![$($val),*];
            vec.extend_from_slice($body.as_bytes());
            vec
        }
    };
}
