// wiev



// ATTRIBUTES

// #![ allow( dead_code ) ]



// IMPORTS  // FIX?: remove

extern crate magick_rust;
extern crate rand;
extern crate sfml;



// ALIASES

use magick_rust as m;
use magick_rust::bindings as mb;

use rand as r;

use sfml::graphics as sg;
use sfml::graphics::RenderTarget;
use sfml::graphics::Transformable;
use sfml::system as ss;

use std::collections as sc;



// CONSTANTS

const BACKGROUND_COLOR: sg::Color = sg::Color::BLACK;

const TEXTURE_CACHE_MAX_BYTE_SIZE: usize = 1 << 29;  // 512 MiB

const COLORSPACE: mb::ColorspaceType = mb::ColorspaceType_sRGBColorspace;
const DOWNSCALE_FILTER: m::FilterType = mb::FilterType_LanczosFilter;
const UPSCALE_FILTER:   m::FilterType = mb::FilterType_MitchellFilter;



// MACROS

macro_rules! expr_str_ln
{
    ( $expr:expr ) => (
        format!( "`{}`: {:?}\n", stringify!( $expr ), $expr )
    );
}

macro_rules! res
{
    ( $res:expr ) => (
        match $res {
            Ok( value_ ) => { value_ },
            Err( err_ ) => {
                return Err( format!(
                        "`{}` failed: {}", stringify!( $res ), err_ ) );
            },
        }
    );
}

macro_rules! opt
{
    ( $opt:expr ) => (
        match $opt {
            Some( value_ ) => { value_ },
            None => {
                return Err( format!( "`{}` failed.", stringify!( $opt ) ) );
            },
        }
    );
}



// AUXILIARY TYPES

#[ derive( Debug, Clone, Copy ) ]
pub enum Index
{
    Set( isize ),
    Offset( isize ),
    Random,
}

#[ derive( Debug, Clone, Copy ) ]
pub enum Scale
{
    None,
    Set( f64 ),
    Current,
}

#[ derive( Debug, Clone, Copy ) ]
pub enum Rotation
{
    None,
    Set( f64 ),
    Current,
}

#[ derive( Debug, Clone, Copy ) ]
struct ImageState
{
    scale: Scale,
    rotation: Rotation,
    wiev: Wiev,
}

type Result< T > = std::result::Result< T, String >;



// TYPES

#[ derive( Debug ) ]
pub struct Wiever
{
    window_size: ( u32, u32 ),  // ( width, height )

    image_index: usize,
    image_path_vec: Vec< String >,
    image_state_map: sc::HashMap< usize, ImageState >,

    bookmark_index: usize,
    bookmark_que: sc::VecDeque< usize >,

    history_index: usize,
    history_que: sc::VecDeque< usize >,
    history_mode_flag: bool,

    texture_cache_key: usize,
    texture_cache_map: sc::HashMap< usize, sg::Texture >,
    texture_cache_que: sc::VecDeque< usize >,
    texture_cache_bypass_flag: bool,

    text_str: String,

    texture_smooth_flag: bool,
    texture_mipmap_flag: bool,
    texture_srgb_flag: bool,
    text_visible_flag: bool,
    cursor_visible_flag: bool,

    pub reload_required_flag: bool,  // FIX?: rethink
}

#[ derive( Debug, Clone, Copy ) ]
pub struct Wiev
{
    window_size: ( u32, u32 ),  // ( width, height )
    image_size: ( u32, u32 ),  // ( width, height )

    image_scale: f64,
    image_rotation: f64,

    view_center: ( f64, f64 ),  // ( x, y )
    view_zoom: f64,
    view_rotation: f64,
    flip_flags: ( bool, bool ),  // ( hflip, vflip )
}



// TYPE IMPLEMENTATIONS

