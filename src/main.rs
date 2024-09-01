use log::{info, debug};
use log::Level::{Info, Debug};
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use xmltree::Element;
use xmltree::XMLNode;
use zip::ZipArchive;
use zip::ZipWriter;


fn main() {
    simple_logger::init_with_level(Debug).unwrap();

    // ------ vars 
    let input_path = String::from("./input/");
    let output_path = String::from("./output/");

    let input_filename = String::from("random.epub");
    let output_filename = String::from(format!("test_{}", input_filename));

    let input_filename_with_path = String::from(format!("{}{}", input_path, input_filename));
    let output_filename_with_path = String::from(format!("{}{}", output_path, output_filename));

    let input_filename_with_path_buf = PathBuf::from(input_filename_with_path);
    let output_filename_with_path_buf = PathBuf::from(output_filename_with_path);
    // ------

    let _ = process(input_filename_with_path_buf, output_filename_with_path_buf);
}

fn process(input_path: PathBuf, output_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut input_zip = ZipArchive::new(input_file)?;

    let output_file = File::create(output_path)?;
    let mut output_zip = ZipWriter::new(output_file);

    for i in 0..input_zip.len() {
        let mut file = input_zip.by_index(i).unwrap();
        debug!("Processing file: {}", file.name());

        let mut buf = Vec::new();
        let _ = file.read_to_end(&mut buf);
        let re = Regex::new(r".*html$").unwrap();
        let files_to_ignore: Vec<&str> = vec!["cover", "nav"];

        if re.is_match(file.name()) {
            // this is two splits in a row
            let filename_split: Vec<&str> = file.name()
                .split_terminator('/')
                .flat_map(|s| s.split('.'))
                .collect();

            if !files_to_ignore.contains(&filename_split[filename_split.len() - 2]) {
                debug!("Convertinf file: {}", file.name());
                buf = modify_xml(&buf);
            }
        }

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        output_zip.start_file(file.name(), options).unwrap();
    
        let _ = output_zip.write(&buf);
    }
    Ok(())
}

fn modify_xml(buf: &[u8], ) -> Vec<u8> {
    let mut names_element = Element::parse(buf).unwrap();

    mutate_text(&mut names_element);

    let mut out_buf: Vec<u8> = vec![];
    names_element.write(&mut out_buf).unwrap();
    out_buf
}

fn mutate_text(element: &mut Element) {
    for node in element.children.iter_mut() {
        match node {
            XMLNode::Element(ref mut elem) => {
                if elem.name != "b" {
                    mutate_text(elem)
                }
            }
            XMLNode::Text(ref mut text) => {
                let bionic: Vec<String> = text.split_whitespace().map(to_bionic).collect();
                let joined = bionic.join(" ");

                let bionic_string = format!("<bionic> {} </bionic>", joined);

                let bionic_element = Element::parse(bionic_string.as_bytes()).unwrap();
                *node = xmltree::XMLNode::Element(bionic_element);
            }
            _ => (),
        }
    }
}

fn to_bionic(word: &str) -> String {
    let trimmed_word = word.trim().replace('&', "&amp;");
    let chars: Vec<char> = trimmed_word.chars().collect();
    let mid_point = chars.len() / 2;

    if mid_point >= chars.len() || chars.is_empty() {
        return trimmed_word;
    }

    if chars.len() == 1 {
        return if chars[0].is_ascii() {
            format!("<b>{} </b>", trimmed_word)
        } else {
            trimmed_word
        };
    }

    let (bold, remaining) = chars.split_at(mid_point);
    let bold_string = String::from_iter(bold);
    let remaining_string = String::from_iter(remaining);

    format!("<b>{}</b>{}", bold_string, remaining_string).replace('&', "&amp;")
}
