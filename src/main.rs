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
 //     .route("/quelle-test", get(get_parse))
        .route("/quelle", post(get_parse_via_post));

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
struct QuelleStatement {
    statement: String,
}

#[derive(Serialize)]
struct RootParse {
    input: String,
    result: Vec<Parsed>,
    error: String,
}

async fn get_parse_simple() -> String {
    //let input = "\"\\foo\\ ... [he \t said] ... /pronoun/&/3p/\" + bar + x|y&z a&b&c > xfile < genesis 1:1";
    let input = "@Help find";

    let pairs = QuelleParser::parse(Rule::statement, input).unwrap_or_else(|e| panic!("{}", e));
    pairs.to_string()
}
/*
async fn get_parse() -> (StatusCode, Json<RootParse>) {
    let input = "\"\\foo\\ ... [he \t said] ... /pronoun/&/3p/\" + bar + x|y&z a&b&c > xfile < genesis 1:1";
    let input_string = input.to_string();

    let mut result: Vec<Parsed> = vec![];
    let mut message: String = "".to_string();
    let task = QuelleParser::parse(Rule::command, input);

    match task {
        Ok(value)    => result = recurse(value, &mut result),
        Err(error)   => message = error.to_string(),
    }

    let root = RootParse {
        input: input_string,
        result: result,
        error: message,
    };
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(root))
}
*/
async fn get_parse_via_post(Json(payload): Json<QuelleStatement>) -> (StatusCode, Json<RootParse>) {
    let input_string = payload.statement.clone();

    let mut top: Vec<Parsed> = vec![];
    let task = QuelleParser::parse(Rule::statement, &payload.statement);

    if task.is_ok() {
        let pairs = task.unwrap();
        recurse(pairs, &mut top);
        let root = RootParse {
            input: input_string,
            result: top,
            error: "".to_string(),
        };
        (StatusCode::CREATED, Json(root))
    }
    else if task.is_err() {
        let root = RootParse {
            input: input_string,
            result: top,
            error: task.unwrap_err().to_string(),
        };
        (StatusCode::NOT_FOUND, Json(root))
    }
    else {
        let root = RootParse {
            input: input_string,
            result: top,
            error: "Internal Error".to_string(),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(root))
    }
}

fn recurse(children: Pairs<Rule>, items: &mut Vec<Parsed>)
{
    for pair in children {
        let mut result: Vec<Parsed> = vec![];
        let text = pair.as_str().to_string();
        let mut rule = pair.to_string();
        // A non-terminal pair can be converted to an iterator of the tokens which make it up:
        recurse(pair.into_inner(), &mut result);

        //let paren = rule.find('(').unwrap();
        //if paren > 0 {
            //rule = rule[0..paren].to_string();
        //}
        let item = Parsed {
            rule: rule,
            text: text,
            children: result,
        };
        items.push(item);
    }
}