#[macro_export]
macro_rules! ffmpeg {
    ($($arg:expr),*) => {{
        use std::process::Command;

        let mut cmd = Command::new("ffmpeg");

        $(
            cmd.arg($arg);
        )*

        cmd
    }};
}
