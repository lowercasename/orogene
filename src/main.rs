use ansi_term::Style;
use chrono::{NaiveDate, NaiveTime, NaiveDateTime, DateTime, FixedOffset, TimeZone};
use clap::Parser;
use comrak::{markdown_to_html, ComrakOptions};
use copy_dir::copy_dir;
// use frontmatter::parse_and_find_content;
use gray_matter::engine::YAML;
use gray_matter::Matter;
use html_minifier::HTMLMinifier;
use regex::Regex;
use serde::Deserialize;
use std::cmp::Reverse;
use std::fs;
use std::io;
use std::time::Instant;
use rss::{ChannelBuilder, Item, ItemBuilder, Guid};
use toml;

#[derive(Deserialize, Debug)]
pub struct FrontMatter {
  title: Option<String>,
  date: Option<String>,
  // tags: Option<Vec<String>>,
}

// pub struct Post {
//   front_matter: FrontMatter,
//   content: String,
// }

// pub struct PostList(Vec<Post>);

// impl PostList {
//   pub fn create_post(&mut self, front_matter: FrontMatter, content: String) -> &Post {
//     let new_post = Post {
//       front_matter,
//       content,
//     };
//     self.0.push(new_post);
//     return &self.0[self.0.len() - 1];
//   }
// }

#[derive(Clone, Debug)]
pub struct CompilationResult {
  html: String,
  plain: String,
  title: Option<String>,
  date: Option<NaiveDate>,
  url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Config {
  title: String,
  url: String,
  description: String,
}

fn read_config() -> std::io::Result<Config> {
  let opts: Opts = Opts::parse();
  let content = std::fs::read_to_string(opts.config_file.unwrap())?;
  Ok(toml::from_str(&content)?)
}

// Build CLI
#[derive(Parser, Debug)]
#[clap(
  name = "Orogene",
  version = "0.2.0",
  author = "Raphael Kabo <mail@raphaelkabo.com>",
  about = "A simple static site generator."
)]
struct Opts {
  // The directory containing your source files.
  #[clap(short, long)]
  input_dir: String,
  // The directory where your site will be generated.
  #[clap(short, long)]
  output_dir: String,
  // The HTML template file with which to build your pages.
  #[clap(short, long)]
  template_file: String,
  // The directory containing your blog posts (optional).
  #[clap(short, long)]
  blog_dir: Option<String>,
  // The HTML template file with which to build your posts (optional; required if --blog-dir is set).
  #[clap(short, long)]
  post_template_file: Option<String>,
  // The HTML template file with which to build your post list entries (optional).
  #[clap(short, long)]
  list_template_file: Option<String>,
  // The CSS file to attach to your pages (optional).
  #[clap(short, long)]
  style_file: Option<String>,
  // The directory where your static assets are located (optional).
  #[clap(short, long)]
  assets_dir: Option<String>,
  // Create a separate directory for each output file.
  #[clap(short, long)]
  directory_per_page: bool,
  // Minify the output files.
  #[clap(short, long)]
  minify: bool,
  // Display verbose generation output.
  #[clap(short, long)]
  verbose: bool,
  // Generate an RSS feed.
  #[clap(short, long)]
  feed: bool,
  // The location of the config file with RSS feed settings.
  #[clap(short, long)]
  config_file: Option<String>
}

fn parse_markdown(md_content: &str) -> String {
  // Parse Markdown
  let options = ComrakOptions {
    ext_autolink: true,
    unsafe_: true,
    github_pre_lang: true,
    ..ComrakOptions::default()
  };
  return markdown_to_html(md_content, &options);
}

