use handlebars::Handlebars;
use headless_chrome::browser::LaunchOptionsBuilder;
use headless_chrome::protocol::page::PrintToPdfOptions;
use headless_chrome::Browser;
use lazy_static::lazy_static;
use quick_error::quick_error;
use serde::Serialize;
use std::fs;
use std::sync::Mutex;
use std::time::Duration;
use tempfile::tempdir;

lazy_static! {
    static ref BROWSER: Mutex<Browser> = Mutex::new(
        Browser::new(
            LaunchOptionsBuilder::default()
                .idle_browser_timeout(Duration::from_secs(10 * 365 * 24 * 60 * 60))
                .build()
                .unwrap()
        )
        .unwrap()
    );
}

quick_error! {
    /// A combined error type for label generation
    #[derive(Debug)]
    pub enum LabelError {
        IOError(err: std::io::Error) {
            from()
            source(err)
            display("{}", err)
        }
        TemplateRenderError(err: handlebars::TemplateRenderError) {
            from()
            source(err)
            display("{}", err)
        }
        BrowserError(err: failure::Error) {
            from()
            // source(err)
            display("{}", err)
        }
    }
}

pub fn make_html<T>(template_name: &str, content: &T) -> Result<String, LabelError>
where
    T: Serialize,
{
    let reg = Handlebars::new();
    let template = fs::read_to_string(format!("./templates/{}.html.hbs", template_name))?;

    Ok(reg.render_template(&template, content)?)
}

pub struct PDF {
    _temp_dir: tempfile::TempDir,
    path: std::path::PathBuf,
    pub page_width_mm: f64,
    pub page_height_mm: f64,
}

impl PDF {
    pub fn get_path(&self) -> &std::path::PathBuf {
        &self.path
    }
}

pub fn make_pdf(html_string: String) -> Result<PDF, LabelError> {
    let browser = BROWSER.lock().unwrap();
    let temp_dir = tempdir()?;

    let html_path = temp_dir.path().join("label.html");
    let html_url = format!("file://{}", &html_path.to_str().unwrap());
    let pdf_path = temp_dir.path().join("label.pdf");
    fs::write(&html_path, &html_string)?;

    let tab = browser.wait_for_initial_tab()?;
    tab.navigate_to(&html_url)?;
    tab.wait_until_navigated()?;

    let label = tab.find_element("#label")?;
    let bounding_box = label.get_box_model()?;
    let page_width_in = bounding_box.content_viewport().width / 96.0;
    let page_height_in = bounding_box.content_viewport().height / 96.0;
    let page_width_mm = page_width_in * 25.4;
    let page_height_mm = page_height_in * 25.4;

    let pdf_options = PrintToPdfOptions {
        landscape: None,
        display_header_footer: None,
        print_background: None,
        scale: None,
        paper_width: Some(page_width_in as f32),
        paper_height: Some(page_height_in as f32),
        margin_top: Some(0.0),
        margin_bottom: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        page_ranges: Some("1".to_string()),
        ignore_invalid_page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: None,
    };
    let pdf_data = tab.print_to_pdf(Some(pdf_options))?;
    fs::write(&pdf_path, pdf_data)?;

    Ok(PDF {
        _temp_dir: temp_dir,
        path: pdf_path,
        page_width_mm,
        page_height_mm,
    })
}

pub fn make_label<T>(template_name: &str, content: &T) -> Result<PDF, LabelError>
where
    T: Serialize,
{
    let html = make_html(template_name, content)?;
    make_pdf(html)
}