impl Wiever
{
// CONSTRUCTORS
    pub fn new(
        window: &sg::RenderWindow,
        image_index: isize,
        image_path_vec: Vec< String >,
    ) -> Self
    {
        let window_size = vector2_to_tuple( window.size() );
        let image_count = image_path_vec.len() as isize;
        let image_index = image_index.rem_euclid( image_count ) as usize;
        let self_ = Self{
            window_size,
            image_index,
            image_path_vec,
            image_state_map: Default::default(),
            bookmark_index: 0,
            bookmark_que: Default::default(),
            history_index: 0,
            history_que: Default::default(),
            history_mode_flag: false,
            texture_cache_key: 0,
            texture_cache_map: Default::default(),
            texture_cache_que: Default::default(),
            texture_cache_bypass_flag: false,
            text_str: Default::default(),
            texture_smooth_flag: true,
            texture_mipmap_flag: false,
            texture_srgb_flag: false,
            text_visible_flag: false,
            cursor_visible_flag: false,
            reload_required_flag: false,
        };

        return self_;
    }

// PUBLIC METHODS
    pub fn display( &mut self,
        window: &mut sg::RenderWindow,
        sprite: &sg::Sprite,
        text: &mut sg::Text,
    )
    {
        assert!( window.set_active( true ) );

        window.set_title( &self.get_window_title() );
        window.set_mouse_cursor_visible( self.cursor_visible_flag );

        self.window_size = vector2_to_tuple( window.size() );

        let window_size = self.window_size;
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.update_window_size( window_size );
        image_state.wiev.draw_sprite( window, sprite );

        if self.text_visible_flag {
            self.update_text_str();
            text.set_string( &self.text_str );
            let image_state = self.get_current_image_state_ref();
            image_state.wiev.draw_text( window, text );
        }

        window.display();
    }

    pub fn close( &mut self, window: &mut sg::RenderWindow )
    {
        window.close();

        self.reload_required_flag = true;
    }

    pub fn create_texture( &mut self ) -> Result< sg::Texture >
    {
        if !self.image_state_map.contains_key( &self.image_index ) {
            let path = self.image_path_vec.get( self.image_index ).unwrap();
            let scale = Scale::None;
            let rotation = Rotation::None;
            let size = res!( Self::read_image_size( path, scale, rotation ) );
            let size = ( size.0 as u32, size.1 as u32 );
            let wiev = Wiev::new( self.window_size, size );
            let image_state = ImageState{ scale, rotation, wiev };
            self.image_state_map.insert( self.image_index, image_state );
        }

        let texture_opt = self.texture_cache_map.remove( &self.image_index );
        let mut texture =
                if !self.texture_cache_bypass_flag && texture_opt.is_some() {
                    opt!( texture_opt )
                }
                else {
                    drop( texture_opt );
                    let image = res!( self.read_current_image() );
                    opt!( sg::Texture::from_image( &image ) )
                };
        if self.texture_mipmap_flag { assert!( texture.generate_mipmap() ); }
        texture.set_srgb( self.texture_srgb_flag );
        texture.set_smooth( self.texture_smooth_flag );

        let image_state = self.get_current_image_state_mut();
        let image_size = vector2_to_tuple( texture.size() );
        image_state.wiev.update_image_size( image_size );
        match image_state.scale {
            Scale::None => {
                image_state.wiev.update_image_scale( 1. );
            },
            Scale::Set( scale_ ) => {
                image_state.wiev.update_image_scale( scale_ );
            },
            _ => { panic!(); },
        };
        match image_state.rotation {
            Rotation::None => {
                image_state.wiev.update_image_rotation( 0. );
            },
            Rotation::Set( rotation_ ) => {
                image_state.wiev.update_image_rotation( rotation_ );
            },
            _ => { panic!(); },
        };

        self.texture_cache_key = self.image_index;
        self.texture_cache_bypass_flag = false;
        self.reload_required_flag = false;

        return Ok( texture );
    }

