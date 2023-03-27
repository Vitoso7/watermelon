struct MediaInfoGeneral {
    complete_name: String,
    format: String,
    commercial_name: String,
    format_version: String,
    format_profile: String,
    format_settings: String,
    file_size: String,
    duration: String,
    overall_bitrate: String,
    encoded_date: String,
    writting_application: String,
}

fn parse_general_output(input: Vec<Option<String>>) {
    for line in input {
        println!("{:?}", line);
    }

    // return MediaInfoGeneral {
    //     complete_name: String::from(""),
    //     format: String::from(""),
    //     commercial_name: String::from(""),
    // };
}

#[test]
fn parse_general_output_test() {}
