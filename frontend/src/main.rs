use common::{CreatePoll, Poll, PollOption, PollOptionId, PollV1, PublicPollId};
use jsonrpc_core_client::transports::wasmhttp;
use sycamore::{
    prelude::*,
    rt::{Event, Reflect},
};
use sycamore_router::{navigate, HistoryIntegration, Route, Router, RouterProps};

pub async fn connect() -> common::ApiClient {
    let (client, receiver_task) = wasmhttp::connect::<common::ApiClient>("http://localhost:3030/")
        .await
        .expect("shouldn't fail");
    wasm_bindgen_futures::spawn_local(receiver_task);
    client
}

#[derive(Route)]
enum AppRoutes {
    #[to("/")]
    CreatePollFonk,
    #[to("/poll/<poll_id>")]
    ViewPoll { poll_id: String },
    #[not_found]
    NotFound,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct EditPollOption {
    id: RcSignal<i32>,
    title: RcSignal<String>,
}

/*{
    let p = create_ref(cx, poll_op_ref2);
    let id = o.id.get();
    let o2 = create_ref(cx, o.title);
    view! { cx,
        li {
            input(bind:value=o2)
            button(on:click=move |_e| p.modify().retain(|e| e.id.get() != id)) { "Remove" }
        }
    }
}*/
#[derive(Prop)]
struct MProp<'a> {
    p: &'a Signal<Vec<EditPollOption>>,
    item: EditPollOption,
}
#[component]
fn InputThong<'a, G: Html>(cx: Scope<'a>, p: MProp<'a>) -> View<G> {
    let flonk = create_ref(cx, p.item.clone());
    let id = *p.item.id.get();
    log::info!("creating new input thong!");

    view! { cx,
        li {
            div(class="field has-addons") {
                div(class="control") {
                    input(class="input", bind:value=flonk.title, placeholder="Option Text")
                }
                div(class="control") {
                    button(class="button is-warning", on:click=move |_e| p.p.modify().retain(|e| *e.id.get() != id)) { "Remove" }
                }
            }
        }
    }
}

#[component]
fn CreatePoll<G: Html>(cx: Scope) -> View<G> {
    let next_id = create_signal(cx, 1i32);
    let new_id = || {
        let id = *next_id.get();
        *next_id.modify() += 1;
        create_rc_signal(id)
    };

    /*create_effect(cx, move ||{
        let sum = sum.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let client = connect().await; // todo: connect only once
            sum.set(client.add(1, 2).await.expect("todo: handle"));
        });
    });*/

    let poll_title = create_signal(cx, String::new());
    let poll_description = create_signal(cx, String::new());

    let poll_options: RcSignal<Vec<EditPollOption>> = create_rc_signal(vec![EditPollOption {
        id: create_rc_signal(1),
        title: create_rc_signal(String::new()),
    }]);
    let poll_op_ref = create_ref(cx, poll_options.clone());

    let add_option = move |_| {
        poll_op_ref.modify().push(EditPollOption {
            id: new_id(),
            title: create_rc_signal(String::new()),
        })
    };
    let poll_options_final: &ReadSignal<Vec<PollOption>> = create_memo(cx, || {
        poll_op_ref
            .get()
            .iter()
            .map(|o| PollOption {
                id: PollOptionId::from_str(format!("{}", *o.id.get())),
                title: o.title.get().to_string(),
                description_text_markdown: "".to_string(),
            })
            .collect()
    });
    let submit_error = create_rc_signal(None);
    let submit_error_ref = create_ref(cx, submit_error.clone());

    let submit_poll = move |_| {
        let submit_error = submit_error.clone();
        if poll_title.get().is_empty() {
            submit_error
                .modify()
                .replace(format!("Poll title must not be empty"));
            return;
        }
        let poll_to_create = CreatePoll {
            title: poll_title.get().to_string(),
            description_text_markdown: poll_description.get().to_string(),
            options: (*poll_options_final.get()).clone(),
        };
        log::info!("creating poll {:#?}", poll_to_create);
        wasm_bindgen_futures::spawn_local(async move {
            let client = connect().await; // todo: connect only once
            log::info!("connected");
            let poll = client.create_poll(poll_to_create).await;
            let poll = match poll {
                Err(e) => {
                    submit_error.modify().replace(format!("Error: {}", e));
                    return;
                }
                Ok(p) => p,
            };
            let poll = match poll {
                Poll::V1(poll) => poll,
            };

            let id = poll.id.to_str();
            navigate(&format!("/poll/{id}"));
        });
    };

    let poll_for_preview = create_memo(cx, || PollV1 {
        id: PublicPollId::from_str("preview".to_string()),
        title: poll_title.get().to_string(),
        description_text_markdown: poll_description.get().to_string(),
        options: (*poll_options_final.get()).clone(),
        votes: vec![],
    });
    /*create_effect(cx, || {
        log::info!("{:#?}", poll_for_preview.get());
    });*/

    view! { cx,
        div {
            h2(class="title is-2") { "Create a poll" }
            div {
                div(class="field") {
                    label(class="label") { "Poll title" }
                    div(class="control") {
                        input(class="input is-primary", bind:value=poll_title)
                    }
                }
                div(class="field") {
                    label(class="label") { "Poll description (markdown)" }
                    div(class="control") {
                        textarea(class="textarea", bind:value=poll_description)
                    }
                }
            }
            "Options:"
            ol {
                Keyed {
                    iterable: poll_op_ref,
                    view: move |cx, o| view! { cx,
                        InputThong { p: poll_op_ref, item: o }
                    },
                    key: |x| x.id.get()
                }
            }
            button(class="button is-secondary", on:click=add_option) {
                "Add option"
            }
            button(class="button is-primary", on:click=submit_poll) {
                "Submit Poll"
            }
            (if let Some(e) = (*submit_error_ref.get()).clone() {
                view! { cx,
                    div(class="notification is-warning") {"Could not submit poll: " (e)} }
            } else {view! {cx, ""}})

            div(class="card is-secondary") {
                div(class="card-header") {
                    div(class="card-header-title") {
                        "Preview:"
                    }
                }
                div(class="card-content") {
                    ChangingViewPoll(poll_for_preview)
                }
            }


        }
    }
}