    pub fn cache_texture( &mut self, texture: sg::Texture )
    {
        if self.texture_cache_bypass_flag { return; }

        Self::add_to_que( &mut self.texture_cache_que, self.texture_cache_key );
        self.texture_cache_map.insert( self.texture_cache_key, texture );

        while self.get_current_texture_cache_byte_size()
                > TEXTURE_CACHE_MAX_BYTE_SIZE {
            if let Some( key_ ) = self.texture_cache_que.pop_back() {
                self.texture_cache_map.remove( &key_ );
            }
        }
    }

    pub fn change_image_index( &mut self, index: Index )
    {
        if self.image_path_vec.len() == 0 { return; }  // FIX?: remove

        if !self.history_mode_flag {
            Self::add_to_que( &mut self.history_que, self.image_index );
            self.history_index = 0;
        }
        self.history_mode_flag = false;

        let index = match index {
            Index::Set( index_ )     => { index_ },
            Index::Offset( offset_ ) => { offset_ + self.image_index as isize },
            Index::Random            => { r::random() },
        };
        let image_count = self.image_path_vec.len() as isize;
        self.image_index = index.rem_euclid( image_count ) as usize;

        self.reload_required_flag = true;
    }

    pub fn change_bookmark_index( &mut self, index: Index )
    {
        if self.bookmark_que.len() == 0 { return; }

        if !self.history_mode_flag {
            Self::add_to_que( &mut self.history_que, self.image_index );
            self.history_index = 0;
        }
        self.history_mode_flag = false;

        let index = match index {
            Index::Set( index_ )     => { index_ },
            Index::Offset( offset_ ) => { offset_ + self.bookmark_index as isize },
            Index::Random            => { r::random() },
        };
        let image_count = self.bookmark_que.len() as isize;
        self.bookmark_index = index.rem_euclid( image_count ) as usize;
        self.image_index = *self.bookmark_que.get( self.bookmark_index ).unwrap();

        self.reload_required_flag = true;
    }

    pub fn change_history_index( &mut self, index: Index )
    {
        if self.history_que.len() == 0 { return; }

        if !self.history_mode_flag {
            Self::add_to_que( &mut self.history_que, self.image_index );
            self.history_index = 0;
        }
        self.history_mode_flag = true;

        let index = match index {
            Index::Set( index_ )     => { index_ },
            Index::Offset( offset_ ) => { offset_ + self.history_index as isize },
            Index::Random            => { r::random() },
        };
        let image_count = self.history_que.len() as isize;
        self.history_index = index.rem_euclid( image_count ) as usize;
        self.image_index = *self.history_que.get( self.history_index ).unwrap();

        self.reload_required_flag = true;
    }

    pub fn add_bookmark( &mut self )
    {
        Self::add_to_que( &mut self.bookmark_que, self.image_index );
        self.bookmark_index = 0;
    }

    pub fn remove_bookmark( &mut self )
    {
        Self::remove_from_que( &mut self.bookmark_que, self.image_index );
        self.bookmark_index = 0;
    }

    pub fn resample_image( &mut self, scale: Scale, rotation: Rotation )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.scale =
                if let Scale::Current = scale {
                    let scale = match image_state.scale {
                        Scale::None          => { 1. },
                        Scale::Set( scale_ ) => { scale_ },
                        _ => { panic!(); },
                    };
                    Scale::Set( scale * image_state.wiev.view_zoom )
                }
                else { scale };
        image_state.rotation =
                if let Rotation::Current = rotation {
                    let rotation = match image_state.rotation {
                        Rotation::None             => { 0. },
                        Rotation::Set( rotation_ ) => { rotation_ },
                        _ => { panic!(); },
                    };
                    let wiev = image_state.wiev;
                    let view_rotation =
                            if wiev.flip_flags.0 ^ wiev.flip_flags.1 {
                                -wiev.view_rotation
                            }
                            else { wiev.view_rotation };
                    Rotation::Set( rotation + view_rotation )
                }
                else { rotation };

