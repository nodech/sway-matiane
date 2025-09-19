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
    (@internal (u32_le $num:expr)$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        raw_packet!(@internal $($($tail),*)? -> (
            $(,$($items),*)?,
            (($num & 0xff) as u8),
            (($num >> 8 & 0xff) as u8),
            (($num >> 16 & 0xff) as u8),
            (($num >> 24 & 0xff) as u8)
        ))
    };

    (@internal (u32_be $num:expr)$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
        raw_packet!(@internal $($($tail),*)? -> (
            $(,$($items),*)?,
            (($num >> 24 & 0xff) as u8),
            (($num >> 16 & 0xff) as u8),
            (($num >> 8 & 0xff) as u8),
            (($num & 0xff) as u8)
        ))
    };

    (@internal (u32_ne $num:expr)$(, $($tail:tt),*)? -> ($(,$($items:tt),*)?)) => {
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
        if cfg!(target_endian="little") {
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
        raw_packet!(@internal $($($tail),*)? -> ($(,$($items),*)?, b'i', b'3', b'-', b'i', b'p', b'c'))
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
macro_rules! raw_packet_with_body {
    (header: [$($val:tt),*], body: $body:expr) => {
        {
            let mut vec: Vec<u8> = raw_packet![$($val),*];
            vec.extend_from_slice($body);
            vec
        }
    };
}

#[macro_export]
macro_rules! raw_subscribe_success {
    () => {{
        let payload: &[u8] = br#"{"success":true}"#;
        raw_packet_with_body! {
            header: [magic, (u32_ne payload.len()), (u32_ne 2)],
            body: payload
        }
    }};
}

#[macro_export]
macro_rules! assert_sway_error {
    ($result:ident, $type:ty, $pattern:pat) => {{
        assert!($result.is_err());
        let error = $result.unwrap_err();
        let spec = error.downcast_ref::<$type>();
        assert!(matches!(spec, Some($pattern)));
    }};
}

#[macro_export]
macro_rules! generate_sway_bad_subscribe_tests {
    ($([$name:ident, $subscribe_response:expr, $error_type:ty, $error_pat:pat]),*$(,)?) => {
        $(
            #[tokio::test]
            async fn $name() -> Result<()> {
                let server_recv = raw_packet_with_body!{
                    header: [magic, (u32_ne 10), (u32_ne 2)],
                    body: br#"["window"]"#
                };

                let MockServer {
                    dir: _dir,
                    bind_path,
                    handle,
                } = setup_mock_server(stringify!($name), server_recv, $subscribe_response)?;

                let subbed = subscribe(&bind_path, EventType::Window).await;
                assert_sway_error!(subbed, $error_type, $error_pat);

                handle.await??;

                Ok(())
            }
        )*
    };
}

#[macro_export]
macro_rules! generate_sway_bad_event_tests {
    ($([$name:ident, $event_packet:expr, $error_type:ty, $error_apt:pat]),*$(,)?) => {
        $(
            #[tokio::test]
            async fn $name() -> Result<()> {
                let server_recv = raw_packet_with_body!{
                    header: [magic, (u32_ne 10), (u32_ne 2)],
                    body: br#"["window"]"#
                };

                let sub_success = raw_subscribe_success!();
                let response = [sub_success, $event_packet].concat();

                let MockServer {
                    dir: _dir,
                    bind_path,
                    handle,
                } = setup_mock_server(stringify!($name), server_recv, response)?;

                let mut events = subscribe(&bind_path, EventType::Window).await?;

                let event = events.next().await.expect("Must return something.");
                assert_sway_error!(event, $error_type, $error_apt);

                handle.await??;

                Ok(())
            }
        )*
    };
}
