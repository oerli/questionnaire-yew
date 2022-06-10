use yew::prelude::*;
use patternfly_yew::*;
use reqwasm::http::Request;
use serde_json::json;
use gloo::storage::{LocalStorage, Storage};

use super::view_question_form::ViewQuestionForm;
use super::{Question, Session};
use crate::answer::{Vote, Answer, self};

use crate::{API_URL, VOTE_KEY};

pub enum Msg {
    LoadQuestions(Vec<Question>),
    ChangeVotes(Question),
    Submit,
    ReceiveSession(Session),
}


#[derive(Clone, Properties, PartialEq)]
pub struct ViewQuestionsProps {
    pub session: String,
}

pub struct ViewQuestions {
    questions: Vec<Question>,
    session: Session,
    vote_key: Option<Session>,
}

impl Component for ViewQuestions {
    type Message = Msg;
    type Properties = ViewQuestionsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let session = ctx.props().session.clone();
        ctx.link().send_future(async move {
            match Request::get(&format!("{}/question/{}", API_URL, session)).send().await {
                Ok(r) => Msg::LoadQuestions(r.json().await.unwrap()),
                Err(_) => todo!()
            }
        });
        ViewQuestions {
            questions: vec![],
            session: Session{session: ctx.props().session.clone()},
            vote_key: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let question_list: Html = self.questions.iter().map(|question| {
            html! {
                <StackItem fill=true>
                    <Card selected=true selectable=true>
                        <Form>
                            <ViewQuestionForm question={question.clone()} on_change_vote={ctx.link().callback(Msg::ChangeVotes)}/>
                        </Form>
                    </Card>
                    <br/>
                </StackItem>
                
            }
        }).collect();

        let onclick_submit = ctx.link().callback(|_| Msg::Submit);

        let vote_key = match &self.vote_key {
            Some(s) => {html!(
                
                <PopoverPopup orientation={Orientation::Bottom} header={html!(<Title level={Level::H2}>{"Vote"}</Title>)}>
                    {s.session.clone()}
                </PopoverPopup>
            )},
            None => html!()
        };

        html! {
            <>
                {question_list}
                <StackItem>
                    <Button icon={Icon::CheckCircle} label="Submit" variant={Variant::Primary} onclick={onclick_submit}/>
                </StackItem>
                {vote_key}
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadQuestions(q) => {
                self.questions = q;

                for question in &mut self.questions {
                    for answer in &mut question.answers {
                        match &answer.vote {
                            Some(a) => (),
                            None => answer.vote = Some(Vote { vote: "false".to_owned(), answer_key: Some(answer.key.clone()), question_key: Some(question.key.clone())}),
                        }
                    }
                }
                true
            },
            Msg::ChangeVotes(question) => {
                for (index, q) in self.questions.iter().enumerate() {
                    if q.key == question.key {
                        let _ = std::mem::replace(&mut self.questions[index], question);
                        LocalStorage::set(VOTE_KEY, &self.questions).unwrap();
                        return true;
                    }
                }
                false
            },
            Msg::Submit => {
                let mut votes: Vec<Vote> = Vec::new();
                for q in &self.questions {
                    for a in &q.answers {
                        match &a.vote {
                            Some(v) => votes.push(v.clone()),
                            None => (),
                        }
                    }
                }

                let session = self.session.session.clone();
                let payload = json!(votes).to_string();
                ctx.link().send_future(async move {
                    //TODO: .json should be used, wait for reqwasm update, serde_json can be removed afterwards
                    match Request::post(&format!("{}/vote/{}", API_URL, session)).header("Content-Type", "application/json").body(payload).send().await {
                        Ok(r) => {
                            Msg::ReceiveSession(r.json().await.unwrap())
                        },
                        Err(e) => {
                            log::debug!("{:?}", e);
                            todo!()
                        }
                    }
                });
                true
            },
            Msg::ReceiveSession(s) => {
                self.vote_key = Some(s);
                true
            }
        }
    }
}
