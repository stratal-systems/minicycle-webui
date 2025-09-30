use leptos::*;
use leptos::prelude::*;
use console_log::init_with_level;
use console_error_panic_hook::set_once;
use log::Level;
use leptos::logging::{log, warn, error};
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;

mod schema;
use crate::schema::v1::Report;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    mount_to_body(|| view! { <App/> });
}

async fn get_data(_: ()) -> String {
    match Request::get("https://mitakihara.webhook.stratal.systems/report-latest")
        .send()
        .await

        {
            Ok(response) => match response.json::<Report>().await {
                Ok(parsed) => format!("Parsed: {:#?}!", parsed).to_string(),
                Err(err) => format!("Parse err: {:#?}!", err).to_string(),
                //Err(err) => match response.text().await {
                //    Ok(text) => format!("Parse err: {:#?}", text).to_string(),
                //    Err(err) => format!("Parse text err: {:#?}", err).to_string(),
                //},
            },
            Err(err) => "network error!".to_string(),
        }
}

//#[component]
//pub fn ReportResult(status: LocalResource<i32>) -> impl IntoView {
//    match status.get() {
//        Some(status) => match status {
//            0 => view! { <span style="color:red;">"Foo!"</span> },
//            1 => view! { <span style="color:blue;">"Bar!"</span> },
//            _ => view! { <span style="color:green;">"Baz!"</span> },
//            },
//        None => view! { <span style="color:gray;">"Loading...."</span> },
//    }
//}
//

#[derive(Clone)]
pub enum APIErr {
    Decode,
    Network
}

pub async fn get_report_result(url: String) -> Result<Report, APIErr> {
    match Request::get(&url)
        .send()
        .await
        {
            Ok(response) => match response.json::<Report>().await {
                Ok(parsed) => Ok(parsed),
                Err(err) => Err(APIErr::Decode),
            },
            Err(err) => Err(APIErr::Network),
    }
    // TODO use map_err
}

#[component]
fn ResultDisplay(
    #[prop(into)]
    result: Result<Report, APIErr>
    ) -> impl IntoView {
    match result {
        Ok(report) => view! { <p> {format!("{:#?}", report)} </p> }.into_any(),
        Err(apierr) => match apierr {
            APIErr::Decode => view! { <p> "Decode error" </p> }.into_any(),
            APIErr::Network => view! { <p> "Network error" </p> }.into_any(),
        }
    }
    // TODO docs said this is bad?
}


#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);
    let async_data = LocalResource::new(move || { count.get(); get_report_result("http://localhost:8081/foo.json".to_string()) } );

    view! {
        <button
            on:click=move |_| *set_count.write() += 3
        >
            "Click me: "
            {count}
        </button>
        
        {move || match async_data.get() {
            None => view! { <p> "Loading" </p> }.into_any(),
            Some(val) => view! { <ResultDisplay result=val /> }.into_any()
        }}
    }
}