// Compiles a list of all the posts in the blog and returns it as an HTML string.
fn compile_post_list(
  template: String,
  posts: Vec<CompilationResult>
) -> String {
  let mut html = "".to_string();
  // let reversed_posts: Vec<CompilationResult> = posts.into_iter().rev().collect();
  for x in posts.iter() {
    let date = x.date.as_ref().unwrap();
    let iso_date = &date.format("%Y-%m-%d").to_string();
    let formatted_date = date.format("%e %h %Y").to_string();
    let title = x.title.as_ref().unwrap();
    let url = x.url.as_ref().unwrap();
    // Fill the post list template with each post's metadata and title
    if template != "" {
      let archive_line = template
        .replace("{{url}}", &url)
        .replace("{{link}}", &format!("<a href='{}'>{{{{title}}}}</a>", &url))
        .replace("{{title}}", &title)
        .replace("{{date}}", &format!("<time datetime='{}'>{}</time>", &iso_date, &formatted_date));
        html.push_str(&archive_line);
    }
    // Fall back to a default template otherwise
    else {
      let archive_line = format!(
        "<article class='post-link'><a href='{}'>{}</a><time datetime='{}'>{}</time></article>",
        &url, 
        &title, 
        &iso_date, 
        &formatted_date, 
      );
      html.push_str(&archive_line);
    }
  }
  return html;
}

fn compile_feed(posts: Vec<CompilationResult>) -> String {
  let opts: Opts = Opts::parse();
  if opts.verbose {
    println!("{}", Style::new().bold().paint("Generating RSS feed"));
  }
  let config = read_config().unwrap();
  let items: Vec<Item> = posts.into_iter().map(|post| {
    // Time is hard
    let time = NaiveTime::from_hms(0, 0, 0);
    let tz_offset = FixedOffset::east(0);
    let datetime = NaiveDateTime::new(post.date.unwrap_or(NaiveDate::from_ymd(1970, 1, 1)), time);
    let datetime_with_tz: DateTime<FixedOffset> = tz_offset.from_local_datetime(&datetime).unwrap();
    let guid = {
      let mut guid = Guid::default();
      guid.set_value(format!("{}{}", config.url, post.url.to_owned().unwrap_or("".to_string())));
      guid.set_permalink(true);
      guid
    };
    ItemBuilder::default()
      .title(post.title)
      .link(format!("{}{}", config.url, post.url.unwrap_or("".to_string())))
      .guid(guid)
      .pub_date(datetime_with_tz.to_rfc2822())
      .description(post.plain)
      .build()
  }).collect();
  let channel = ChannelBuilder::default()
    .title(config.title)
    .link(config.url)
    .description(config.description)
    .items(items)
    .build();
  let output = channel.to_string();
  return output;
}

