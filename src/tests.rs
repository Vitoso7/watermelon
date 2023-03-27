#[cfg(test)]
mod tests {
    use crate::{
        extract_filter_prefix, get_filter_value, get_frame_per_timestamp, get_timecode,
        parse_video_duration,
    };

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

    #[test]
    fn get_frame_per_timestamp_6_97363() {
        let value = get_frame_per_timestamp(6.97363);
        assert_eq!(value, 209);
    }

    #[test]
    fn get_frame_per_timestamp_7_007() {
        let value = get_frame_per_timestamp(7.007);
        assert_eq!(value, 210);
    }

    #[test]
    fn get_frame_per_timestamp_22_0554() {
        let value = get_frame_per_timestamp(22.0554);
        assert_eq!(value, 661);
    }

    #[test]
    fn get_frame_per_timestamp_24_024000() {
        let value = get_frame_per_timestamp(24.024000);
        assert_eq!(value, 720);
    }

    #[test]
    fn get_frame_per_timestamp_37_037() {
        let value = get_frame_per_timestamp(37.037);
        assert_eq!(value, 1110);
    }

    #[test]
    fn get_frame_per_timestamp_21_9553() {
        let value = get_frame_per_timestamp(21.9553);
        assert_eq!(value, 658);
    }

    #[test]
    fn get_frame_per_timestamp_6_94027() {
        let value = get_frame_per_timestamp(6.94027);
        assert_eq!(value, 208);
    }

    #[test]
    fn get_value_from_string_duration() {
        let value = {
            let param = "Duration";
            let input_string = "Duration: 00:00:39.04, start: 0.000000, bitrate: 60024 kb/s";
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
        };
        assert_eq!(value, Some(String::from("00:00:39.04")));
    }

    #[test]
    fn get_value_from_string_black_start() {
        let value = {
            let param = "black_start";
            let input_string = "[blackdetect @ 0x12ce05c50] black_start:37.037 black_end:39.0056 black_duration:1.96863";
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
        };
        assert_eq!(value, Some(String::from("37.037")));
    }

    #[test]
    fn parse_video_duration_00_00_39_04() {
        let value = parse_video_duration(String::from("00:00:39.04"));
        assert_eq!(value, 39.04)
    }

    #[test]
    fn parse_video_duration_00_01_39_04() {
        let value = parse_video_duration(String::from("00:01:39.04"));
        assert_eq!(value, 99.04);
    }
}
