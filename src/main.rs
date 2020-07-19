// main



// ATTRIBUTES

// #![ feature( euclidean_division ) ]



// IMPORTS  // FIX?: remove

extern crate magick_rust;
// extern crate rand;
extern crate sfml;



// MODULES

mod wiev;



// ALIASES

use magick_rust as mr;

use sfml::graphics as sg;
use sfml::system as ss;
use sfml::window as sw;
use sfml::window::Event::KeyPressed as KP;
use sfml::window::Key as K;

use std::str::FromStr;

use wiev as w;



// CONSTANTS

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

const REFRESH_FLAG_COUNT: i32 = 100;  // in frames
const IDLE_SLEEP_TIME: i32 = 100;  // in msec



// MACROS

macro_rules! key
{
    ( $code:ident ) => (
        KP{ code: K::$code, alt: false, ctrl: false, shift: false, system: false }
    );
    ( $code:ident, a ) => (
        KP{ code: K::$code, alt: true,  ctrl: false, shift: false, system: false }
    );
    ( $code:ident, c ) => (
        KP{ code: K::$code, alt: false, ctrl: true,  shift: false, system: false }
    );
    ( $code:ident, s ) => (
        KP{ code: K::$code, alt: false, ctrl: false, shift: true,  system: false }
    );
    ( $code:ident, a, c ) => (
        KP{ code: K::$code, alt: true,  ctrl: true,  shift: false, system: false }
    );
    ( $code:ident, a, s ) => (
        KP{ code: K::$code, alt: true,  ctrl: false, shift: true,  system: false }
    );
    ( $code:ident, c, s ) => (
        KP{ code: K::$code, alt: false, ctrl: true,  shift: true,  system: false }
    );
    ( $code:ident, a, c, s ) => (
        KP{ code: K::$code, alt: true,  ctrl: true,  shift: true,  system: false }
    );
}



// FUNCTIONS

fn handle_event(
    event: sw::Event,
    window: &mut sg::RenderWindow,
    wiever: &mut w::Wiever,
    refresh_flag_count: &mut i32,
)
{
    use w::Index::Set as wIS;
    use w::Index::Offset as wIO;
    use w::Index::Random as wIR;
    use w::Scale::None as wSN;
    // use w::Scale::Set as wSS;
    use w::Scale::Current as wSC;
    use w::Rotation::None as wRN;
    // use w::Rotation::Set as wRS;
    use w::Rotation::Current as wRC;

    use IMAGE_HOP as IH;
    use IMAGE_HOP_FAST as IHF;
    use MOVE as M;
    use MOVE_FAST as MF;
    use ZOOM as Z;
    use ZOOM_FAST as ZF;
    use ROTATE as R;
    use ROTATE_FAST as RF;

    match event {
        key!( Escape, a ) | sw::Event::Closed => { wiever.close( window ); },

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
        key!( C, c ) => { wiever.clear_texture_cache(); },

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

        sw::Event::Resized{ .. } => {},

        _ => { return; },
    }

    *refresh_flag_count = REFRESH_FLAG_COUNT;
}



// MAIN

fn main()
{
    mr::magick_wand_genesis();

    let mut window = sg::RenderWindow::new(
        sw::VideoMode::desktop_mode(),
        "wiev",
        sw::Style::DEFAULT,
        &sw::ContextSettings{
            antialiasing_level: 8,
            // srgb_capable: ss::TRUE,
            ..Default::default()
        },
    );
    window.set_vertical_sync_enabled( true );

    let mut arg_iter = std::env::args().skip( 1 );
    let image_index = isize::from_str( &arg_iter.next().unwrap() ).unwrap();
    let image_path_vec = arg_iter.collect();
    let mut wiever = w::Wiever::new( &window, image_index, image_path_vec );

    let font = sg::Font::from_file( FONT_PATH ).unwrap();
    let mut text = sg::Text::new( "", &font, FONT_SIZE );
    text.set_fill_color( &FONT_FILL_COLOR );
    text.set_outline_color( &FONT_OUTLINE_COLOR );
    text.set_outline_thickness( FONT_OUTLINE_THICKNESS );

    while window.is_open() {
        let texture = wiever.create_texture().unwrap();
        let sprite = sg::Sprite::with_texture( &texture );

        let mut refresh_flag_count = REFRESH_FLAG_COUNT;
        while !wiever.reload_required_flag {
            if refresh_flag_count > 0 {
                refresh_flag_count -= 1;
                wiever.display( &mut window, &sprite, &mut text );
            }
            else { ss::sleep( ss::Time::milliseconds( IDLE_SLEEP_TIME ) ); }

            while let Some( event_ ) = window.poll_event() {
                handle_event(
                    event_,
                    &mut window,
                    &mut wiever,
                    &mut refresh_flag_count,
                );
            }
        }

        drop( sprite );
        wiever.cache_texture( texture );
    }

    mr::magick_wand_terminus();
}
