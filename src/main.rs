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

#[derive(Clone)]
pub enum APIErr {
    Network(String),
    Decode(String),
    Lazy(String),
}

impl std::fmt::Display for APIErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            APIErr::Network(msg) => write!(f, "{}", msg),
            APIErr::Decode(msg) => write!(f, "{}", msg),
            APIErr::Lazy(msg) => write!(f, "{}", msg),
        }
    }
}


#[derive(Clone)]
pub enum LazyLoad<T> {
    Present(T),
    Absent(String),
}

pub async fn get_json<T>(do_it: bool, url: String) -> Result<T, APIErr>
where T: Clone + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    if do_it {
        TimeoutFuture::new(1000).await;
        match Request::get(&url).send().await {
            Ok(response) => match response.json::<T>().await {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => Err(APIErr::Decode(err.to_string())),
            },
            Err(err) => Err(APIErr::Network(err.to_string())),
        }
    } else {
        Err(APIErr::Lazy("Not yet loaded".to_string()))
    }
}

pub async fn get_string(do_it: bool, url: String) -> Result<String, APIErr>
{
    if do_it {
        TimeoutFuture::new(1000).await;
        match Request::get(&url).send().await {
            Ok(response) => match response.text().await {
                    Ok(parsed) => Ok(parsed),
                    Err(err) => Err(APIErr::Decode(err.to_string())),
            },
            Err(err) => Err(APIErr::Network(err.to_string())),
        }
    } else {
        Err(APIErr::Lazy("Not yet loaded".to_string()))
    }
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
            APIErr::Network(ref msg) => format!("Network err: {}", msg),
            APIErr::Decode(ref msg) => format!("Decode err: {}", msg),
            APIErr::Lazy(_) => "...".to_string(),
        } } </b>
        //<p> { error.to_string() } </p>
    }
}

#[component]
fn ResultLoaded(
        report: Report,
        #[prop(into)]
        viewer_sig: WriteSignal<bool>,
        #[prop(into)]
        log_sig: WriteSignal<bool>,
    ) -> impl IntoView {
    // TODO double match on report.finish? Is this ok?
    view! {
        <ul>
            <li> "Status: " { match report.finish {
                Some(ref finish) => if finish.ok {
                    view!{ <code>"OK"</code> }.into_any()
                    } else {
                    view!{ <code>"ERR"</code> }.into_any()
                },
                None => view!{ "in progress... " }.into_any(),
                }
            } </li>
            <li> "Started: " <VersaTime unixtime=report.start.time /> </li>
            <li> "Run time: " { match report.finish {
                Some(ref finish) => view!{
                    // TODO generally fix all of these dangerous weird
                    // i64 u64 casts
                    <VersaTimeDelta seconds=finish.time as i64 - report.start.time as i64/>
                }.into_any(),
                None => view!{ "in progress... " }.into_any(),
                }
            } </li>
            <li> "Commit message: " { report.message } </li>
            <li> "Commit ref: " { report.r#ref } </li>
            <li> "Artifact ID: " { report.artifacts } </li>
            <a href="#" on:click = move |_| { viewer_sig.set(true); log_sig.set(true); }>
                "View"
            </a>

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
fn VersaTimeDelta(seconds: i64) -> impl IntoView {

    let systime = chrono::Duration::seconds(seconds);
    let humantime = chrono_humanize::HumanTime::from(systime);
    // TODO better to use to_rfc3339?

    view! {
        <div class="versatimedelta">
            <div class="wrap">
                <div class="human"> <div class="wrap"> {
                    humantime.to_text_en(
                        chrono_humanize::Accuracy::Precise,
                        chrono_humanize::Tense::Present,
                    )
                } </div> </div>
                <div class="seconds"> <div class="wrap"> { format!("{}", seconds) } </div></div>
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
    #[prop(into)]
    log_sig: WriteSignal<bool>,
    ) -> impl IntoView {
    match report {
        None => view! { <ResultLoading /> }.into_any(),
        Some(result) => match result {
            Ok(report) => view! { <ResultLoaded report=report viewer_sig=viewer_sig log_sig=log_sig /> }.into_any(),
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
fn LogViewer(content: Option<Result<String, APIErr>>) -> impl IntoView {
    match content {
        None => view! { "Loading..." }.into_any(),
        Some(what) => match what {
            Ok(string) => view! { <pre>{string}</pre> }.into_any(),
            Err(err) => view! { <ErrorDisplay error=err /> }.into_any(),
        },
    }
}

#[component]
fn App() -> impl IntoView {
    let (report_r, report_w) = signal(false);
    let (log_r, log_w) = signal(false);
    let (viewer_sig, set_viewer_sig) = signal(false);
    //let report = LocalResource::new(move || { report_r.get(); get_report_result("http://localhost:8081/foo.json".to_string()) } );
    let report = LocalResource::new(move || {
        get_json(report_r.get(), "http://localhost:8081/foo.json".to_string())
    } );
    let log = LocalResource::new(move || { 
        get_string(log_r.get(), "http://localhost:8081/log".to_string())
    } );



    view! {
        <button on:click=move |_| {
            // change back to "loading..." animation
            // while resouce is loading
            report.set(None);
            report_w.set(true);
        } >
            "Click to load latest report"
        </button>
        
        { 
            move || view! {
                <ReportDisplay report=report.get() viewer_sig=set_viewer_sig log_sig=log_w />
            }
        }

        { move || view! { <LogViewer content=log.get() /> } }

        <hr />
        <p>
            "I am "
            { env!("CARGO_PKG_NAME") }
            "version "
            { env!("CARGO_PKG_VERSION") }
            " licensed under "
            { env!("CARGO_PKG_LICENSE") }
            ". "
            "Get my source code at "
            <a href={ env!("CARGO_PKG_REPOSITORY") }>
                { env!("CARGO_PKG_REPOSITORY") }
            </a>
            ". "
        </p>
    }
}

