# Orogene

Orogene is a simple static site generator written in Rust. And when I say simple, I _really do mean simple_.

Given a source directory and the appropriate command line parameters, Orogene can do the following:

- Convert Markdown and text files into HTML
- Copy that HTML into a single location in a specified HTML template file
- Copy the contents of a CSS file into another location in the specified HTML template file
- Save the resultant HTML files into a new directory, optionally each within their own subdirectory
- Copy across a single assets directory into the new directory
- Tell you how long that all took.

The benefits of Orogene are:

- It's fast. Like, _really_ fast.
- It's tiny. Like, _really_ tiny (79 lines of code, excluding comments and whitespace).
- It's completely customizable on the fly and can therefore be built into other scripts.
- It's named after a group of characters in a phenomenally good series of fantasy novels by [N. K. Jemisin](https://en.wikipedia.org/wiki/N._K._Jemisin).

# Get started

Orogene needs a couple of things to be set up just the way it likes.

1. First, you'll need an up-to-date [Rust installation](https://www.rust-lang.org/learn/get-started).
2. Clone this GitHub repository and build the binary file:
   ```
   $ git clone https://github.com/lowercasename/orogene
   $ cd orogene
   $ cargo build --release
   ```
   This will create a binary file at `target/release/orogene`, which you can add to your `$PATH` or move to `/usr/local/bin`.
3. Orogene will use any set of directories you provide it with, but I prefer the following directory structure to keep everything organised:
   ```
    src
    ├── assets
    │   ├── image.jpg
    │   ├── file.pdf
    │   └── meme.gif
    ├── pages
    │   ├── index.md
    │   ├── page1.md
    │   └── poetry.md
    ├── style.css
    └── template.html
   ```
   Your `template.html` file should be a complete HTML file containing, in a single location, the string `{{content}}`, which will be replaced with your page content during generation, and optionally the string `{{style}}`, which can be replaced with the contents of a CSS file.
4. Run Orogene. For the directory structure above, something like the following will do the trick:
   ```
   orogene --input-dir ./src/pages --output-dir ./build --template-file ./src/template.html --style-file ./src/style.css --assets-dir ./src/assets --minify --directory-per-page --verbose
   ```
   You can of course also use short versions of all these flags:
   ```
   orogene -mdv -i ./src/pages -o ./build -t ./src/template.html -s ./src/style.css -a ./src/assets
   ```
5. This will create the following website in a directory called `build`:
  ```
  build
  ├── assets
  │   ├── image.jpg
  │   ├── file.pdf
  │   └── meme.gif
  ├── index.html
  ├── page1
  │   └── index.html
  └── page2
      └── index.html
  ```