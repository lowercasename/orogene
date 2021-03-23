# Orogene

Orogene is a simple static site generator written in Rust. And when I say simple, I _really do mean simple_.

Given a source directory and the appropriate command line parameters, Orogene can do the following:

1. Convert Markdown and text files into HTML
2. Copy that HTML into a template field (`{{content}}`) in a specified HTML template file
3. **Optionally:** Copy the contents of a CSS file into another field (`{{style}}`) in the specified HTML template file
4. **Optionally:** Perform steps 1-3 again, but with a directory containing a set of blog posts, using their own template HTML file which renders their titles and date from YAML front matter, then generate a list of links to those posts and pop it in another template field (`{{post_list}}`) in one of your pages
5. Save the resultant HTML files into a new directory, optionally each within their own subdirectory
6. **Optionally:** Copy across a single assets directory into the new directory
7. Tell you how long that all took.

The benefits of Orogene are:

- It's fast. Like, _really_ fast.
- It's tiny. Like, _really_ tiny (~260 lines of code, excluding comments and whitespace).
- As a command line utility, it's completely customizable on the fly and can therefore be built into other scripts.
- It's named after a group of characters in a phenomenally good series of fantasy novels by [N. K. Jemisin](https://en.wikipedia.org/wiki/N._K._Jemisin).

Keep in mind:

- I think this is standard practice, but Orogene wipes and recreates its output directory every time it runs. **Don't keep any files in the ouput directory**!

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
    │   ├── blog.md
    │   ├── index.md
    │   ├── page1.md
    │   └── poetry.md
    ├── blog
    │   ├── 2019-07-22-a-blog-post.md
    │   ├── 2019-10-09-another-blog-post.md
    │   └── 2020-05-05-a-third-blog-post.md
    ├── style.css
    ├── list_template.html
    ├── post_template.html
    └── template.html
   ```
   Your `template.html` file should be a complete HTML file containing, in a single location, the string `{{content}}`, which will be replaced with your page content during generation, and optionally the string `{{style}}`, which can be replaced with the contents of a CSS file.
   If you're using the optional blog generator function:
      - Your `post_template.html` file should be an HTML file containing the strings `{{title}}`, `{{date}}`, and `{{content}}`, which will be replaced, respectively, with the `title` and `date` fields from the YAML front matter of each post, and the Markdown content of that post. This rendered file will in turn be included in the top-level `template.html` file, so you only need to write your top-level HTML once.
      - Your `archive_template.html` file should be an HTML file containing the strings `{{link}}` and `{{date}}`. For each post in your list of posts, these tags will be replaced, respectively, with an `<a>` to the post and its front matter `date` field.
      - You should also include the string `{{post_list}}` in one of your Markdown pages - Orogene will dump the generated HTML list of posts into this field.
4. Run Orogene. For the directory structure above, something like the following will do the trick:
   ```
   orogene --minify --directory-per-page --verbose --input-dir ./src/pages --output-dir ./build --template-file ./src/template.html --style-file ./src/style.css --assets-dir ./src/assets --blog-dir ./src/blog --post-template-file ./src/post_template.html
   ```
   You can of course also use short versions of all these flags:
   ```
   orogene -mdv -i ./src/pages -o ./build -t ./src/template.html -s ./src/style.css -a ./src/assets -b ./src/blog -p ./src/post_template.html
   ```
5. This will create the following website in a directory called `build`:
    ```
    build
    ├── assets
    │   ├── image.jpg
    │   ├── file.pdf
    │   └── meme.gif
    ├── blog
    │   ├── a-blog-post
    │   │   └── index.html
    │   ├── another-blog-post
    │   │   └── index.html
    │   └── a-third-blog-post
    │       └── index.html
    ├── index.html
    ├── page1
    │   └── index.html
    ├── page2
    │   └── index.html
    └── page2
        └── index.html
    ```

The simplest Orogene command would be something like this:

```
orogene -i ./src/pages -o ./build -t ./src/template.html
```

This will create one unstyled HTML file for each Markdown file provided to it - but you can, of course, simply include CSS directly in your template file.
