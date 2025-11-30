use axum::{
    extract::Form,
    response::Html,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::{
    fs::OpenOptions,
    io::Write,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Build our router with two routes:
    // GET /      -> show the HTML form
    // POST /submit -> handle submitted text
    let app = Router::new()
        .route("/", get(show_form))
        .route("/submit", post(handle_submit));

    // Bind to 0.0.0.0 so your phone on the LAN can reach it
    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to address");

    println!("Server running on http://localhost:3000");
    axum::serve(listener, app)
        .await
        .expect("server error");
}

// ----- Data types -----

#[derive(Debug)]
struct Item {
    name: String,
    quantity: i32,
}

#[derive(Deserialize)]
struct InputForm {
    // `name="text"` in the HTML form must match this field name
    text: String,
}

// ----- Handlers -----

async fn show_form() -> Html<&'static str> {
    // Super simple HTML, no JS, one big textarea and a submit button
    Html(r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Inventory Inbox</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
  </head>
  <body style="font-family: sans-serif; padding: 1rem;">
    <h1>Inventory Inbox</h1>
    <form method="post" action="/submit">
      <label for="text">Speak or paste your message:</label><br>
      <textarea id="text" name="text" rows="8" cols="40" style="width: 100%;"></textarea><br><br>
      <button type="submit">Submit</button>
    </form>
  </body>
</html>"#)
}

async fn handle_submit(Form(input): Form<InputForm>) -> Html<String> {
    // This is where your LLM will eventually live.
    let items = fake_llm_parse(&input.text);

    // Append items to a CSV file as a stub for "saving inventory".
    if let Err(e) = append_items_to_csv("inventory.csv", &items) {
        eprintln!("Failed to write CSV: {e}");
    }

    // TODO: send labels to Zebra printer here.

    // Render a simple confirmation page listing what we parsed.
    let mut html = String::new();
    html.push_str("<!doctype html><html><head><meta charset=\"utf-8\"><title>Inventory Saved</title>");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"></head><body style=\"font-family: sans-serif; padding: 1rem;\">");
    html.push_str("<h1>Parsed Items</h1><ul>");

    for item in &items {
        html.push_str(&format!(
            "<li>{} &times; {}</li>",
            item.quantity, html_escape(&item.name)
        ));
    }

    html.push_str("</ul>");
    html.push_str(r#"<p><a href="/">Back</a></p>"#);
    html.push_str("</body></html>");

    Html(html)
}

// ----- "LLM" stub -----

/// For now, this is a fake “LLM parser” so you can test the flow.
/// Replace this with a real local LLM call later.
fn fake_llm_parse(raw: &str) -> Vec<Item> {
    // Extremely dumb parser:
    // - split on newlines
    // - treat a leading number as quantity, rest as name
    // Example input lines:
    // "3 boxes of screws"
    // "2x paint brush"
    // "hammer"  (defaults to quantity = 1)
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            // Try to parse a leading integer
            let mut parts = line.split_whitespace();
            let first = parts.next().unwrap_or("");

            if let Ok(qty) = first.trim_end_matches(|c: char| !c.is_ascii_digit()).parse::<i32>() {
                let name = parts.collect::<Vec<_>>().join(" ");
                Item {
                    name: if name.is_empty() { line.to_string() } else { name },
                    quantity: qty,
                }
            } else {
                Item {
                    name: line.to_string(),
                    quantity: 1,
                }
            }
        })
        .collect()
}

// ----- CSV stub -----

fn append_items_to_csv(path: &str, items: &[Item]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    for item in items {
        // Very naive CSV: quantity,name
        // (No escaping of commas/quotes; good enough for version 0.)
        writeln!(file, "{},{}", item.quantity, item.name)?;
    }

    Ok(())
}

// ----- Small helper -----

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