        self.texture_cache_bypass_flag = true;
        self.reload_required_flag = true;
    }

    pub fn toggle_texture_smooth( &mut self )
    {
        self.texture_smooth_flag = !self.texture_smooth_flag;

        self.reload_required_flag = true;
    }

    pub fn toggle_texture_mipmap( &mut self )
    {
        self.texture_mipmap_flag = !self.texture_mipmap_flag;

        self.texture_cache_bypass_flag = true;
        self.reload_required_flag = true;
    }

    #[ allow( dead_code ) ]
    pub fn toggle_texture_srgb( &mut self )
    {
        self.texture_srgb_flag = !self.texture_srgb_flag;

        self.texture_cache_bypass_flag = true;
        self.reload_required_flag = true;
    }

    pub fn toggle_text_visible( &mut self )
    {
        self.text_visible_flag = !self.text_visible_flag;
    }

    pub fn toggle_cursor_visible( &mut self )
    {
        self.cursor_visible_flag = !self.cursor_visible_flag;
    }

// PUBLIC WIEV METHODS
    pub fn default_( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.default_();
    }

    pub fn move_( &mut self,
        ( x, y ): ( f64, f64 ),
        rotate_flag: bool,
        scale_flag: bool,
    )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.move_( ( x, y ), rotate_flag, scale_flag );
    }

    pub fn center( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.center();
    }

    pub fn fit_max_dim( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.fit_max_dim();
    }

    pub fn fit_min_dim( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.fit_min_dim();
    }

    pub fn zoom( &mut self, factor: f64 )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.zoom( factor );
    }

    pub fn reset_zoom( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.reset_zoom();
    }

    pub fn rotate( &mut self, angle: f64 )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.rotate( angle );
    }

    pub fn reset_rotation( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.reset_rotation();
    }

    pub fn hflip( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.hflip();
    }

    pub fn vflip( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.vflip();
    }

    pub fn reset_flip( &mut self )
    {
        let image_state = self.get_current_image_state_mut();
        image_state.wiev.reset_flip();
    }

