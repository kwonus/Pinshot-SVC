extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::any::Any;
use pest::Parser;
use pest::iterators::Pairs;

#[derive(Parser)]
#[grammar = "avx-quelle.pest"]
struct QuelleParser;

use axum::http::StatusCode;
use axum::routing::get;
use axum::routing::post;
use axum::response::IntoResponse;
use axum::Json;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;


#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/simple", get(get_parse_simple))
        .route("/quelle", get(get_parse))
        .route("/quelle_post", post(get_parse_via_post));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//  tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "I got your pinshot. I keep it with your letter!"
}
// basic handler that responds with a static string

#[derive(Serialize)]
struct Parsed {
    rule: String,
    text: String,
    children: Vec<Parsed>,
}

#[derive(Deserialize)]
struct QuelleInput {
    username: String,
}

#[derive(Serialize)]
struct RootParse {
    input: String,
    result: Vec<Parsed>,
}

async fn get_parse_simple() -> String {
    let input = "\"\\foo\\ ... [he \t said] ... /pronoun/&/3p/\" + bar + x|y&z a&b&c > xfile < genesis 1:1";
    let input_string = input.to_string();

    let pairs = QuelleParser::parse(Rule::command, input).unwrap_or_else(|e| panic!("{}", e));
    pairs.to_string()
}

async fn get_parse() -> (StatusCode, Json<RootParse>) {
    let input = "\"\\foo\\ ... [he \t said] ... /pronoun/&/3p/\" + bar + x|y&z a&b&c > xfile < genesis 1:1";
    let input_string = input.to_string();

    let pairs = QuelleParser::parse(Rule::command, input).unwrap_or_else(|e| panic!("{}", e));

    let mut result: Vec<Parsed> = vec![];
    recurse(pairs, &mut result);

    let root = RootParse {
        input: input_string,
        result: result,
    };
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(root))
}

async fn get_parse_via_post(Json(payload): Json<QuelleInput>) -> (StatusCode, Json<RootParse>) {
    let input = "\"#foo ... [he \t said] ... /pronoun/&/3p/\" + bar + x|y&z a&b&c > xfile < genesis 1:1";
    let input_string = input.to_string();

    let pairs = QuelleParser::parse(Rule::command, input).unwrap_or_else(|e| panic!("{}", e));

    let mut result: Vec<Parsed> = vec![];
    recurse(pairs, &mut result);

    let root = RootParse {
        input: input_string,
        result: result,
    };
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(root))
}

fn recurse(children: Pairs<Rule>, items: &mut Vec<Parsed>)
{
    for pair in children {
        let mut result: Vec<Parsed> = vec![];
        let text = pair.as_str().to_string();
        let rule = pair.to_string();
        // A non-terminal pair can be converted to an iterator of the tokens which make it up:
        recurse(pair.into_inner(), &mut result);

        let item = Parsed {
            rule: rule,
            text: text,
            children: result,
        };
        items.push(item);
    }
}