#[component]
fn ChangingViewPoll<'a, G: Html>(cx: Scope<'a>, poll: &'a ReadSignal<PollV1>) -> View<G> {
    sycamore::view::View::new_dyn(cx, move || ViewPoll(cx, (*poll.get()).clone()))
}

#[component]
async fn LoadViewPoll<G: Html>(cx: Scope<'_>, _poll_id: String) -> View<G> {
    let poll_id = PublicPollId::from_str(_poll_id.to_string());
    let poll = connect().await.get_poll(poll_id).await;
    match poll {
        Ok(poll) => {
            let poll = match poll {
                Poll::V1(p) => p,
            };
            view! { cx,
                ViewPoll(poll)
                a(class="button is-info", href="/") { "Create a new poll" }
            }
        }
        Err(e) => {
            view! { cx,
                div(class="notification is-danger") {
                    "Could not load poll "(_poll_id)": " (e)
                }
            }
        }
    }
}

#[component]
fn ViewPoll<'a, G: Html>(cx: Scope<'a>, poll: PollV1) -> View<G> {
    log::info!("creating new viewpoll");
    let options = View::new_fragment(
        poll.options
            .clone()
            .into_iter()
            .map(|o| {
                view! { cx,
                    tr {
                        td { (o.title) }
                        td { "0 1 2 3 4 5 6 7 8 9" }
                    }
                }
            })
            .collect::<Vec<View<G>>>(),
    );
    view! { cx,
        div(class="poll") {
            h2(class="title is-2") {(poll.title)}
            div(class="subtitle is-3") {(poll.description_text_markdown)}
            (poll.votes.len()) " votes so far"
            table(class="table") {
                thead {
                    tr { td { "Option" } td { "Your vote" } }
                }
                tbody {
                    (options)
                }
            }
            button(class="button is-primary") { "Submit vote" }
        }
    }
}

fn switch<'a, G: Html>(cx: Scope<'a>, route: &'a ReadSignal<AppRoutes>) -> View<G> {
    view! { cx,
        div { (match route.get().as_ref() {
            AppRoutes::ViewPoll { poll_id } => view! { cx,
                LoadViewPoll(poll_id.to_string())
            },
            AppRoutes::CreatePollFonk => view! { cx, CreatePoll() },
            AppRoutes::NotFound => view! { cx, "404 Not Found" },
        }) }
    }
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
    view! { cx,
        section(class="section") {
            div(class="container") {
                h1(class="title is-1") { "Score Voting Tool" }
                Router {
                    integration: HistoryIntegration::new(),
                    view: switch,
                }
            }
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    log::info!("Hello, world!");
    sycamore::render(|cx| {
        view! { cx,
            App {}
        }
    })
}
