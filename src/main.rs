mod tests;

#[macro_use]
mod ffmpeg;

use nom::{
    bytes::complete::{tag, take_until},
    sequence::delimited,
    IResult,
};
use std::env;
use std::io::BufRead;
use std::process::Stdio;

// Crate to calculate timecode
use vtc::{rates, Timecode};

// The content needs to be the last black frame before the material ~ the last material frame;
// the frame found by black_end is a content frame and also must be subtract by 1 because the SOM must start with a blackframe
// the frame found by black_start to get EOM must be subtract by 1 because black_start is a black_frame and it does not belong to the end of material

fn main() {
    let args: Vec<String> = env::args().collect();
    args.get(1)
        .unwrap_or_else(|| panic!("error: missing video path argument"));

    run_ffmpeg_cmd(&args)
}

fn run_ffmpeg_cmd(args: &Vec<String>) {
    let file_input_arg = &args[1];

    let mut ffmpeg_cmd = ffmpeg![
        "-hide_banner",
        "-loglevel",
        "debug",
        "-i",
        file_input_arg,
        "-vf",
        "blackdetect=d=1:pix_th=0.00",
        "-f",
        "null",
        "-"
    ]
    .stderr(Stdio::piped())
    .spawn()
    .expect("error: running ffmpeg command, maybe bad path for a video of ffmpeg not found");

    let stderr = ffmpeg_cmd
        .stderr
        .take()
        .expect("error: getting the stderr output");
    let stderr_reader = std::io::BufReader::new(stderr);

    let mut blackdetect_list: Vec<String> = vec![];
    let mut raw_duration_line: Option<String> = None;
    for line in stderr_reader.lines() {
        let buf_line = line.expect("error: failed to read line from stdout");
        if buf_line.contains("Duration: ") {
            raw_duration_line = Some(buf_line);
        } else if buf_line.contains("black_start") {
            blackdetect_list.push(buf_line);
        }
    }

    get_som_eom(&mut blackdetect_list, raw_duration_line);
}

fn get_som_eom(blackdetects: &mut Vec<String>, raw_duration_line: Option<String>) {
    println!("Number of blackdetects {}", blackdetects.len());

    let first_blackdetect = blackdetects.first_mut();

    match first_blackdetect {
        Some(b) => match extract_filter_prefix(b) {
            Ok((f, _)) => match get_filter_value(f, "black_end") {
                Some(v) => {
                    let frame = get_frame_per_timestamp(v);
                    let timecode = get_timecode(frame - 1);
                    println!("SOM (Start Of Material) Timecode {}", timecode);
                }
                None => panic!("error: black_start value for the last black detection not found"),
            },
            Err(e) => {
                panic!("error: parsing the first black detection filter {}", e);
            }
        },
        None => {
            panic!("error: no black detect found");
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
                    None => panic!("black_start: value for the last black detection not found"),
                },
                Err(e) => {
                    panic!("error: parsing the last black detection filter: {}", e);
                }
            },
            None => {
                panic!("error: no black detect found");
            }
        }
    } else {
        let duration_str: Option<String>;
        match raw_duration_line {
            Some(v) => {
                let value = get_value_from_string("Duration", v);
                duration_str = value;
            }
            None => panic!("duration info not found on ffmpeg"),
        };

        let duration: f32;
        match duration_str {
            Some(v) => {
                let value = convert_video_ffmpeg_duration(v);
                duration = value;
            }
            None => panic!("error: parsing ffmpeg duration buffer line"),
        }

        let frame = get_frame_per_timestamp(duration);
        let timecode = get_timecode(frame - 1);
        println!("EOM (End of Material) Timecode: {}", timecode);
        println!("Atenção: Material Terminou no último segundo de vídeo, ou seja, não foi detectado nenhum 'black' no fim do vídeo");
    }
}

fn get_frame_per_timestamp(timestamp: f32) -> u64 {
    let frame = (timestamp * 29.97).round() as u64;
    return frame;
}

fn get_value_from_string(param: &str, input_string: String) -> Option<String> {
    let pattern = format!("{}:", param);
    if let Some(index) = input_string.find(&pattern) {
        let value_start = index + pattern.len();
        let value_string = input_string[value_start..].trim_start();
        if let Some(value_end) = value_string.find(' ') {
            return Some(value_string[..value_end].trim_end_matches(',').to_string());
        } else {
            return Some(value_string.trim_end_matches(',').to_string());
        }
    }
    None
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

fn convert_video_ffmpeg_duration(time_str: String) -> f32 {
    let time_parts: Vec<&str> = time_str.split(':').collect();
    let hours: f32 = time_parts[0].parse().unwrap();
    let minutes: f32 = time_parts[1].parse().unwrap();
    let seconds_parts: Vec<&str> = time_parts[2].split('.').collect();
    let seconds: f32 = seconds_parts[0].parse().unwrap();
    let milliseconds: f32 = seconds_parts[1].parse().unwrap();
    let total_seconds = hours * 3600.0 + minutes * 60.0 + seconds + milliseconds / 100.0;
    return total_seconds;
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
