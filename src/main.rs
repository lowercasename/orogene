use comrak::{markdown_to_html, ComrakOptions};
use html_minifier::HTMLMinifier;
use copy_dir::copy_dir;
use rustop::opts;
use std::{fs, path};
use ansi_term::Style;
use std::time::{Instant};

fn main() {
  // Start operation timer
  let operation_timer = Instant::now();
  // Rustop sets up our command line arguments.
  let (args, _rest) = opts! {
      synopsis "A simple static site generator.";
      opt input_dir:String, desc:"The directory containing your source files.";
      opt output_dir:String, desc:"The directory where your site will be generated.";
      opt assets_dir:Option<String>, desc:"The directory where your static assets are located (optional).";
      opt template_file:String, desc:"The HTML template file with which to build your pages.";
      opt style_file:Option<String>, desc:"The CSS file to attach to your pages (optional).";
      opt directory_per_page:bool, desc:"Create a separate directory for each output file.";
      opt minify:bool, desc:"Minify the output files.";
      opt verbose:bool, desc:"Display verbose conversion output.";
  }
  .parse_or_exit();

  let output_directory = &args.output_dir;
  let paths = fs::read_dir(args.input_dir).unwrap();
  let template_content =
    fs::read_to_string(args.template_file).expect("Something went wrong reading the template file");
  // I think these have to be set initially in case a style file isn't being used
  let mut with_style = false;
  let mut style_content: String = "".to_string();
  if let Some(style_file) = args.style_file {
    style_content =
      fs::read_to_string(style_file).expect("Something went wrong reading the CSS file");
    with_style = true;
  }
  // Wipe the build directory first
  if args.verbose { println!("{} {}", Style::new().bold().paint("Recreating build directory:"), &output_directory) }
  fs::remove_dir_all(&output_directory).unwrap();
  fs::create_dir(&output_directory).unwrap();
  // Copy the assets dir over, if we're using one
  if let Some(assets_dir) = args.assets_dir {
    let dir_name: String = assets_dir.split('/').last().unwrap().to_string();
    // From src/foo to build/foo
    let dest_path = [&output_directory, "/", &dir_name].concat();
    if args.verbose { println!("{} {} > {}", Style::new().bold().paint("Copying assets directory:"), &assets_dir, &dest_path) }
    copy_dir(&assets_dir, &dest_path).unwrap();
  }
  // Loop through the top level of our parent directory.
  for entry in paths {
    let entry = entry.unwrap();
    let file_path = entry.path();
    let file_name = file_path.file_stem().unwrap().to_string_lossy();
    let md_content =
    fs::read_to_string(&file_path).expect("Something went wrong reading an input file");
    if args.verbose { println!("{} {}", Style::new().bold().paint("Generating HTML file:"), [&file_name,".html"].concat()) }
    // Comrak is our Markdown parser
    let options = ComrakOptions {
      ext_autolink: true,
      unsafe_: true,
      ..ComrakOptions::default()
    };
    let rendered_content = markdown_to_html(&md_content, &options);
    let mut result = str::replace(&template_content, "{{content}}", &rendered_content);
    if with_style {
      if args.verbose { println!("{}", Style::new().bold().paint("    Including CSS")) }
      result = str::replace(&result, "{{style}}", &style_content);
    }
    if args.minify {
      if args.verbose { println!("{}", Style::new().bold().paint("    Minifying")) }
      let mut html_minifier = HTMLMinifier::new();
      html_minifier.digest(result).unwrap();
      result = html_minifier.get_html();
    }
    // Create the initial output filename
    let mut output_filename: String = [output_directory, "/", &file_name, ".html"].concat();
    // If we're creating a directory per file, change the output filename and create the directory
    if file_name != "index" && args.directory_per_page {
      if args.verbose { println!("{}", Style::new().bold().paint("    Creating page directory")) }
      let subfolder_path: String = [output_directory, "/", &file_name].concat();
      if !path::Path::new(&subfolder_path).exists() {
        fs::create_dir(&subfolder_path).unwrap();
      }
      output_filename = [&subfolder_path, "/index.html"].concat();
    }
    fs::write(&output_filename, result).expect("Unable to write file");
    if args.verbose { println!("{}", Style::new().bold().paint("    Writing file")) }
  }

  if args.verbose { println!("{} {}{}", Style::new().bold().paint("Done in"), operation_timer.elapsed().as_millis(), "ms") }
}
