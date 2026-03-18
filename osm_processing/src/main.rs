//Std library
use std::error::Error;

// REading OSMs
mod osm_readers;
use osm_readers::osmread::{getcontour, Contour, Node, Bound, overpass_osm};
use std::collections::HashMap;

// For drawing preview
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;

use sdl2::gfx::primitives::DrawRenderer;

const SCREEN_WIDTH: u32 = 1872;
const SCREEN_HEIGHT: u32 = 1404;

// ---------------
fn linspace(start: f64, end: f64, num: usize) -> Vec<f64> {
    if num < 2 {
        panic!("num must be at least 2");
    }
    let step = (end - start) / (num - 1) as f64;
    (0..num).map(|i| start + i as f64 * step).collect()
}

fn contour_feedback(contours:&Vec<Contour>, on:bool){
    if on{
        println!("I'll print an example now. First Contour:");
        // Print first contour as example
        if let Some(example) = contours.first() {
            println!("Contour at {}m has {} nodes:", example.elevation, example.nodes.len());
            for node in &example.nodes {
                println!("  Node {}: {:.6}, {:.6}", node.id, node.lat, node.lon);
            }
        }
        println!("Enjoy the numbers. Here is the last contour:");
        if let Some(example) = contours.last() {
            println!("Contour at {}m has {} nodes:", example.elevation, example.nodes.len());
            for node in &example.nodes {
                println!("  Node {}: {:.6}, {:.6}", node.id, node.lat, node.lon);
            }
        }
        if let Some(example) = contours.last() {
            println!("Contour at {}m has {} nodes:", example.elevation, example.nodes.len());
            for node in &example.nodes {
                println!("  Node {}: {:.6}, {:.6}", node.id, node.pix_y, node.pix_x);
            }
        }
        if let Some(example) = contours.first() {
            println!("Contour at {}m has {} nodes:", example.elevation, example.nodes.len());
            for node in &example.nodes {
                println!("  Node {}: {:.6}, {:.6}", node.id, node.pix_y, node.pix_x);
            }
        }
    }
    else{
        return
    }
}

//----------------
fn main() -> Result<(), Box<dyn Error>> {


    let osm_file_path = "./Srtm2Osm/edale.osm";
    let file_json = "./Srtm2Osm/jsontest.json";
    let file_osm = "./Srtm2Osm/osmtest.osm";

/// Imports the countours from a given OSM file
// Returns an example, stores values in contour vectors with references to 
// nodes in a hashmap
    let mut contours = Vec::<Contour>::new();
    let mut nodes = HashMap::<String, Node>::new();
    let mut boundbox = Bound{lat_min: 0.0, lat_max: 0.0, lon_min: 0.0, lon_max: 0.0};
    let c_fb = false;


    match getcontour(osm_file_path) {
    Ok((rtnd_nodes, rtnd_contours,rtnd_boundbox)) => {
        nodes = rtnd_nodes;
        contours = rtnd_contours;
        boundbox = rtnd_boundbox;
        println!("Contours Extracted");
    }
    Err(e) => {
        println!("Error Extracting Contours: {}", e);
    }
    }

    let tout: u64 = 90;

    overpass_osm(&boundbox,file_json,file_osm, tout)?;
    // json_to_osm(file_json,file_osm);

    contour_feedback(&contours, c_fb);

    // Use boundingbox to pring scale
    // Each pixel will be a lon lat chunk
    // the max
    println!("Bounding Box: Lat Min: {}, Lat Max: {}, Lon Min: {}, Lon Max: {}", boundbox.lat_min, boundbox.lat_max, boundbox.lon_min, boundbox.lon_max);
    let lat_height = boundbox.lat_max - boundbox.lat_min;
    let lon_width = boundbox.lon_max - boundbox.lon_min;

    // Normalise the node values and get pixel coordinates
    for contour in &mut contours {
        for node in &mut contour.nodes {
            node.norm_lat = (node.lat - boundbox.lat_min) / lat_height;
            node.norm_lon = (node.lon - boundbox.lon_min) / lon_width;
            node.pix_x = (node.norm_lon * SCREEN_WIDTH as f64) as i32;
            node.pix_y = (node.norm_lat * SCREEN_HEIGHT as f64) as i32;
        }
    }

// ----
// inits canvas and draws contours on the canvas
    
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window(
            "Contour Map Output",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    // For each coutour
    // For each pair of nodes
    // Draw a line between the nodes
    // !! Convert the scale appropriately. 
    // !! Scale exists in scale variable and converts lat/lon to pixel coordinates
    canvas.present();
    
    let mut events = sdl_context.event_pump()?;
// ----
// Draws contours on the canvas
    'main: loop {
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'main,
    
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => {
                        if keycode == Keycode::Escape {
                            break 'main;
                        } else if keycode == Keycode::Space {
                            println!("space down");
                            for contour in &contours {
                                for i in 0..contour.nodes.len() - 1 {
                                    let color = pixels::Color::RGB(255, 255, 255);
                                    let _ = canvas.line(contour.nodes[i].pix_x as i16, contour.nodes[i].pix_y  as i16, contour.nodes[i + 1].pix_x  as i16, contour.nodes[i + 1].pix_y  as i16, color);
                                }
                            }
                            canvas.present();
                        }
                    }
    
                    _ => {}
                }
            }
        }



    Ok(())
}