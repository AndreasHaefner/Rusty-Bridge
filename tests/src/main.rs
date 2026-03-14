// Everything AI, use with care (testing only so it should be fine)

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let url = "wss://rusty-bridge.haefner-co.de/ws";
    
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let iterations = 10000;
    let mut latencies = Vec::with_capacity(iterations);
    let mut failures = 0;

    println!("Starte Belastungstest (10k Requests)...");

    for _ in 0..iterations {
        let start = Instant::now();
        
        // Senden und Empfangen in einem Block zur Fehlererkennung
      let result = async {
            write.send(Message::Text("deal".into())).await
                .map_err(|e| e.to_string())?; // Fehler explizit zu String wandeln
            
            read.next().await
                .ok_or("Stream geschlossen".to_string())? // Option zu Result
                .map_err(|e| e.to_string())?; // Fehler zu String wandeln
            
            Ok::<(), String>(())
        }.await;

        match result {
            Ok(_) => latencies.push(start.elapsed().as_micros()),
            Err(_) => failures += 1, // Zähle Fehler ohne das Programm zu beenden
        }
    }

    // Berechnung nur bei erfolgreichen Requests
    if !latencies.is_empty() {
        let sum: u128 = latencies.iter().sum();
        let avg = sum / latencies.len() as u128;
        let min = *latencies.iter().min().unwrap();
        let max = *latencies.iter().max().unwrap();

        println!("\n--- Benchmark Ergebnis ---");
        println!("Erfolgreich:  {}", latencies.len());
        println!("Fehler:       {}", failures);
        println!("Durchschnitt: {:?} µs", avg);
        println!("Minimum:      {:?} µs", min);
        println!("Maximum:      {:?} µs", max);
    } else {
        println!("\nTest fehlgeschlagen: Keine erfolgreichen Requests.");
    }
}