// Combine an input Markdown file with an HTML template file. Returns an HTML string.
fn compile_html(
  input_file: std::path::PathBuf,
  parent_template_content: &String,
  item_template_content: Option<String>,
  style_file: Option<String>,
  with_frontmatter: bool,
  verbose: bool,
  minify: bool,
  posts: Option<Vec<CompilationResult>>,
) -> Result<CompilationResult, io::Error> {
  // Begin reading the file
  let input_content =
    fs::read_to_string(&input_file).expect("Something went wrong reading an input file");

  let mut compilation_result;
  if with_frontmatter {
    // If we're using frontmatter, we need to extract the frontmatter here
    // and incorporate it into our template
    let matter = Matter::<YAML>::new();
    let result = matter.parse_with_struct::<FrontMatter>(&input_content).expect("Something went wrong reading front matter. Make sure your blog posts start with a YAML front matter block.");
    let rendered_content = parse_markdown(&result.content);
    let title = result.data.title.unwrap_or("Untitled".to_string());
    let date_string = result.data.date.unwrap_or("1970-01-01".to_string());
    let date = NaiveDate::parse_from_str(&date_string, "%Y-%m-%d").unwrap();
    let formatted_date = date.format("%e %h %Y").to_string();
    // If there's an optional item template, then this item needs to be compiled
    // into its item template first, and then that needs to be compiled into the
    // parent template
    if let Some(item_template_content) = item_template_content {
      // Fill the item template with the item content and frontmatter
      let item_html = item_template_content
        .replace("{{title}}", &title)
        .replace("{{date}}", &formatted_date)
        .replace("{{content}}", &rendered_content);
      // Fill the parent template with the rendered item
      let html = parent_template_content.replace("{{content}}", &item_html);
      compilation_result = CompilationResult {
        plain: result.content,
        html,
        date: Some(date),
        title: Some(title),
        url: None,
      };
    } else {
      // There's no item template, just fill the parent template with the rendered item body
      let html = parent_template_content.replace("{{content}}", &rendered_content);
      compilation_result = CompilationResult {
        plain: result.content,
        html,
        date: None,
        title: None,
        url: None,
      };
    }
  } else {
    // We're not using frontmatter, just render the entire file into the template
    let rendered_content = parse_markdown(&input_content);
    let html;
    if let Some(item_template_content) = item_template_content {
      // Fill the item template with the rendered file
      let item_html = item_template_content
        .replace("{{content}}", &rendered_content);
      // Fill the parent template with the rendered item
      html = parent_template_content.replace("{{content}}", &item_html);
    } else {
      // There's no item template, just fill the parent template with the file
      html = parent_template_content.replace("{{content}}", &rendered_content);
    }
    compilation_result = CompilationResult {
      plain: input_content,
      html,
      date: None,
      title: None,
      url: None,
    };
  }

  if let Some(style_file) = style_file {
    let style_content =
      fs::read_to_string(style_file).expect("Something went wrong reading the CSS file");
    if verbose {
      println!("{}", Style::new().bold().paint("    Including CSS"))
    }
    compilation_result.html = compilation_result.html.replace("{{style}}", &style_content);
  }

  // Generate a list of blog posts, if we're building a blog and this HTML template
  // contains the {{post_list}} code
  if compilation_result.html.contains("{{post_list}}") {
    let opts: Opts = Opts::parse();
    let mut post_list_template_content: String = "".to_string();
    if let Some(post_list_template_file) = opts.list_template_file {
      post_list_template_content = fs::read_to_string(post_list_template_file)
        .expect("Something went wrong reading the archive template file");
    }
    if let Some(posts) = posts {
      let post_list = compile_post_list(post_list_template_content, posts);
      compilation_result.html = compilation_result.html.replace("{{post_list}}", &post_list);
    }
  }
  if minify {
    if verbose {
      println!("{}", Style::new().bold().paint("    Minifying"))
    }
    let mut html_minifier = HTMLMinifier::new();
    html_minifier.digest(compilation_result.html).expect("Something went wrong minifying HTML");
    compilation_result.html = html_minifier.get_html();
  }
  Ok(compilation_result)
}