// PRIVATE METHODS
    fn get_window_title( &self ) -> String
    {
        let index = self.image_index;
        let total = self.image_path_vec.len();
        let path = self.image_path_vec.get( self.image_index ).unwrap();
        let title = format!( "wiev  ::  {} / {}  ::  {}", index, total, path );

        return title;
    }

    fn update_text_str( &mut self )
    {
        self.text_str.clear();

        // let window_size = self.window_size;
        let image_index = self.image_index;
        let image_total = self.image_path_vec.len();
        let image_path = self.image_path_vec.get( self.image_index ).unwrap();
        let bookmark_index = self.bookmark_index;
        let bookmark_que = &self.bookmark_que;
        let history_index = self.history_index;
        let history_que = &self.history_que;
        // let history_mode_flag = self.history_mode_flag;
        // let texture_cache_key = self.texture_cache_key;
        // let texture_cache_que = &self.texture_cache_que;
        let texture_cache_size = self.get_current_texture_cache_byte_size();
        let texture_smooth_flag = self.texture_smooth_flag;
        let texture_mipmap_flag = self.texture_mipmap_flag;
        // let texture_srgb_flag = self.texture_srgb_flag;
        // let text_visible_flag = self.text_visible_flag;
        // let cursor_visible_flag = self.cursor_visible_flag;
        let image_state = self.get_current_image_state_ref();
        let image_scale    = image_state.scale;
        let image_rotation = image_state.rotation;
        let image_size    = image_state.wiev.image_size;
        let window_size   = image_state.wiev.window_size;
        let view_size     = image_state.wiev.get_view_size();
        let view_center   = image_state.wiev.view_center;
        let view_zoom     = image_state.wiev.view_zoom;
        let view_rotation = image_state.wiev.view_rotation;
        let flip_flags    = image_state.wiev.flip_flags;

        // self.text_str.push_str( &expr_str_ln!( window_size ) );
        self.text_str.push_str( &expr_str_ln!( image_index ) );
        self.text_str.push_str( &expr_str_ln!( image_total ) );
        self.text_str.push_str( &expr_str_ln!( image_path ) );
        self.text_str.push_str( &expr_str_ln!( bookmark_index ) );
        self.text_str.push_str( &expr_str_ln!( bookmark_que ) );
        self.text_str.push_str( &expr_str_ln!( history_index ) );
        self.text_str.push_str( &expr_str_ln!( history_que ) );
        // self.text_str.push_str( &expr_str_ln!( history_mode_flag ) );
        // self.text_str.push_str( &expr_str_ln!( texture_cache_key ) );
        // self.text_str.push_str( &expr_str_ln!( texture_cache_que ) );
        self.text_str.push_str( &expr_str_ln!( texture_cache_size ) );
        self.text_str.push_str( &expr_str_ln!( texture_smooth_flag ) );
        self.text_str.push_str( &expr_str_ln!( texture_mipmap_flag ) );
        // self.text_str.push_str( &expr_str_ln!( texture_srgb_flag ) );
        // self.text_str.push_str( &expr_str_ln!( text_visible_flag ) );
        // self.text_str.push_str( &expr_str_ln!( cursor_visible_flag ) );
        self.text_str.push_str( &expr_str_ln!( image_scale ) );
        self.text_str.push_str( &expr_str_ln!( image_rotation ) );
        self.text_str.push_str( &expr_str_ln!( image_size ) );
        self.text_str.push_str( &expr_str_ln!( window_size ) );
        self.text_str.push_str( &expr_str_ln!( view_size ) );
        self.text_str.push_str( &expr_str_ln!( view_center ) );
        self.text_str.push_str( &expr_str_ln!( view_zoom ) );
        self.text_str.push_str( &expr_str_ln!( view_rotation ) );
        self.text_str.push_str( &expr_str_ln!( flip_flags ) );
    }

    fn get_current_image_state_ref( &self ) -> &ImageState
    {
        let image_state =
                self.image_state_map.get( &self.image_index ).unwrap();

        return image_state;
    }

    fn get_current_image_state_mut( &mut self ) -> &mut ImageState
    {
        let image_state =
                self.image_state_map.get_mut( &self.image_index ).unwrap();

        return image_state;
    }

    fn get_current_texture_cache_byte_size( &self ) -> usize
    {
        let mut byte_size = 0;
        for ( _, value_ ) in &self.texture_cache_map {
            byte_size += Self::get_texture_byte_size( value_ );
        }

        return byte_size;
    }

    #[ allow( dead_code ) ]
    fn read_current_image_size( &self ) -> Result< ( usize, usize ) >
    {
        let image_path = self.image_path_vec.get( self.image_index ).unwrap();
        let image_state = self.get_current_image_state_ref();
        let image_size_res = Self::read_image_size(
                image_path, image_state.scale, image_state.rotation );

        return image_size_res;
    }

    fn read_current_image( &self ) -> Result< sg::Image >
    {
        let image_path = self.image_path_vec.get( self.image_index ).unwrap();
        let image_state = self.get_current_image_state_ref();
        let image_res = Self::read_image(
                image_path, image_state.scale, image_state.rotation );

        return image_res;
    }

