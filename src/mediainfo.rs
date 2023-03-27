#[macro_export]
macro_rules! mediainfo {
    ($($arg:expr),*) => {{
        use std::process::Command;

        let mut cmd = Command::new("mediainfo");

        $(
            cmd.arg($arg);
        )*

        cmd
    }};
}
