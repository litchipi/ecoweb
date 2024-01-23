# Ecoweb

Blog engine written in Rust.

Aims to do things minimally, but have room for any feature I may want to add later.

By itself, this engine is not enough, you need to start it with a specified blog root dir containing the necessary elements.  
You can see an example of such root on [my blog git repo](https://github.com/litchipi/blog).

Meant to be minimal, and configured / thought to ensure its environmental impact is as little as possible

## Posts

#### Storage

The storage is agnostic to the backend used, it can be local filesystem (current only implementation), but can also use any DB backend you want.

#### Format

The format used for posts is **Markdown** only, and it uses [mdtrans](https://github.com/litchipi/mdtrans) to convert them to HTML.

## Extensions

Some extensions are already created, such as:

- RSS feed
- Humans.txt file
- Webring metadata
- Hireme page
- Git webhook blog data update
