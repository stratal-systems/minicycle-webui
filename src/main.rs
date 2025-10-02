use leptos::*;
use leptos::prelude::*;
use console_log::init_with_level;
use console_error_panic_hook::set_once;
use log::Level;
use leptos::logging::{log, warn, error};
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use std::time::SystemTime;
use std::time::Duration;
use std::fmt::Debug;

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
    Network(String),
    Decode(String),
}

impl std::fmt::Display for APIErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            APIErr::Network(msg) => write!(f, "{}", msg),
            APIErr::Decode(msg) => write!(f, "{}", msg),
        }
    }
}

pub async fn generic_get<T>(url: String) -> Result<T, APIErr>
where T: Clone + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    match Request::get(&url)
        .send()
        .await
        {
            Ok(response) => match response.json::<T>().await {
                Ok(parsed) => Ok(parsed),
                Err(err) => Err(APIErr::Decode(err.to_string())),
            },
            Err(err) => Err(APIErr::Network(err.to_string())),
    }
    // TODO use map_err
}

#[component]
fn ResultLoading() -> impl IntoView {
    view! {
        <p> "Loading..." </p>
    }
}

#[component]
fn ErrorDisplay(error: APIErr) -> impl IntoView {
    view! {
        <b> { match error {
            APIErr::Network(_) => "Network err",
            APIErr::Decode(_) => "Decode err",
        } } </b>
        <p> { error.to_string() } </p>
    }
}

#[component]
fn ResultLoaded(
        report: Report,
        #[prop(into)]
        viewer_sig: WriteSignal<bool>,
    ) -> impl IntoView {
    view! {
        <ul>
            <li> "Started: " <VersaTime unixtime=report.start.time /> </li>
            <li> "Finished: " { match report.finish {
                Some(finish) => view!{ <VersaTime unixtime=finish.time /> }.into_any(),
                None => view!{ "in progress... " }.into_any(),
                }
            } </li>
            <li> "Commit message: " { report.message } </li>
            <li> "Commit ref: " { report.r#ref } </li>
            <li> "Artifact ID: " { report.artifacts } </li>
            <button on:click = move |_| { viewer_sig.set(true); }>
                "foo "
            </button>

        </ul>
    }
}

#[component]
fn VersaTime(unixtime: u64) -> impl IntoView {

    let systime = SystemTime::UNIX_EPOCH + Duration::from_secs(unixtime);
    let humantime = chrono_humanize::HumanTime::from(systime);
    // TODO better to use to_string that format?
    let dt: chrono::DateTime<chrono::Local> = systime.into();
    // TODO will this use browser's Local tz??
    // Do need to bridge to js manually?
    let isotime = dt.format("%+");
    // TODO better to use to_rfc3339?

    view! {
        <div class="versatime">
            <div class="wrap">
                <div class="human"> <div class="wrap"> { format!("{}", humantime) } </div> </div>
                <div class="unix"> <div class="wrap"> { format!("{}", unixtime) } </div></div>
                <div class="iso"> <div class="wrap"> { format!("{}", isotime) } </div></div>
            </div>
        </div>
    }
}

#[component]
fn ReportDisplay(
    #[prop(into)]
    report: Option<Result<Report, APIErr>>,
    #[prop(into)]
    viewer_sig: WriteSignal<bool>,
    ) -> impl IntoView {
    match report {
        None => view! { <ResultLoading /> }.into_any(),
        Some(result) => match result {
            Ok(report) => view! { <ResultLoaded report=report viewer_sig=viewer_sig/> }.into_any(),
            Err(err) => view! { <ErrorDisplay error=err /> }.into_any(),
            //Err(_) => view! { "whoops" }.into_any(),
            //Err(err) => view! { { err.what } }.into_any(),
            //Err(apierr) => match apierr {
            //    APIErr::Decode(err) => view! { <ResultDecodeErr /> }.into_any(),
            //    APIErr::Network(err) => view! { <ResultNetErr / > }.into_any(),
            //}
        }
    }
    // TODO docs said this is bad?
}

#[component]
fn LogViewer(content: String) -> impl IntoView {
    view! {
        <pre>{content}</pre>
    }
}

#[component]
fn App() -> impl IntoView {
    let (report_r, report_w) = signal(());
    let (viewer_sig, set_viewer_sig) = signal(false);
    //let report = LocalResource::new(move || { report_r.get(); get_report_result("http://localhost:8081/foo.json".to_string()) } );
    let report = LocalResource::new(move || { report_r.get(); generic_get("http://localhost:8081/foo.json".to_string()) } );

    view! {
        <button on:click=move |_| { report_w.write(); } >
            "Click me"
        </button>
        
        { move || view! { <ReportDisplay report=report.get() viewer_sig=set_viewer_sig /> } }

        <p> {viewer_sig} </p>
        { move || view! { <LogViewer content=viewer_sig.get().to_string() /> } }
    }
}

