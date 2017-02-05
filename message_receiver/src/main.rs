extern crate nanomsg;
extern crate sdl2;
extern crate clap;

use clap::{Arg, App, SubCommand};


use std::path::Path;
use std::fs::File;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::render::Renderer;
use sdl2::pixels::Color;
use sdl2::render::Texture;
use sdl2::render::BlendMode;
use sdl2::ttf::Font;

use nanomsg::{Socket, Protocol, PollFd, PollInOut, PollRequest};

use std::thread;
use std::time::Duration;

use std::io::{Read, Write};

static SCREEN_WIDTH: u32 = 800;
static SCREEN_HEIGHT: u32 = 480;

macro_rules! rect (
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn show_text(text: String, font_percent: &mut Font, disp_size: Rect, renderer: &mut Renderer, x_offset: u32) {
    let mut split = text.split("|");
    let mut offset = 22;
    renderer.fill_rect(Rect::new((0 + x_offset) as i32, 0, disp_size.width()/3, disp_size.height()));
    renderer.set_draw_color(Color::RGBA(0, 0, 0, 255)); // Black
    for text in split {
        println!("{}", text);
        //let surface_coffee_percent = font_percent.render((&text)).blended_wrapped(Color::RGBA(255, 255, 255, 255), 300).unwrap();
        let surface_coffee_percent = font_percent.render((&text)).shaded(Color::RGBA(255, 255, 255, 255), Color::RGBA(0, 0, 0, 0)).unwrap();
        let mut coffee_tex = renderer.create_texture_from_surface(&surface_coffee_percent).unwrap();
        let TextureQuery { width, height, .. } = coffee_tex.query();
        let coffe_tex_rect = rect!(5 + x_offset, 0 + offset, width, height);
        offset = offset + height;
//        renderer.fill_rect(Rect::new(0 + x_offset, 0, width/3, height));
//        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255)); // Black
        renderer.set_blend_mode(BlendMode::None);
        renderer.copy(&coffee_tex, None, Some(coffe_tex_rect));
    }
    renderer.present();
}


fn main() {
    let matches = App::new("message_receiver")
        .arg(Arg::with_name("font_path")
                 .help("path and ttf font to use") // Displayed when showing help info
                 .required(true)                // By default this argument MUST be present
        )
        .get_matches();

    let font_path = matches.value_of("font_path").unwrap();
    //    let traffic_url ="ipc:///tmp/pubsub.ipc";
    let day_url = "tcp://127.0.0.1:8021";
    let weather_url = "tcp://127.0.0.1:8022";
    let traffic_url = "tcp://127.0.0.1:5555";
    let mut socket_day = Socket::new(Protocol::Sub).unwrap();
    let mut endpoint_day = socket_day.bind(day_url).unwrap();

    let mut socket = Socket::new(Protocol::Sub).unwrap();
    let mut endpoint = socket.bind(traffic_url).unwrap();

    let mut socket_weather = Socket::new(Protocol::Sub).unwrap();
    let mut endpoint_weather = socket_weather.bind(weather_url).unwrap();


    match socket_day.subscribe(b"") {
        Ok(_) => println!("Subscribed to '{}'.", "day"),
        Err(err) => panic!("{}", err)
    }
    match socket.subscribe(b"") {
        Ok(_) => println!("Subscribed to '{}'.", "traffic"),
        Err(err) => panic!("{}", err)
    }
    match socket_weather.subscribe(b"") {
        Ok(_) => println!("Subscribed to '{}'.", "weather"),
        Err(err) => panic!("{}", err)
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let disp_size = video_subsys.display_bounds(0).ok().expect("Could not read size of display 0");
    //let disp_size = Rect::new(0i32, 0i32, SCREEN_WIDTH, SCREEN_HEIGHT);
    let window = video_subsys.window("SDL2_TTF Example", disp_size.width(), disp_size.height())
        .position_centered()
        .opengl()
        .fullscreen()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();
    renderer.set_draw_color(Color::RGBA(0, 0, 0, 255)); // Black
    renderer.clear();
    renderer.present();

    // Load a font
    let path: &Path = Path::new(font_path);
    let mut font = ttf_context.load_font(path, 128).unwrap();
    let mut font_percent = ttf_context.load_font(path, 18).unwrap();
    font.set_style(sdl2::ttf::STYLE_BOLD);


    let mut pollfd_vec: Vec<PollFd> = vec![
        socket.new_pollfd(PollInOut::In),
        socket_day.new_pollfd(PollInOut::In),
        socket_weather.new_pollfd(PollInOut::In)];

    let mut poll_req = PollRequest::new(&mut pollfd_vec[..]);
    let timeout = 1000;
    'mainloop: loop {
        let poll_result = Socket::poll(&mut poll_req, timeout);
        match poll_result {
            Ok(_) => println!("Something to read"),
            Err(err) => println!("{}", err)
        }
        if poll_req.get_fds()[1].can_read() {
            let mut msg = String::new();
            match socket_day.read_to_string(&mut msg) {
                Ok(_) => {
                    println!("Day Recv '{}'.", msg);
                    show_text(msg, &mut font_percent, disp_size, &mut renderer, disp_size.width()/3);
                },
                Err(err) => {
                    println!("Client failed to receive msg '{}'.", err);
                }
            }
        }
        if poll_req.get_fds()[0].can_read() {
            let mut msg = String::new();
            match socket.read_to_string(&mut msg) {
                Ok(_) => {
                    println!("Traffic Recv '{}'.", msg);
                    show_text(msg, &mut font_percent, disp_size, &mut renderer, 0);
                },
                Err(err) => {
                    println!("Client failed to receive msg '{}'.", err);
                }
            }
        }
        if poll_req.get_fds()[2].can_read() {
            let mut msg = String::new();
            match socket_weather.read_to_string(&mut msg) {
                Ok(_) => {
                    println!("Weather Recv '{}'.", msg);
                    show_text(msg, &mut font_percent, disp_size, &mut renderer, disp_size.width()/3*2);
                },
                Err(err) => {
                    println!("Client failed to receive msg '{}'.", err);
                }
            }
        }
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'mainloop,
                _ => {}
            }
        }
    }

    endpoint.shutdown();
    endpoint_day.shutdown();
    endpoint_weather.shutdown();
}
