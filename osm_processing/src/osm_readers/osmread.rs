use roxmltree::Document;
use std::collections::HashMap;
use std::fs;
use std::error::Error;
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs::File;
use std::io::{self, Write};
use std::time::Duration;
// ------------------------------------------------------------------


pub struct Contour {
    pub elevation: i32,
    pub nodes: Vec<Node>,
}

#[derive(Default)]
pub struct Node {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub norm_lat: f64,
    pub norm_lon: f64,
    pub pix_x : i32,
    pub pix_y : i32,
}

pub struct Bound {
    pub lat_min: f64,
    pub lat_max: f64,
    pub lon_min: f64,
    pub lon_max: f64,   
}

pub fn loadosm(file:&str) -> Result<String, Box<dyn Error>> {
    match fs::read_to_string(file){
        Ok(content) => {
            println!("File Read Successfully");
            Ok(content)
        }
        Err(e) => {
            println!("Error Reading File: {}", e);
            Err(Box::new(e))
        }
    }
}

pub fn getcontour(file:&str) -> Result<(HashMap<String, Node>, Vec<Contour>, Bound), Box<dyn Error>> {
    // load the osm file
    let osm = loadosm(file)?;
    println!("OSMcontours loaded successfully ({} bytes)", osm.len());
    // process the osm file
    let doc = Document::parse(&osm)?;
    println!("OSMcontours parsed successfully");
    let mut nodesmap: HashMap<String, Node> = HashMap::new();
    let mut contours: Vec<Contour> = Vec::new();

    // init bounding box
    let mut last_lat_max = 0.0;
    let mut last_lat_min = 0.0;
    let mut last_lon_max = 0.0;
    let mut last_lon_min = 0.0;
    let mut firstcall = true;
    for node in doc.descendants().filter(|n| n.has_tag_name("node")) {
        let id = node.attribute("id").unwrap().to_string();
        let lat = node.attribute("lat").unwrap().parse::<f64>().unwrap();
        let lon = node.attribute("lon").unwrap().parse::<f64>().unwrap();
        if firstcall {
            last_lat_max = lat;
            last_lat_min = lat;
            last_lon_max = lon;
            last_lon_min = lon;
            firstcall = false;
        }
        else if lat > last_lat_max {
            last_lat_max = lat;}
        else if lat < last_lat_min {
            last_lat_min = lat;
        }
        else if lon > last_lon_max {
            last_lon_max = lon;
        }
        else if lon < last_lon_min {
            last_lon_min = lon;
        }
        nodesmap.insert(id.clone(), Node{id:id, lat:lat, lon:lon,..Default::default()});
    }
    let boundbox = Bound{lat_min: last_lat_min, lat_max: last_lat_max, lon_min: last_lon_min, lon_max: last_lon_max};
    
    println!("Contour Nodes processed successfully");
    // For each contour, stored as a way in the OSM file
    for way in doc.descendants().filter(|n| n.has_tag_name("way")) {
        // Find elevation tag
        if let Some(elevation) = way.children()
            .find(|n| n.has_tag_name("tag") && n.attribute("k") == Some("ele"))
            .and_then(|tag| tag.attribute("v"))
            .and_then(|v| v.parse::<i32>().ok())
        {
            // Collect referenced nodes
            let mut contour_nodes = Vec::new();
            for nd in way.children().filter(|n| n.has_tag_name("nd")) {
                if let Some(ref_id) = nd.attribute("ref") {
                    if let Some(node) = nodesmap.get(ref_id) {
                        contour_nodes.push(Node {
                            id: node.id.clone(),
                            lat: node.lat,
                            lon: node.lon,
                            ..*node
                        });
                    }
                }
            }
            
            contours.push(Contour {
                elevation,
                nodes: contour_nodes,
            });
        }
    }

    Ok((nodesmap, contours,boundbox))
}

