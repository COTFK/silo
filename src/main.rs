use axum::{
    extract::Query,
    routing::get,
    Router,
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::future::try_join_all;
use serde::Deserialize;
use serde::Serialize;

/*
 * The structs "Name" and "Card," as well as the function "get_card," are associated with the
 * processing of the JSON returned by yaml-yugi. They can be updated if more fields from yaml-yugi
 * are needed.
 */
#[derive(Serialize, Deserialize)]
struct Name {
    en: String,
}

#[derive(Serialize, Deserialize)]
struct Card {
    name: Name,
}

async fn get_card(password: u32) -> Result<Card, reqwest::Error> {
    let url = format!("https://raw.githubusercontent.com/DawnbrandBots/yaml-yugi/master/data/cards/{:08}.json", password);

    reqwest::get(url).await?.error_for_status()?.json::<Card>().await
}

/*
 * The struct "CardsQuery" is deserialized (using the function "deserialize_list") from the GET
 * param received by Axum.
 */
#[derive(Deserialize)]
struct CardsQuery {
    #[serde(deserialize_with = "deserialize_list")]
    list: Vec<u32>,
}

fn deserialize_list<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    s.split(',')
        .map(|part| {
            part.parse::<u32>()
                .map_err(serde::de::Error::custom)
        })
        .collect()
}

/*
 * This error is used to bubble up errors from reqwest back to the client. Essentially, if reqwest
 * returns an error, it's wrapped in an AppError and Axum returns that AppError to the client.
 * Some traits are implemented for AppError to fulfill trait bounds that Axum requires.
 */
struct AppError(reqwest::Error);

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_GATEWAY,
            self.0.to_string(),
        )
            .into_response()
    }
}

/*
 * This function returns the list of card names as Json to the caller, or returns an AppError on
 * failure.
 */
async fn cards(Query(query): Query<CardsQuery>) -> Result<Json<Vec<Card>>, AppError> {

    let futures = query.list.into_iter().map(get_card);

    let cards: Vec<Card> = try_join_all(futures).await?;

    Ok(Json(cards))

}


#[tokio::main]
async fn main() {
    let app = Router::new().route("/v1/cards", get(cards));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}



