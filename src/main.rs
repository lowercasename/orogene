use ansi_term::Style;
use chrono::{DateTime, NaiveDate};
use clap::Clap;
use comrak::{markdown_to_html, ComrakOptions};
use copy_dir::copy_dir;
use frontmatter::parse_and_find_content;
use html_minifier::HTMLMinifier;
use regex::Regex;
use std::cmp::Reverse;
use std::fs;
use std::time::Instant;

// Build CLI
#[derive(Clap)]
#[clap(
  name = "Orogene",
  version = "0.1.0",
  author = "Raphael Kabo <raphaelkabo@gmail.com>",
  about = "A simple static site generator."
)]
struct Opts {
  #[clap(short, long, about = "The directory containing your source files.")]
  input_dir: String,
  #[clap(short, long, about = "The directory where your site will be generated.")]
  output_dir: String,
  #[clap(short, long, about = "The HTML template file with which to build your pages.")]
  template_file: String,
  #[clap(short, long, about = "The directory containing your blog posts (optional).")]
  blog_dir: Option<String>,
  #[clap(
    short,
    long,
    about = "The HTML template file with which to build your posts (optional; required if --posts-dir is set)."
  )]
  post_template_file: Option<String>,
  #[clap(short, long, about = "The CSS file to attach to your pages (optional).")]
  style_file: Option<String>,
  #[clap(short, long, about = "The directory where your static assets are located (optional).")]
  assets_dir: Option<String>,
  #[clap(short, long, about = "Create a separate directory for each output file.")]
  directory_per_page: bool,
  #[clap(short, long, about = "Minify the output files.")]
  minify: bool,
  #[clap(short, long, about = "Display verbose generation output.")]
  verbose: bool,
}

fn parse_markdown(md_content: &str) -> String {
  // Parse Markdown
  let options = ComrakOptions {
    ext_autolink: true,
    unsafe_: true,
    ..ComrakOptions::default()
  };
  return markdown_to_html(md_content, &options);
}

fn generate_html(
  paths: Vec<std::fs::DirEntry>, output_directory: &str, with_style: bool, style_content: &String,
  template_content: &String, post_template_content: Option<&String>, with_frontmatter: bool,
  blog_posts_vector: Option<Vec<Vec<String>>>,
) -> Vec<Vec<String>> {
  let mut blog_posts = Vec::new();
  for entry in paths {
    let opts: Opts = Opts::parse();
    let file_path = entry.path();
    let mut file_name = file_path.file_stem().unwrap().to_string_lossy().to_string();

    // Check if the file name is preceeded with a date - we need to chop it off
    let re = Regex::new("^[0-9]{4}-[0-9]{2}-[0-9]{2}").unwrap();
    let matches = re.is_match(&file_name);
    if matches {
      let (_date, title) = file_name.split_at(11);
      file_name = title.to_string();
    }

    // Begin reading the file
    let file_content =
      fs::read_to_string(&file_path).expect("Something went wrong reading an input file");
    if opts.verbose {
      println!(
        "{} {}",
        Style::new().bold().paint("Generating HTML file:"),
        [&file_name, ".html"].concat()
      )
    }
    // let rendered_content;
    let mut result = String::new();
    // If we're using frontmatter, we need to extract the frontmatter here and incorporate it into our post template
    if with_frontmatter {
      let parse_result = parse_and_find_content(&file_content);
      let (front_matter, md_content) = parse_result.unwrap();
      let front_matter = front_matter.unwrap();
      let rendered_content = &parse_markdown(md_content);
      let title = &front_matter["title"].as_str().unwrap();
      let date = &front_matter["date"].as_str().unwrap();
      // Fill the post template with the post content and frontmatter
      if let Some(post_template_content) = post_template_content {
        let post_in_template = post_template_content
          .replace("{{title}}", title)
          .replace("{{date}}", date)
          .replace("{{content}}", rendered_content);
        // Then fill the page template with the rendered post
        result = template_content.replace("{{content}}", &post_in_template);
        let dir_name = opts.blog_dir.unwrap().split('/').last().unwrap().to_string();
        let blog_post = vec![dir_name, file_name.to_string(), title.to_string(), date.to_string()];
        blog_posts.push(blog_post);
      }
    } else {
      let rendered_content = parse_markdown(&file_content);
      result = template_content.replace("{{content}}", &rendered_content);
    }

    if with_style {
      if opts.verbose {
        println!("{}", Style::new().bold().paint("    Including CSS"))
      }
      result = result.replace("{{style}}", style_content);
    }

    if opts.minify {
      if opts.verbose {
        println!("{}", Style::new().bold().paint("    Minifying"))
      }
      let mut html_minifier = HTMLMinifier::new();
      html_minifier.digest(result).unwrap();
      result = html_minifier.get_html();
    }
    // Create the initial output filename
    let mut output_filename: String = [output_directory, "/", &file_name, ".html"].concat();
    // If we're creating a directory per file, change the output filename and create the directory
    if file_name != "index" && opts.directory_per_page {
      if opts.verbose {
        println!("{}", Style::new().bold().paint("    Creating page directory"))
      }
      let subfolder_path: String = [output_directory, "/", &file_name].concat();
      fs::create_dir(&subfolder_path).unwrap();
      output_filename = [&subfolder_path, "/index.html"].concat();
    }
    // Generate a list of blog posts, if we're building a blog
    if let Some(blog_posts_vector) = &blog_posts_vector {
      if result.contains("{{post_list}}") {
        let mut html = "".to_string();
        for x in blog_posts_vector.iter() {
          let line = format!(
                      "<article class='post-link'><a href='/{}/{}'>{}</a><time datetime='{}'>{}</time></article>",
                      &x[0], // Blog directory
                      &x[1], // Blog post filename
                      &x[2], // Blog post title
                      &x[3], // Blog post date
                      &x[3], // Blog post date
                    );
          html.push_str(&line);
        }
        result = result.replace("{{post_list}}", &html);
      }
    }

    fs::write(&output_filename, result).expect("Something went wrong saving a generated file");
    if opts.verbose {
      println!("{}", Style::new().bold().paint("    Writing file"))
    }
  }
  return blog_posts;
}

