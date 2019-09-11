# wiev

## Summary

**wiev** is a simple GPU-accelerated image viewer written in [Rust] that uses [ImageMagick] for reading and [SFML] for rendering the images.

- **wiev** is _adequate_. Supported image formats and colorspaces are those of ImageMagick.
- **wiev** is _fast_. All transformations, like panning, rotations, and zooming, are handled by the GPU.
- **wiev** is _minimal_. The GUI consists of only the rendered image and nothing else.
- **wiev** is _simple_. The current codebase is only about 1k SLOC in size and depends only on ImageMagick and SFML.
- **wiev** is _written in Rust_. Code modifications can be made without the fear of breaking something unexpectedly.
- **wiev** is _a work in progress_. Expect rough edges.


## Overview

**wiev** currently _allows_ the following:
- arbitrary pans, rotations, and zoom levels (there are no artificial limits on these; you can pan past the image borders if you want)
- true horizontal and vertical flips (the actual view is flipped; some programs flip only the underlying image)
- on-the-fly image resampling with respect to scale, rotation, or both using ImageMagick ([Lanczos] filter is used for downscaling and [Mitchell] filter for upscaling)
- image state preservation ('you reap what you sow', in a nonnegative sense; currently session bound)
- bookmarks and history (currently session bound)
- texture caching in GPU VRAM (cached images load practically instantaneously, no matter the size)
- textual info overlay (some (less than) meaningful metrics; togglable)

**wiev** currently _does not allow_ the following:
- non-image input (such will cause a panic; likely to be fixed later)
- any kind of mouse control (keyboard is your friend; likely to be added later)
- image saving, modification, or deletion
- explicit fullscreen mode (window managers can provide this)
- image dimensions exceeding GPU texture size limits (newer GPUs/drivers should allow something like 8Ki or 16Ki px for both dimensions; possibly fixed later)

**wiev** currently _is lacking_ in the following:
- error handling (nonexistent)
- unit testing (nonexistent)
- documentation (nonexistent)
- command-line interface (rudimentary)
- configuration (hardcoded)


## Configuration

**wiev** currently has various configuration options hardcoded into the global constants in `main.rs` and `wiev.rs`. This is likely to be fixed later, but in the meantime feel free to change these; note that especially the `FONT_PATH` should be changed to a proper one.

Below are excerpts from the source showing the constants; `main.rs`:
```rust
const IMAGE_HOP:      isize = 10;
const IMAGE_HOP_FAST: isize = 100;

const MOVE:      f64 = 10.;   // in px
const MOVE_FAST: f64 = 100.;  // in px
const ROTATE:      f64 = 1.;   // in deg
const ROTATE_FAST: f64 = 30.;  // in deg
const ZOOM:      f64 = 1.044273782427413840321966478739;  // 2^(1/16)
const ZOOM_FAST: f64 = 1.189207115002721066717499970560;  // 2^(1/4)

// const FONT_PATH: &str = "/usr/share/fonts/liberation/LiberationMono-Regular.ttf";
const FONT_PATH: &str = "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc";
const FONT_SIZE: u32 = 12;
const FONT_FILL_COLOR:    sg::Color = sg::Color::WHITE;
const FONT_OUTLINE_COLOR: sg::Color = sg::Color::BLACK;
const FONT_OUTLINE_THICKNESS: f32 = 2.;
```
`wiev.rs`:
```rust
const BACKGROUND_COLOR: sg::Color = sg::Color::BLACK;

const TEXTURE_CACHE_MAX_BYTE_SIZE: usize = 1 << 29;  // 512 MiB

const COLORSPACE: mb::ColorspaceType = mb::ColorspaceType_sRGBColorspace;
const DOWNSCALE_FILTER: m::FilterType = mb::FilterType_LanczosFilter;
const UPSCALE_FILTER:   m::FilterType = mb::FilterType_MitchellFilter;
```


## Keybindings

**wiev** currently has keybindings hardcoded into the `handle_event` function in `main.rs`. This is likely to be fixed later, but in the meantime feel free to change these; the key names correspond to the [sfml::window::Key] enum variants.

