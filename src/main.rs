use nom::{
    bytes::complete::{tag, take_until},
    sequence::delimited,
    IResult,
};
use std::io::BufRead;
use std::process::Stdio;
use std::{env, process::Command};

// Crate to calculate timecode
use vtc::{rates, Timecode};

fn main() {
    let args: Vec<String> = env::args().collect();
    args.get(1).unwrap_or_else(|| panic!("missing argument"));

    run_ffmpeg_cmd(&args)
}

fn run_ffmpeg_cmd(args: &Vec<String>) {
    let file_input_arg = &args[1];

    let mut ffmpeg_cmd = Command::new("ffmpeg")
        .args(["-hide_banner", "-i"])
        .arg(file_input_arg)
        .args(["-vf", "blackdetect,blackframe", "-f", "null", "-"])
        .stderr(Stdio::piped())
        .spawn()
        .expect("error running ffprobe command");

    let stderr = ffmpeg_cmd.stderr.take().unwrap();
    let stderr_reader = std::io::BufReader::new(stderr);

    let mut prev_line = String::new();
    for line in stderr_reader.lines() {
        let buf_line = line.expect("Failed to read line from stdout");
        if buf_line.contains("blackdetect") {
            ffmpeg_cmd.kill().expect("failed to kill ffmpeg process");

            match extract_filter_prefix(&mut prev_line) {
                Ok((f, _)) => match get_filter_value(f, "frame") {
                    Some(v) => {
                        let timecode = get_timecode(v as u32);
                        println!("{}", timecode)
                    }
                    None => panic!("nothing found"),
                },
                Err(_) => panic!("Error removing prefix"),
            }

            get_timecode(210);
        }
        prev_line = buf_line;
    }
}

fn get_filter_value(raw_str: &str, value: &str) -> Option<f32> {
    for part in raw_str.split_whitespace() {
        let mut split_iter = part.split(':');
        let key = split_iter.next()?;
        let value_str = split_iter.next()?;

        if key == value {
            return value_str.parse().ok();
        }
    }

    return None;
}

fn extract_filter_prefix(input: &mut String) -> IResult<&str, ()> {
    let (parsed_value, _) = delimited(tag("["), take_until("] "), tag("] "))(input.as_str())?;
    return Ok((parsed_value, ()));
}

fn get_timecode(frame: u32) -> String {
    return Timecode::with_frames(frame, rates::F29_97_DF)
        .unwrap()
        .timecode();
}

#[cfg(test)]
mod tests {
    use crate::{extract_filter_prefix, get_filter_value, get_timecode};

    #[test]
    fn get_framerate_vtc_lib_frame_209() {
        let timecode = get_timecode(209);
        assert_eq!(timecode, "00:00:06;29");
    }

    #[test]
    fn get_framerate_vtc_lib_frame_210() {
        let timecode = get_timecode(210);
        assert_eq!(timecode, "00:00:07;00");
    }

    #[test]
    fn get_blackframe_filter_values() {
        let mut test_str = String::from("[Parsed_blackframe_0 @ 0x10e60fd50] frame:720 pblack:100 pts:720 t:24.024000 type:I last_keyframe:720");
        let filter_value = extract_filter_prefix(&mut test_str);
        assert_eq!(
            filter_value,
            Ok((
                "frame:720 pblack:100 pts:720 t:24.024000 type:I last_keyframe:720",
                ()
            ))
        )
    }

    #[test]
    fn get_blackdetect_filter_values() {
        let mut test_str = String::from("[blackdetect @ 0x13e00d040] black_start:4.97163 black_end:7.007 black_duration:2.03537");
        let filter_value = extract_filter_prefix(&mut test_str);
        assert_eq!(
            filter_value,
            Ok((
                "black_start:4.97163 black_end:7.007 black_duration:2.03537",
                ()
            ))
        )
    }

    #[test]
    fn get_filter_value_black_end() {
        let value = get_filter_value(
            "black_start:4.97163 black_end:7.007 black_duration:2.03537",
            "black_end",
        );
        assert_eq!(value, Some(7.007));
    }

    #[test]
    fn get_filter_value_frame() {
        let value = get_filter_value(
            "frame:720 pblack:100 pts:720 t:24.024000 type:I last_keyframe:720",
            "frame",
        );
        assert_eq!(value, Some(720.0));
    }

    #[test]
    fn get_filter_value_none() {
        let value = get_filter_value(
            "frame:720 pblack:100 pts:720 t:24.024000 type:I last_keyframe:720",
            "nothing",
        );
        assert_eq!(value, None);
    }
}