fn main() {
  // Start operation timer
  let operation_timer = Instant::now();
  // Fetch comand line arguments from Clap.
  let opts: Opts = Opts::parse();

  // Base variables
  let output_directory = &opts.output_dir;
  let input_paths: Vec<_> = fs::read_dir(opts.input_dir).unwrap().map(|r| r.unwrap()).collect();

  // Base HTML template
  let template_content =
    fs::read_to_string(opts.template_file).expect("Something went wrong reading the template file");

  // Style generation
  let mut with_style = false;
  let mut style_content: String = "".to_string();
  if let Some(style_file) = opts.style_file {
    style_content =
      fs::read_to_string(style_file).expect("Something went wrong reading the CSS file");
    with_style = true;
  }

  // Recreate the build directory first
  if opts.verbose {
    println!("{} {}", Style::new().bold().paint("Recreating build directory:"), &output_directory)
  }
  fs::remove_dir_all(&output_directory).unwrap();
  fs::create_dir(&output_directory).unwrap();

  // Copy the assets dir over, if we're using one
  if let Some(assets_dir) = opts.assets_dir {
    let dir_name: String = assets_dir.split('/').last().unwrap().to_string();
    // From src/foo to build/foo
    let dest_path = [&output_directory, "/", &dir_name].concat();
    if opts.verbose {
      println!(
        "{} {} > {}",
        Style::new().bold().paint("Copying assets directory:"),
        &assets_dir,
        &dest_path
      )
    }
    copy_dir(&assets_dir, &dest_path).unwrap();
  }

  // Optional blog post generation
  // If the blog posts directory is set...
  let mut blog_posts = Vec::new();
  if let Some(blog_dir) = opts.blog_dir {
    let mut blog_input_paths: Vec<_> =
      fs::read_dir(&blog_dir).unwrap().map(|r| r.unwrap()).collect();
    blog_input_paths.sort_by_key(|dir| Reverse(dir.path()));
    let post_template_content;
    let dir_name = blog_dir.split('/').last().unwrap().to_string();
    let dest_path = [&output_directory, "/", &dir_name].concat();
    // If the blog template file is set...
    if let Some(post_template_file) = opts.post_template_file {
      post_template_content = fs::read_to_string(post_template_file)
        .expect("Something went wrong reading the post template file");
      // Create the blog posts directory
      fs::create_dir(&dest_path).unwrap();
      blog_posts = generate_html(
        blog_input_paths,
        &dest_path,
        with_style,
        &style_content,
        &template_content,
        Some(&post_template_content),
        true,
        None,
      );
    }
  }

  // Loop through the top level of our input directory.
  generate_html(
    input_paths,
    output_directory,
    with_style,
    &style_content,
    &template_content,
    None,
    false,
    Some(blog_posts),
  );

  println!(
    "{} {}{}",
    Style::new().bold().paint("Done in"),
    operation_timer.elapsed().as_millis(),
    "ms"
  )
}
