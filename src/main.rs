use std::io::BufRead;
use std::process::Stdio;
use std::{env, process::Command};

const FRAME_RATE: f32 = 29.97;

fn main() {
    let args: Vec<String> = env::args().collect();
    args.get(1).unwrap_or_else(|| panic!("missing argument"));

    run_ffmpeg_cmd(&args)
}

fn run_ffmpeg_cmd(args: &Vec<String>) {
    let file_input_arg = &args[1];

    // TODO Convert frame to time example: 209 frame === 6.29
    // TODO Stop process when all needed values is found
    // TODO try using list of .arg
    // TODO identify the blackdetect data that has at least 5s and is at the start of the video

    // ffmpeg -hide_banner -i
    // videos-promo-test/ARQUIVO_COMERCIAL_TESTE_ORIGINAL_8CH.mxf -vf
    // blackframe,blackdetect -f null -

    // ffprobe -hide_banner -f lavfi -i "movie=videos-promo-test/ARQUIVO_C
    // OMERCIAL_TESTE_ORIGINAL_8CH.mxf,blackframe,blackdetect" -show_entries frame=pkt_pts -of default=nw=1

    // Running FFMPEG on Docker
    let mut ffprobe_cmd = Command::new("docker")
        .args(["exec", "e99da464f45d", "ffmpeg", "-hide_banner", "-i"])
        .arg(format!("/{}", file_input_arg))
        .args(["-vf", "blackframe,blackdetect", "-f", "null", "-"])
        .stderr(Stdio::piped())
        .spawn()
        .expect("error running ffprobe command");

    let stderr = ffprobe_cmd.stderr.take().unwrap();

    let stderr_reader = std::io::BufReader::new(stderr);

    for line in stderr_reader.lines() {
        let buf_line = line.expect("Failed to read line from stdout");
        println!("{}", buf_line);
        if buf_line.contains("blackdetect") {
            ffprobe_cmd.kill().expect("failed to kill ffmpeg process");
            calc_timecode_by_frame(210, FRAME_RATE);
            calc_timecode_by_timestamp(4.004, FRAME_RATE as f64);
        }
    }
}

fn calc_timecode_by_frame(frame: u16, frame_rate: f32) -> String {
    let total_elapsed_time = frame as f32 / frame_rate;
    println!("{}", total_elapsed_time);

    let hours = (total_elapsed_time / 3600.0).floor();
    let minutes = ((total_elapsed_time - (hours * 3600.0)) / 60.0).floor();
    let seconds = (total_elapsed_time - (hours * 3600.0) - (minutes * 60.0)).floor();
    let frames =
        ((total_elapsed_time - (hours * 3600.0) - (minutes * 60.0) - seconds) * frame_rate).round();

    let timecode = format!("{:02}:{:02}:{:02}:{:02}", hours, minutes, seconds, frames);

    println!("timecode from frame = {}", timecode);

    return timecode;
}

fn calc_timecode_by_timestamp(timestamp: f64, frame_rate: f64) -> String {
    let hours = (timestamp / 3600.0).floor();
    let minutes = ((timestamp / 60.0) % 60.0).floor();
    let seconds = (timestamp % 60.0).floor();
    let frames = ((timestamp * frame_rate) % frame_rate).round();

    let timecode = format!("{:02}:{:02}:{:02}:{:02}", hours, minutes, seconds, frames);

    println!("timecode from timestamp = {}", timecode);

    return timecode;
}

#[cfg(test)]
mod tests {
    use crate::{calc_timecode_by_frame, calc_timecode_by_timestamp, FRAME_RATE};

    #[test]
    fn calc_timecode_frame_209() {
        let timecode = calc_timecode_by_frame(209, FRAME_RATE);
        assert_eq!(timecode, "00:00:06:29");
    }

    #[test]
    fn calc_timecode_frame_210() {
        let timecode = calc_timecode_by_frame(210, FRAME_RATE);
        assert_eq!(timecode, "00:00:07:00");
    }

    #[test]
    fn calc_timecode_by_timestamp_6_973633() {
        let timecode = calc_timecode_by_timestamp(6.973633, FRAME_RATE as f64);
        assert_eq!(timecode, "00:00:06:29");
    }

    #[test]
    fn calc_timecode_by_timestamp_7_007() {
        let timecode = calc_timecode_by_timestamp(7.007, FRAME_RATE as f64);
        assert_eq!(timecode, "00:00:07:00")
    }

    #[test]
    fn calc_timecode_by_timestamp_4_004() {
        let timecode = calc_timecode_by_timestamp(4.004, FRAME_RATE as f64);
        assert_eq!(timecode, "00:00:04:00");
    }
}
