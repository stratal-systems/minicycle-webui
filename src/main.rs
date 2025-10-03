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

#[derive(Clone)]
pub enum LazyLoad<T> {
    Present(T),
    Absent(String),
}

impl std::fmt::Display for APIErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            APIErr::Network(msg) => write!(f, "{}", msg),
            APIErr::Decode(msg) => write!(f, "{}", msg),
        }
    }
}

pub async fn get_json<T>(url: String) -> Result<T, APIErr>
where T: Clone + Debug + serde::de::DeserializeOwned + serde::Serialize
{
    TimeoutFuture::new(1000).await;
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
}

pub async fn get_string(url: String) -> Result<String, APIErr>
{
    match Request::get(&url)
        .send()
        .await
        {
            Ok(response) => match response.text().await {
                Ok(parsed) => Ok(parsed),
                Err(err) => Err(APIErr::Decode(err.to_string())),
            },
            Err(err) => Err(APIErr::Network(err.to_string())),
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
        #[prop(into)]
        log_sig: WriteSignal<()>,
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
            <button on:click = move |_| { viewer_sig.set(true); log_sig.write(); }>
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
    log_sig: WriteSignal<()>,
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
    let (log_r, log_w) = signal(());
    let (viewer_sig, set_viewer_sig) = signal(false);
    //let report = LocalResource::new(move || { report_r.get(); get_report_result("http://localhost:8081/foo.json".to_string()) } );
    let report = LocalResource::new(move || {
        if report_r.get() {
            LazyLoad::Present(get_json("http://localhost:8081/foo.json".to_string()))
        } else {
            LazyLoad::Absent("foo".to_string())
        }
    } );
    let log = LocalResource::new(move || { log_r.get(); get_string("http://localhost:8081/log".to_string()) } );



    view! {
        <button on:click=move |_| {
            // change back to "loading..." animation
            // while resouce is loading
            report.set(None);
            report_w.write();
        } >
            "Click me"
        </button>
        
        { 
            move || view! {
                <ReportDisplay report=report.get() viewer_sig=set_viewer_sig log_sig=log_w />
            }
        }

        <p> {viewer_sig} </p>
        { move || view! { <LogViewer content=log.get() /> } }
    }
}

