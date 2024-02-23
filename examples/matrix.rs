use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use google_maps::prelude::*;

use indicatif::ProgressIterator;

#[tokio::main]
async fn main() {
    let mut destinations = File::open("destinations.txt").unwrap();
    let mut content = String::new();
    destinations.read_to_string(&mut content).unwrap();
    content = content.trim_end_matches("\n").to_string();
    // println!("{content:?}");
    let destinations = content.split("\n").into_iter().collect::<Vec<&str>>();
    println!("{destinations:?}");

    let mut google_maps_client = GoogleMapsClient::new(env!("GMAPS_TOKEN"));
    google_maps_client.with_rate(Api::All, 1, std::time::Duration::new(4, 0));

    // Example request:
    let mut keys = HashMap::new();

    for origin in destinations.iter().progress() {
        let destinations: Vec<&&str> = destinations.iter().filter(|x| *x != origin).collect();
        let waypoints: Vec<Waypoint> = destinations
            .iter()
            .map(|x| format!("{x}, Motala kommun"))
            .map(|x| Waypoint::Address(x))
            .collect();

        let origin_waypoint = vec![Waypoint::Address(format!("{origin}, Motala kommun"))];
        for (des, way) in destinations.iter().zip(waypoints.into_iter()).progress() {

        let matrix = google_maps_client
            .distance_matrix(
                // Origins
                origin_waypoint.clone(),
                // Destinations
                vec![way],
            )
            .with_language(Language::Swedish)
            .with_restriction(Avoid::Tolls)
            .execute()
            .await;
        if let Ok(res) = matrix {
            let elements = res.rows.into_iter().next().unwrap().elements;
                keys.entry(origin.to_string()).or_insert_with(HashMap::new).insert(des.to_string(), elements);
            // keys.insert(origin.to_string(), dest_elem);
        } else {
            println!("{matrix:#?}")
        }

}    }

    let json = serde_json::to_string_pretty(&keys).unwrap();
    let mut buffer = File::create("matrix.json").unwrap();
    File::write_all(&mut buffer, json.as_bytes()).unwrap();
}