Below is an excerpt from the source showing the keybindings:
```rust
key!( Escape ) | sw::Event::Closed => { wiever.close( window ); },

key!( PageUp )         => { wiever.change_image_index( wIO( -1 ) ); },
key!( PageDown )       => { wiever.change_image_index( wIO(  1 ) ); },
key!( PageUp,   c )    => { wiever.change_image_index( wIO( -IH ) ); },
key!( PageDown, c )    => { wiever.change_image_index( wIO(  IH ) ); },
key!( PageUp,   s )    => { wiever.change_image_index( wIO( -IHF ) ); },
key!( PageDown, s )    => { wiever.change_image_index( wIO(  IHF ) ); },
key!( PageUp,   c, s ) => { wiever.change_image_index( wIS(  0 ) ); },
key!( PageDown, c, s ) => { wiever.change_image_index( wIS( -1 ) ); },
key!( PageDown, a )    => { wiever.change_image_index( wIR ); },

key!( Home )       => { wiever.change_bookmark_index( wIO( -1 ) ); },
key!( End )        => { wiever.change_bookmark_index( wIO(  1 ) ); },
key!( Home, c )    => { wiever.change_bookmark_index( wIO( -IH ) ); },
key!( End,  c )    => { wiever.change_bookmark_index( wIO(  IH ) ); },
key!( Home, s )    => { wiever.change_bookmark_index( wIO( -IHF ) ); },
key!( End,  s )    => { wiever.change_bookmark_index( wIO(  IHF ) ); },
key!( Home, c, s ) => { wiever.change_bookmark_index( wIS(  0 ) ); },
key!( End,  c, s ) => { wiever.change_bookmark_index( wIS( -1 ) ); },
key!( End,  a )    => { wiever.change_bookmark_index( wIR ); },

key!( Insert )       => { wiever.change_history_index( wIO( -1 ) ); },
key!( Delete )       => { wiever.change_history_index( wIO(  1 ) ); },
key!( Insert, c )    => { wiever.change_history_index( wIO( -IH ) ); },
key!( Delete, c )    => { wiever.change_history_index( wIO(  IH ) ); },
key!( Insert, s )    => { wiever.change_history_index( wIO( -IHF ) ); },
key!( Delete, s )    => { wiever.change_history_index( wIO(  IHF ) ); },
key!( Insert, c, s ) => { wiever.change_history_index( wIS(  0 ) ); },
key!( Delete, c, s ) => { wiever.change_history_index( wIS( -1 ) ); },
key!( Delete, a )    => { wiever.change_history_index( wIR ); },

key!( BackSpace )    => { wiever.add_bookmark(); },
key!( BackSpace, c ) => { wiever.remove_bookmark(); },

key!( Numpad0 ) => { wiever.resample_image( wSN, wRN ); },
key!( Numpad1 ) => { wiever.resample_image( wSC, wRC ); },
key!( Numpad2 ) => { wiever.resample_image( wSC, wRN ); },
key!( Numpad3 ) => { wiever.resample_image( wSN, wRC ); },

key!( S )    => { wiever.toggle_texture_smooth(); },
key!( S, c ) => { wiever.toggle_texture_mipmap(); },
// key!( S, s ) => { wiever.toggle_texture_srgb(); },
key!( T )    => { wiever.toggle_text_visible(); },
key!( C )    => { wiever.toggle_cursor_visible(); },

key!( Return )    => { wiever.default_(); },
key!( Return, c ) => { wiever.fit_min_dim(); },
key!( Return, s ) => { wiever.fit_max_dim(); },

key!( Space )       => { wiever.reset_zoom(); },
key!( Space, c )    => { wiever.reset_rotation(); },
key!( Space, s )    => { wiever.center(); },
key!( Space, c, s ) => { wiever.reset_flip(); },

key!( Up )       => { wiever.move_( ( 0., -MF ), true, true ); },
key!( Down )     => { wiever.move_( ( 0.,  MF ), true, true ); },
key!( Left )     => { wiever.move_( ( -MF, 0. ), true, true ); },
key!( Right )    => { wiever.move_( (  MF, 0. ), true, true ); },
key!( Up,    s ) => { wiever.move_( ( 0., -M ), true, true ); },
key!( Down,  s ) => { wiever.move_( ( 0.,  M ), true, true ); },
key!( Left,  s ) => { wiever.move_( ( -M, 0. ), true, true ); },
key!( Right, s ) => { wiever.move_( (  M, 0. ), true, true ); },

key!( Up,   c )    => { wiever.zoom( ZF ); },
key!( Down, c )    => { wiever.zoom( ZF.recip() ); },
key!( Up,   c, s ) => { wiever.zoom( Z ); },
key!( Down, c, s ) => { wiever.zoom( Z.recip() ); },

key!( Left,  c )    => { wiever.rotate( -RF ); },
key!( Right, c )    => { wiever.rotate(  RF ); },
key!( Left,  c, s ) => { wiever.rotate( -R ); },
key!( Right, c, s ) => { wiever.rotate(  R ); },

key!( Dash )    => { wiever.hflip(); },
key!( Dash, s ) => { wiever.vflip(); },
```


## Installation

Install [nightly Rust] and compile with:
- `cargo build --release`

The compiled executable will be in `wiev/target/release/wiev`.

The only library dependencies are ImageMagick and SFML, so these should be installed as well.


## Usage

- `wiev <start_index> <image_path>+`

For example:
- `wiev 0 a.jpg b.webp c.png  # starts at a.jpg`
- `wiev -1 a.jpg b.webp c.png  # starts at c.png`
- `wiev -42 a.jpg b.webp c.png  # starts at whatever the modulo is`


[Rust]: https://www.rust-lang.org/
[nightly Rust]: https://doc.rust-lang.org/nightly/edition-guide/rust-2018/rustup-for-managing-rust-versions.html

[ImageMagick]: https://imagemagick.org/index.php
[Lanczos]: https://www.imagemagick.org/Usage/filter/#lanczos
[Mitchell]: https://www.imagemagick.org/Usage/filter/#mitchell

[SFML]: https://www.sfml-dev.org/
[sfml::window::Key]: https://docs.rs/sfml/0.14.0/sfml/window/enum.Key.html
