use std::io::BufRead;
use std::process::Stdio;
use std::{env, process::Command};

fn main() {
    let args: Vec<String> = env::args().collect();
    args.get(1).unwrap_or_else(|| panic!("missing argument"));

    run_ffmpeg_cmd(&args)
}

fn run_ffmpeg_cmd(args: &Vec<String>) {
    let file_input_arg = &args[1];

    println!("{}", file_input_arg);

    // TODO Convert frame to time example: 209 frame === 6.29
    // TODO Stop process when all needed values is found
    // TODO try using list of .arg
    // TODO identify the blackdetect data that has at least 5s and is at
    // the start of the video

    // ffmpeg -hide_banner -i
    // videos-promo-test/ARQUIVO_COMERCIAL_TESTE_ORIGINAL_8CH.mxf -vf
    // blackframe,blackdetect -f null -

    // ffprobe -hide_banner -f lavfi -i "movie=videos-promo-test/ARQUIVO_C
    // OMERCIAL_TESTE_ORIGINAL_8CH.mxf,blackframe,blackdetect" -show_entries frame=pkt_pts -of default=nw=1

    let mut ffprobe_cmd = Command::new("ffmpeg")
        .args(["-hide_banner", "-i"])
        .arg(file_input_arg)
        .args(["-vf", "blackframe,blackdetect", "-f", "null", "-"])
        .stderr(Stdio::piped())
        .spawn()
        .expect("error running ffprobe command");

    let stderr = ffprobe_cmd.stderr.take().unwrap();

    let stderr_reader = std::io::BufReader::new(stderr);

    // Kill ffmpeg process
    for line in stderr_reader.lines() {
        let buf_line = line.expect("Failed to read line from stdout");
        println!("{}", buf_line);
        if buf_line.contains("blackdetect") {
            ffprobe_cmd.kill().expect("bruh");
        }
    }
}