// PRIVATE STATIC METHODS
    fn get_texture_byte_size( texture: &sg::Texture ) -> usize
    {
        let size = vector2_to_tuple( texture.size() );
        let size = ( size.0 as usize, size.1 as usize );
        let byte_size = size.0 * size.1 * 4;  // assume 4-byte pixels

        return byte_size;
    }

    fn add_to_que< T >( que: &mut sc::VecDeque< T >, value: T )
    where T: Copy + Eq
    {
        que.retain( | &value_ | { value_ != value } );
        que.push_front( value );
    }

    fn remove_from_que< T >( que: &mut sc::VecDeque< T >, value: T )
    where T: Copy + Eq
    {
        que.retain( | &value_ | { value_ != value } );
    }

    // When rotation is a multiple of 90, the returned size is exact.
    // When not, the returned size is only an approximation of the real size,
    // and the error would seem to change quite randomly between -1 and 1
    // for both dimensions and depending on the rotation.
    fn read_image_size( path: &str, scale: Scale, rotation: Rotation )
            -> Result< ( usize, usize ) >
    {
        let wand = m::MagickWand::new();
        res!( wand.ping_image( path ) );

        let mut image_width  = wand.get_image_width();
        let mut image_height = wand.get_image_height();
        match scale {
            Scale::None => {},
            Scale::Set( scale_ ) => {
                image_width  = (scale_ * image_width  as f64).round() as usize;
                image_height = (scale_ * image_height as f64).round() as usize;
            },
            _ => { panic!(); },
        }
        match rotation {
            Rotation::None => {},
            Rotation::Set( rotation_ ) => {
                let size = ( image_width as f64, image_height as f64 );
                let size = get_bounding_box( size, -rotation_ );
                image_width  = size.0.round() as usize;
                image_height = size.1.round() as usize;
                if rotation_.rem_euclid( 90. ) >= 1e-6 {
                    // reduce consistent offset
                    image_width  += 2;
                    image_height += 2;
                }
            },
            _ => { panic!(); },
        }
        let image_size = ( image_width, image_height );

        return Ok( image_size );
    }

    fn read_image( path: &str, scale: Scale, rotation: Rotation )
            -> Result< sg::Image >
    {
        let wand = m::MagickWand::new();
        res!( wand.read_image( path ) );
        res!( wand.transform_image_colorspace( COLORSPACE ) );

        match scale {
            Scale::None => {},
            Scale::Set( scale_ ) => {
                let mut image_width  = wand.get_image_width();
                let mut image_height = wand.get_image_height();
                image_width  = (scale_ * image_width  as f64).round() as usize;
                image_height = (scale_ * image_height as f64).round() as usize;
                let filter_type = if scale_ <= 1. { DOWNSCALE_FILTER }
                                  else            { UPSCALE_FILTER };
                wand.resize_image( image_width, image_height, filter_type );
            },
            _ => { panic!(); },
        }

        match rotation {
            Rotation::None => {},
            Rotation::Set( rotation_ ) => {
                let mut bg_color = m::PixelWand::new();
                res!( bg_color.set_color( "rgba(0,0,0,0)" ) );
                res!( wand.rotate_image( &bg_color, -rotation_ ) );
            },
            _ => { panic!(); },
        }

        let image_width  = wand.get_image_width();
        let image_height = wand.get_image_height();
        let pixel_vec = opt!( wand.export_image_pixels(
                0, 0, image_width, image_height, "rgba" ) );
        drop( wand );
        let image = opt!( sg::Image::create_from_pixels(
                image_width as u32, image_height as u32, &pixel_vec ) );

        return Ok( image );
    }
}

impl Wiev
{
// CONSTRUCTORS
    pub fn new( window_size: ( u32, u32 ), image_size: ( u32, u32 ) ) -> Self
    {
        let mut self_ = Self{
            window_size,
            image_size,
            image_scale: 1.,
            image_rotation: 0.,
            view_center: ( 0., 0. ),
            view_zoom: 1.,
            view_rotation: 0.,
            flip_flags: ( false, false ),
        };
        Self::default_( &mut self_ );

        return self_;
    }

// PUBLIC METHODS
    pub fn draw_sprite( &self,
        window: &mut sg::RenderWindow,
        sprite: &sg::Sprite,
    )
    {
        window.clear( &BACKGROUND_COLOR );

        let view = self.create_view();
        window.set_view( &view );

        let render_states = sg::RenderStates{
            transform: self.create_transform(),
            ..Default::default()
        };
        window.draw_with_renderstates( sprite, render_states );
    }

