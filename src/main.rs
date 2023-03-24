mod tests;

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

// The content needs to be the last black frame before the material ~ the last material frame;
// the frame found by black_end is a content frame and also must be subtract by 1 because the SOM must start with a blackframe
// the frame found by black_start to get EOM must be subtract by 1 because black_start is a black_frame and it does not belong to the end of material

// Start of Material and End of Material

fn main() {
    let args: Vec<String> = env::args().collect();
    args.get(1)
        .unwrap_or_else(|| panic!("missing video path argument"));

    run_ffmpeg_cmd(&args)
}

fn run_ffmpeg_cmd(args: &Vec<String>) {
    let file_input_arg = &args[1];

    let mut ffmpeg_cmd = Command::new("ffmpeg")
        .args(["-hide_banner", "-i"])
        .arg(file_input_arg)
        .args([
            "-an",
            "-vf",
            "blackdetect=d=1,blackframe",
            "-f",
            "null",
            "-",
        ])
        .stderr(Stdio::piped())
        .spawn()
        .expect("error running ffmpeg command, maybe bad path for a video");

    let stderr = ffmpeg_cmd
        .stderr
        .take()
        .expect("error getting the stderr output");
    let stderr_reader = std::io::BufReader::new(stderr);

    let mut blackdetect_list: Vec<String> = vec![];
    let mut raw_video_duration: Option<String> = None;
    let mut found_duration = false;
    for line in stderr_reader.lines() {
        let buf_line = line.expect("Failed to read line from stdout");
        if buf_line.contains("Duration") {
            raw_video_duration = Some(buf_line);
            found_duration = true;
        } else if found_duration && buf_line.contains("black_start") {
            blackdetect_list.push(buf_line);
        }
    }

    match raw_video_duration {
        Some(d) => {
            get_som_eom(&mut blackdetect_list, d);
        }
        None => {
            panic!("Video duration not found")
        }
    }
}

fn get_som_eom(blackdetects: &mut Vec<String>, raw_video_duration: String) {
    println!("Number of blackdetects {}", blackdetects.len());
    println!("Video duration {}", raw_video_duration);

    let first_blackdetect = blackdetects.first_mut();

    match first_blackdetect {
        Some(b) => match extract_filter_prefix(b) {
            Ok((f, _)) => match get_filter_value(f, "black_end") {
                Some(v) => {
                    let frame = get_frame_per_timestamp(v);
                    let timecode = get_timecode(frame - 1);
                    println!("SOM (Start Of Material) Timecode {}", timecode);
                }
                None => panic!("black_start value for the last black detection not found"),
            },
            Err(e) => {
                panic!("error parsing the first black detection filter {}", e);
            }
        },
        None => {
            panic!("no black detect found");
        }
    }

    if blackdetects.len() > 1 {
        let last_blackdetect = blackdetects.last_mut();

        match last_blackdetect {
            Some(b) => match extract_filter_prefix(b) {
                Ok((f, _)) => match get_filter_value(f, "black_start") {
                    Some(v) => {
                        let frame = get_frame_per_timestamp(v);
                        let timecode = get_timecode(frame - 1);
                        println!("EOM (End of Material) Timecode: {}", timecode);
                    }
                    None => panic!("black_start value for the last black detection not found"),
                },
                Err(e) => {
                    panic!("error parsing the last black detection filter: {}", e);
                }
            },
            None => {
                panic!("no black detect found");
            }
        }
    } else {
        println!("EOM (End of Material) Timecode: {}", 10);
        println!("Atenção: Material Terminou no último segundo de vídeo, ou seja, não foi detectado nenhum 'black' no fim do vídeo");
    }
}

fn get_frame_per_timestamp(timestamp: f32) -> u64 {
    let frame = (timestamp * 29.97).round() as u64;
    return frame;
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

fn get_timecode(frame: u64) -> String {
    return Timecode::with_frames(frame, rates::F29_97_DF)
        .unwrap()
        .timecode();
}