fn build_dist_directory(
  source_directory: &String, dist_directory: &String, parent_template_content: &String, item_template_content: Option<String>, create_subdirectory: bool, with_frontmatter: bool, posts: Option<Vec<CompilationResult>>) -> Vec<CompilationResult> {
  let mut items = Vec::new();
  let opts: Opts = Opts::parse();
  // Our initial output path is simply the base dist directory.
  let mut output_path: String = dist_directory.to_string();
  // If we're creating a subdirectory containing our output files (for instance if they're blog
  // posts in a /blog/ subdirectory)
  if create_subdirectory {
    // The subdirectory name is the same as the final segment of the source directory name
    let subdirectory_name = source_directory.split('/').last().unwrap().to_string();
    output_path = [&dist_directory, "/", &subdirectory_name].concat();
    // Create the subdirectory
    fs::create_dir(&output_path).unwrap();
  }

  // Create a sorted vector of all the input files in the source directory
  let mut input_paths: Vec<_> =
    fs::read_dir(&source_directory).unwrap().map(|r| r.unwrap()).collect();
  input_paths.sort_by_key(|dir| Reverse(dir.path()));

  // Process the input files
  for entry in input_paths {
    let file_path = entry.path();
    let file_extension = file_path.extension().unwrap().to_string_lossy();
    // Only read .md and .txt files, ignore everything else
    if file_extension == "md" || file_extension == "txt" {
      let mut file_name = file_path.file_stem().unwrap().to_string_lossy().to_string();

      // Check if the file name is preceeded with a date - we need to chop it off
      let re = Regex::new("^[0-9]{4}-[0-9]{2}-[0-9]{2}").unwrap();
      let matches = re.is_match(&file_name);
      if matches {
        let (_date, title) = file_name.split_at(11);
        file_name = title.to_string();
      }
      if opts.verbose {
        println!(
          "{} {}",
          Style::new().bold().paint("Generating HTML file:"),
          [&file_name, ".html"].concat()
        )
      }
      let mut compilation_result =
        compile_html(entry.path(), &parent_template_content, item_template_content.to_owned(), opts.style_file.to_owned(), with_frontmatter, opts.verbose, opts.minify, posts.clone())
          .unwrap();
      // Create the initial output filename
      let mut output_filename: String = [&output_path, "/", &file_name, ".html"].concat();
      // If we're creating a directory per file, change the output filename and create the directory
      if file_name != "index" && opts.directory_per_page {
        if opts.verbose {
          println!("{}", Style::new().bold().paint("    Creating page directory"))
        }
        let subfolder_path: String = [&output_path, "/", &file_name].concat();

        // In some cases, in directory-per-page mode, we have a page with the name 'blog' or
        // 'posts', and our blog posts directory is also called 'blog' or 'posts'. In this case,
        // instead of creating a new subfolder, we just put the index.html file in the already
        // existent directory.
        let dir_name = opts.blog_dir.to_owned().unwrap().split('/').last().unwrap().to_string();
        if file_name != dir_name {
          fs::create_dir(&subfolder_path).unwrap();
        }
        output_filename = [&subfolder_path, "/index.html"].concat();
      }
      // Update the URL field in our compilation result
      if opts.directory_per_page {
        // In directory-per-page mode, URLs are /subdirectory/post-name
        // The subdirectory is optional
        if create_subdirectory {
          // The subdirectory name is the same as the final segment of the source directory name
          let subdirectory_name = source_directory.split('/').last().unwrap().to_string();
          compilation_result.url = Some(["/", &subdirectory_name, "/", &file_name].concat());
        } else {
          compilation_result.url = Some(["/", &file_name].concat());
        }
      } else {
        // In file-per-page mode, URLS are /subdirectory/post-name.html
        // The subdirectory is optional
        if create_subdirectory {
          // The subdirectory name is the same as the final segment of the source directory name
          let subdirectory_name = source_directory.split('/').last().unwrap().to_string();
          compilation_result.url = Some(["/", &subdirectory_name, "/", &file_name, ".html"].concat());
        } else {
          compilation_result.url = Some(["/", &file_name, ".html"].concat());
        }
      }
      fs::write(&output_filename, &compilation_result.html).expect("Something went wrong saving a generated file");
      if opts.verbose {
        println!("{}", Style::new().bold().paint("    Writing file"))
      }
      // Add the item to the items vector
      items.push(compilation_result);
    }
  }
  return items;
}

fn main() {
  // Start operation timer
  let operation_timer = Instant::now();
  // Fetch comand line arguments from Clap.
  let opts: Opts = Opts::parse();

  // Base variables
  let output_directory = &opts.output_dir;

  // Recreate the build directory first
  if opts.verbose {
    println!("{} {}", Style::new().bold().paint("Recreating build directory:"), &output_directory)
  }
  // Only delete the build directory if it exists
  if fs::metadata(&output_directory).is_ok() {
    fs::remove_dir_all(&output_directory).unwrap();
  }
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

  // Generate main template content
  let template_content =
    fs::read_to_string(opts.template_file).expect("Something went wrong reading the template file");
  // let style_file = Some(opts.style_file).unwrap();

  // Optional blog post generation
  let mut post_template_content = String::from("");
  if let Some(post_template_file) = opts.post_template_file {
    post_template_content = fs::read_to_string(post_template_file).unwrap_or("".to_string());
  }
  // If the blog posts directory is set...
  let mut blog_posts = Vec::new();
  if let Some(blog_dir) = opts.blog_dir {
    blog_posts = build_dist_directory(&blog_dir, &output_directory, &template_content, Some(post_template_content), true, true, None);
  }

  // Page generation
  build_dist_directory(&opts.input_dir, &output_directory, &template_content, None, false, false, Some(blog_posts.to_owned()));

  // Optional feed generation
  if opts.feed {
    let feed = compile_feed(blog_posts);
    let feed_path = [&output_directory, "/feed.rss"].concat();
    fs::write(&feed_path, &feed).expect("Something went wrong saving the RSS feed");
  }

  println!(
    "{} {}{}",
    Style::new().bold().paint("Done in"),
    operation_timer.elapsed().as_millis(),
    "ms"
  )
}