    pub fn draw_text( &self,
        window: &mut sg::RenderWindow,
        text: &mut sg::Text,
    )
    {
        let size = self.get_view_size();
        let x = -size.0 / 2.;
        let y = -size.1 / 2.;
        let ( mut x, mut y ) = rotate_xy( ( x, y ), self.view_rotation );
        x += self.view_center.0;
        y += self.view_center.1;
        let position = ( x as f32, y as f32 );
        text.set_position( position );

        let scale = self.view_zoom.recip() as f32;
        text.set_scale( ( scale, scale ) );

        let rotation = self.view_rotation as f32;
        text.set_rotation( rotation );

        let render_states = Default::default();
        window.draw_with_renderstates( text, render_states );
    }

    pub fn default_( &mut self )
    {
        self.center();
        self.fit_max_dim();
        self.reset_rotation();
        self.reset_flip();
    }

    pub fn move_( &mut self,
        ( x, y ): ( f64, f64 ),
        rotate_flag: bool,
        scale_flag: bool,
    )
    {
        let ( mut x, mut y ) =
                if rotate_flag { rotate_xy( ( x, y ), self.view_rotation ) }
                else           { ( x, y ) };

        if scale_flag { x /= self.view_zoom;  y /= self.view_zoom; }

        self.view_center.0 += x;
        self.view_center.1 += y;
    }

    pub fn center( &mut self )
    {
        self.center_view();
    }

    pub fn fit_max_dim( &mut self )
    {
        self.fit_view( false );
    }

    pub fn fit_min_dim( &mut self )
    {
        self.fit_view( true );
    }

    pub fn zoom( &mut self, factor: f64 )
    {
        self.view_zoom *= factor;
    }

    pub fn reset_zoom( &mut self )
    {
        self.view_zoom = 1.;
    }

    pub fn rotate( &mut self, angle: f64 )
    {
        self.view_rotation += angle;
    }

    pub fn reset_rotation( &mut self )
    {
        self.view_rotation = 0.;
    }

    pub fn hflip( &mut self )
    {
        self.flip( ( true, false ) );
    }

    pub fn vflip( &mut self )
    {
        self.flip( ( false, true ) );
    }

    pub fn reset_flip( &mut self )
    {
        self.flip( self.flip_flags );
    }

// PRIVATE METHODS
    fn create_view( &self ) -> sg::View
    {
        let center = ( self.view_center.0 as f32, self.view_center.1 as f32 );
        let size = self.get_view_size();
        let size = ( size.0 as f32, size.1 as f32 );
        let mut view = sg::View::new( center.into(), size.into() );
        view.set_rotation( self.view_rotation as f32 );

        return view;
    }

    fn create_transform( &self ) -> sg::Transform
    {
        let mut transform = sg::Transform::IDENTITY;

        if self.flip_flags.0 {
            let mut hflip_transform = sg::Transform::new( -1., 0., 0.,
                                                           0., 1., 0.,
                                                           0., 0., 1. );
            transform.combine( &mut hflip_transform );
        }

        if self.flip_flags.1 {
            let mut vflip_transform = sg::Transform::new( 1.,  0., 0.,
                                                          0., -1., 0.,
                                                          0.,  0., 1. );
            transform.combine( &mut vflip_transform );
        }

        let x = -f64::from( self.image_size.0 ) / 2.;
        let y = -f64::from( self.image_size.1 ) / 2.;
        transform.translate( x as f32, y as f32 );

        return transform;
    }

    fn update_window_size( &mut self, window_size: ( u32, u32 ) )
    {
        self.window_size = window_size;
    }

    fn update_image_size( &mut self, image_size: ( u32, u32 ) )
    {
        self.image_size = image_size;
    }

    fn update_image_scale( &mut self, image_scale: f64 )
    {
        let scale_change = image_scale / self.image_scale;
        let x = scale_change * self.view_center.0;
        let y = scale_change * self.view_center.1;
        self.view_center = ( x, y );
        self.view_zoom /= scale_change;

        self.image_scale = image_scale;
    }