pub fn overpass_osm(bbox:&Bound, file_path: &str,osm_path: &str,tout: u64) -> Result<(), Box<dyn Error>>{
    
    let mut file = File::create(file_path)?;
    
    let (s,w,n,e) = (bbox.lat_min,bbox.lon_min,bbox.lat_max,bbox.lon_max);
    let overpass_query = format!(
        "[out:json];way({},{},{},{});out;",
        s, w, n, e
    );
    let url = "https://overpass-api.de/api/interpreter";
    let client = Client::builder().
        timeout(Duration::from_secs(tout))
        .build()?;

    println!("Sending request to Overpass API: {}", url);
    println!("Query: {}", overpass_query);

    let response = client.post(url).body(overpass_query).send()?;
    let json_data: Value = response.json()?;
    let json_string = serde_json::to_string_pretty(&json_data)?;
    file.write_all(json_string.as_bytes())?;
    println!("JSON data saved to {}", file_path);
    
    json_to_osm(file_path,osm_path);
    Ok(())
}

fn json_to_osm(json_path: &str, osm_path: &str) -> io::Result<()> {
    // Read the JSON file
    let json_data = std::fs::read_to_string(json_path)?;
    let parsed_json: Value = serde_json::from_str(&json_data)?;

    // Open the output OSM file
    let mut osm_file = File::create(osm_path)?;

    // Write the OSM XML header
    writeln!(osm_file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(osm_file, r#"<osm version="0.6" generator="rust-json-to-osm">"#)?;

    // Extract nodes
    if let Some(elements) = parsed_json["elements"].as_array() {
        for element in elements {
            if element["type"] == "node" {
                let id = element["id"].as_i64().unwrap_or(0);
                let lat = element["lat"].as_f64().unwrap_or(0.0);
                let lon = element["lon"].as_f64().unwrap_or(0.0);
                writeln!(
                    osm_file,
                    r#"<node id="{}" lat="{}" lon="{}">"#,
                    id, lat, lon
                )?;

                // Write tags if they exist
                if let Some(tags) = element["tags"].as_object() {
                    for (key, value) in tags {
                        writeln!(
                            osm_file,
                            r#"  <tag k="{}" v="{}" />"#,
                            key, value.as_str().unwrap_or("")
                        )?;
                    }
                }

                writeln!(osm_file, r#"</node>"#)?;
            }
        }
    }

    // Extract ways
    if let Some(elements) = parsed_json["elements"].as_array() {
        for element in elements {
            if element["type"] == "way" {
                let id = element["id"].as_i64().unwrap_or(0);
                writeln!(osm_file, r#"<way id="{}">"#, id)?;

                if let Some(nodes) = element["nodes"].as_array() {
                    for node_id in nodes {
                        writeln!(osm_file, r#"  <nd ref="{}" />"#, node_id)?;
                    }
                }

                // Write tags if they exist
                if let Some(tags) = element["tags"].as_object() {
                    for (key, value) in tags {
                        writeln!(
                            osm_file,
                            r#"  <tag k="{}" v="{}" />"#,
                            key, value.as_str().unwrap_or("")
                        )?;
                    }
                }

                writeln!(osm_file, r#"</way>"#)?;
            }
        }
    }

    // Extract relations (optional)
    if let Some(elements) = parsed_json["elements"].as_array() {
        for element in elements {
            if element["type"] == "relation" {
                let id = element["id"].as_i64().unwrap_or(0);
                writeln!(osm_file, r#"<relation id="{}">"#, id)?;

                if let Some(members) = element["members"].as_array() {
                    for member in members {
                        let member_type = member["type"].as_str().unwrap_or("");
                        let ref_id = member["ref"].as_i64().unwrap_or(0);
                        let role = member["role"].as_str().unwrap_or("");
                        writeln!(
                            osm_file,
                            r#"  <member type="{}" ref="{}" role="{}" />"#,
                            member_type, ref_id, role
                        )?;
                    }
                }

                // Write tags if they exist
                if let Some(tags) = element["tags"].as_object() {
                    for (key, value) in tags {
                        writeln!(
                            osm_file,
                            r#"  <tag k="{}" v="{}" />"#,
                            key, value.as_str().unwrap_or("")
                        )?;
                    }
                }

                writeln!(osm_file, r#"</relation>"#)?;
            }
        }
    }

    // Write the OSM XML footer
    writeln!(osm_file, r#"</osm>"#)?;

    Ok(())
}