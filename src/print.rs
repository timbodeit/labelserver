use regex::Regex;
use std::io;
use std::io::Write;
use std::process::Command;

fn page_size_attribute_for_height(height_mm: f64) -> Result<String, io::Error> {
    let height = height_mm / 25.4 * 72.0;
    let ppd_file = std::fs::read_to_string("/etc/cups/ppd/QL600.ppd")?;

    let re: Regex =
        Regex::new(r#"\*PaperDimension (Br[A-z0-9]*)/(c[0-9]+x[0-9]+):\s"([0-9.]+) ([0-9.]+)""#)
            .unwrap();
    let mut result = "BrL063E01E745F9".to_owned();
    let mut penalty = 5000.0;

    for capture in re.captures_iter(&ppd_file) {
        let name = capture.get(1).unwrap().as_str();
        let h = capture.get(4).unwrap().as_str().parse::<f64>().unwrap();
        let distance = (height - h).abs();
        if distance < penalty {
            result = name.to_owned();
            penalty = distance;
        }
    }

    Ok(result)
}

pub fn print_pdf(pdf: crate::generator::PDF) -> Result<(), io::Error> {
    let output = Command::new("lp")
        .arg("-d")
        .arg("QL600")
        .arg("-o")
        .arg("BrTrimtape=OFF")
        .arg("-o")
        .arg("BrPriority=BrQuality")
        .arg("-o")
        .arg(format!(
            "PageSize={}",
            page_size_attribute_for_height(pdf.page_height_mm)?
        ))
        .arg(pdf.get_path().to_str().unwrap()) // use ok_or
        .output()?;

    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    Ok(())
}