    fn update_image_rotation( &mut self, image_rotation: f64 )
    {
        let mut rotation_change = image_rotation - self.image_rotation;
        if self.flip_flags.0 ^ self.flip_flags.1 {
            rotation_change = -rotation_change;
        }
        self.view_center = rotate_xy( self.view_center, -rotation_change );
        self.view_rotation -= rotation_change;

        self.image_rotation = image_rotation;
    }

    fn center_view( &mut self )
    {
        self.view_center = ( 0., 0. );
    }

    fn fit_view( &mut self, min_dim_flag: bool )
    {
        let image_aspect_ratio = get_aspect_ratio( self.image_size );
        let window_aspect_ratio = get_aspect_ratio( self.window_size );
        let flag = (image_aspect_ratio > window_aspect_ratio) ^ min_dim_flag;
        if flag { self.fit_view_width(); }
        else    { self.fit_view_height(); }
    }

    fn fit_view_width( &mut self )
    {
        self.set_view_width( self.image_size.0.into() );
    }

    fn fit_view_height( &mut self )
    {
        self.set_view_height( self.image_size.1.into() );
    }

    fn set_view_width( &mut self, view_width: f64 )
    {
        self.view_zoom = f64::from( self.window_size.0 ) / view_width;
    }

    fn set_view_height( &mut self, view_height: f64 )
    {
        self.view_zoom = f64::from( self.window_size.1 ) / view_height;
    }

    fn get_view_size( &self ) -> ( f64, f64 )
    {
        let mut view_size =
                ( self.window_size.0.into(), self.window_size.1.into() );
        view_size.0 /= self.view_zoom;
        view_size.1 /= self.view_zoom;

        return view_size;
    }

    fn flip( &mut self, flip_flags: ( bool, bool ) )
    {
        if flip_flags.0 {
            self.view_center.0 = -self.view_center.0;
            self.view_rotation = -self.view_rotation;

            self.flip_flags.0 = !self.flip_flags.0;
        }

        if flip_flags.1 {
            self.view_center.1 = -self.view_center.1;
            self.view_rotation = -self.view_rotation;

            self.flip_flags.1 = !self.flip_flags.1;
        }
    }
}



// AUXILIARY FUNCTIONS

fn get_bounding_box< T, U >( size: ( T, T ), rotation: U )
        -> ( f64, f64 )
where
    T: Into< f64 >,
    U: Into< f64 >,
{
    let width  = size.0.into();
    let height = size.1.into();
    let rotation = rotation.into();

    let ( x1, y1 ) = rotate_xy( ( width,  height ), rotation );
    let ( x2, y2 ) = rotate_xy( ( width, -height ), rotation );

    let x1 = x1.abs();
    let x2 = x2.abs();
    let y1 = y1.abs();
    let y2 = y2.abs();

    let width  = x1.max( x2 );
    let height = y1.max( y2 );
    let bounding_box = ( width, height );

    return bounding_box;
}

fn rotate_xy< T, U >( ( x, y ): ( T, T ), angle: U ) -> ( f64, f64 )
where
    T: Into< f64 >,
    U: Into< f64 >,
{
    let x = x.into();
    let y = y.into();
    let angle = angle.into();

    let ( sin, cos ) = angle.to_radians().sin_cos();
    let x_ = (cos * x) - (sin * y);
    let y_ = (sin * x) + (cos * y);

    return ( x_, y_ );
}

fn get_aspect_ratio< T >( size: ( T, T ) ) -> f64
where
    T: Into< f64 >,
{
    let width  = size.0.into();
    let height = size.1.into();
    let aspect_ratio = width / height;

    return aspect_ratio;
}

fn vector2_to_tuple< T >( vec: ss::Vector2< T > ) -> ( T, T )
{
    let tuple = ( vec.x, vec.y );

    return tuple;
}
