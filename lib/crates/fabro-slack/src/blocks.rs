use fabro_interview::Question;
use fabro_types::QuestionType;
use serde_json::{Value, json};

use crate::payload::{SlackActionPayload, SlackGateDecision, encode_action_value};

const ANSWER_ACTION_ID: &str = "interview.answer";
const GATE_DECISION_ACTION_ID: &str = "gate.decision";
const MULTI_SELECT_BLOCK_ID: &str = "interview.checkboxes";
const MULTI_SELECT_ACTION_ID: &str = "interview.select";
const MULTI_SELECT_SUBMIT_ACTION_ID: &str = "interview.submit";

fn text_block(text: &str) -> Value {
    json!({
        "type": "section",
        "text": {
            "type": "mrkdwn",
            "text": text
        }
    })
}

fn context_blocks(question: &Question) -> Vec<Value> {
    let Some(context) = question
        .context_display
        .as_deref()
        .map(str::trim)
        .filter(|context| !context.is_empty())
    else {
        return vec![text_block(&question.text)];
    };

    vec![
        text_block(&question.text),
        text_block(&format!(
            "*Context*\n{}",
            truncate_for_slack_section(context, 2988)
        )),
    ]
}

fn truncate_for_slack_section(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    if max_chars <= 3 {
        return ".".repeat(max_chars);
    }
    let mut truncated: String = text.chars().take(max_chars - 3).collect();
    truncated.push_str("...");
    truncated
}

fn button(label: &str, value: &str, action_id: &str) -> Value {
    json!({
        "type": "button",
        "text": {
            "type": "plain_text",
            "text": label
        },
        "value": value,
        "action_id": action_id
    })
}

fn indexed_action_id(base: &str, index: usize) -> String {
    format!("{base}.{index}")
}

pub fn answered_blocks(question_text: &str, answer_text: &str) -> Vec<Value> {
    vec![text_block(&format!(
        "~{question_text}~\n*Answer:* {answer_text}"
    ))]
}

pub fn question_to_blocks(run_id: &str, question_id: &str, question: &Question) -> Vec<Value> {
    match question.question_type {
        QuestionType::YesNo | QuestionType::Confirmation => {
            let actions = json!({
                "type": "actions",
                "elements": [
                    button("Yes", &encode_action_value(&SlackActionPayload::Yes {
                        run_id: run_id.to_string(),
                        qid: question_id.to_string(),
                    }), &indexed_action_id(ANSWER_ACTION_ID, 0)),
                    button("No", &encode_action_value(&SlackActionPayload::No {
                        run_id: run_id.to_string(),
                        qid: question_id.to_string(),
                    }), &indexed_action_id(ANSWER_ACTION_ID, 1)),
                ]
            });
            let mut blocks = context_blocks(question);
            blocks.push(actions);
            blocks
        }
        QuestionType::MultipleChoice => {
            let elements: Vec<Value> = question
                .options
                .iter()
                .enumerate()
                .map(|(index, opt)| {
                    let (value, action_id) =
                        if let Some(decision) = gate_decision_for_label(&opt.label) {
                            (
                                encode_action_value(&SlackActionPayload::GateDecision {
                                    run_id: run_id.to_string(),
                                    gate_id: question_id.to_string(),
                                    decision,
                                }),
                                indexed_action_id(GATE_DECISION_ACTION_ID, index),
                            )
                        } else {
                            (
                                encode_action_value(&SlackActionPayload::Selected {
                                    run_id: run_id.to_string(),
                                    qid: question_id.to_string(),
                                    key: opt.key.clone(),
                                }),
                                indexed_action_id(ANSWER_ACTION_ID, index),
                            )
                        };
                    button(&opt.label, &value, &action_id)
                })
                .collect();
            let actions = json!({
                "type": "actions",
                "elements": elements
            });
            let mut blocks = context_blocks(question);
            blocks.push(actions);
            blocks
        }
        QuestionType::MultiSelect => {
            let options: Vec<Value> = question
                .options
                .iter()
                .map(|opt| {
                    json!({
                        "text": { "type": "plain_text", "text": opt.label },
                        "value": opt.key
                    })
                })
                .collect();
            let checkboxes = json!({
                "type": "actions",
                "block_id": MULTI_SELECT_BLOCK_ID,
                "elements": [{
                    "type": "checkboxes",
                    "action_id": MULTI_SELECT_ACTION_ID,
                    "options": options
                }]
            });
            let submit = json!({
                "type": "actions",
                "elements": [
                    button("Submit", &encode_action_value(&SlackActionPayload::SubmitMulti {
                        run_id: run_id.to_string(),
                        qid: question_id.to_string(),
                    }), MULTI_SELECT_SUBMIT_ACTION_ID),
                ]
            });
            let mut blocks = context_blocks(question);
            blocks.push(checkboxes);
            blocks.push(submit);
            blocks
        }
        QuestionType::Freeform => {
            let mut blocks = vec![text_block(&format!(
                "{}\n_Please reply in thread (mention me with your answer)._",
                question.text
            ))];
            blocks.extend(context_blocks(question).into_iter().skip(1));
            blocks
        }
    }
}

fn gate_decision_for_label(label: &str) -> Option<SlackGateDecision> {
    let label = label.to_ascii_lowercase();
    if label.contains("approve") || label.contains("approved") {
        Some(SlackGateDecision::Approve)
    } else if label.contains("reject") || label.contains("revise") || label.contains("revision") {
        Some(SlackGateDecision::Reject)
    } else if label.contains("edit") {
        Some(SlackGateDecision::Edit)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use fabro_types::InterviewOption;

    use super::*;

    #[test]
    fn yes_no_produces_two_buttons() {
        let q = Question::new("Approve this PR?", QuestionType::YesNo);
        let blocks = question_to_blocks("run-1", "q-1", &q);
        let blocks_json: Value = serde_json::to_value(&blocks).unwrap();

        let section = &blocks_json[0];
        assert_eq!(section["type"], "section");
        assert!(
            section["text"]["text"]
                .as_str()
                .unwrap()
                .contains("Approve this PR?")
        );

        let actions = &blocks_json[1];
        assert_eq!(actions["type"], "actions");
        let elements = actions["elements"].as_array().unwrap();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0]["text"]["text"], "Yes");
        assert_eq!(elements[1]["text"]["text"], "No");
    }

    #[test]
    fn confirmation_produces_two_buttons() {
        let q = Question::new("Continue?", QuestionType::Confirmation);
        let blocks = question_to_blocks("run-1", "q-2", &q);
        let blocks_json: Value = serde_json::to_value(&blocks).unwrap();

        let actions = &blocks_json[1];
        let elements = actions["elements"].as_array().unwrap();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0]["text"]["text"], "Yes");
        assert_eq!(elements[1]["text"]["text"], "No");
    }

    #[test]
    fn multiple_choice_produces_button_per_option() {
        let mut q = Question::new("Pick a language:", QuestionType::MultipleChoice);
        q.options = vec![
            InterviewOption {
                key: "rs".to_string(),
                label: "Rust".to_string(),
            },
            InterviewOption {
                key: "ts".to_string(),
                label: "TypeScript".to_string(),
            },
            InterviewOption {
                key: "py".to_string(),
                label: "Python".to_string(),
            },
        ];
        let blocks = question_to_blocks("run-1", "q-3", &q);
        let blocks_json: Value = serde_json::to_value(&blocks).unwrap();

        let actions = &blocks_json[1];
        let elements = actions["elements"].as_array().unwrap();
        assert_eq!(elements.len(), 3);
        assert_eq!(elements[0]["text"]["text"], "Rust");
        assert_eq!(
            elements[0]["action_id"],
            indexed_action_id(ANSWER_ACTION_ID, 0)
        );
        assert!(
            elements[0]["value"]
                .as_str()
                .unwrap()
                .contains("\"run_id\":\"run-1\"")
        );
        assert_eq!(elements[1]["text"]["text"], "TypeScript");
        assert_eq!(elements[2]["text"]["text"], "Python");
    }

    #[test]
    fn approval_choice_uses_gate_decision_payload() {
        let mut q = Question::new("Approve plan?", QuestionType::MultipleChoice);
        q.options = vec![
            InterviewOption {
                key: "A".to_string(),
                label: "[A] Approve".to_string(),
            },
            InterviewOption {
                key: "R".to_string(),
                label: "[R] Revise".to_string(),
            },
        ];
        let blocks = question_to_blocks("run-1", "gate-1", &q);
        let blocks_json: Value = serde_json::to_value(&blocks).unwrap();
        let elements = blocks_json[1]["elements"].as_array().unwrap();

        assert_eq!(
            elements[0]["action_id"],
            indexed_action_id(GATE_DECISION_ACTION_ID, 0)
        );
        assert_eq!(
            elements[1]["action_id"],
            indexed_action_id(GATE_DECISION_ACTION_ID, 1)
        );
        assert!(
            elements[0]["value"]
                .as_str()
                .unwrap()
                .contains("\"decision\":\"approve\"")
        );
        assert!(
            elements[1]["value"]
                .as_str()
                .unwrap()
                .contains("\"decision\":\"reject\"")
        );
    }

    #[test]
    fn approval_choice_includes_context_display_before_actions() {
        let mut q = Question::new("Approve plan?", QuestionType::MultipleChoice);
        q.context_display = Some("Generated plan body".to_string());
        q.options = vec![InterviewOption {
            key: "A".to_string(),
            label: "[A] Approve".to_string(),
        }];

        let blocks = question_to_blocks("run-1", "gate-1", &q);
        let blocks_json: Value = serde_json::to_value(&blocks).unwrap();

        assert_eq!(blocks_json.as_array().unwrap().len(), 3);
        assert_eq!(blocks_json[1]["type"], "section");
        assert!(
            blocks_json[1]["text"]["text"]
                .as_str()
                .unwrap()
                .contains("Generated plan body")
        );
        assert_eq!(blocks_json[2]["type"], "actions");
    }

    #[test]
    fn freeform_produces_section_prompting_thread_reply() {
        let q = Question::new("What's the repo URL?", QuestionType::Freeform);
        let blocks = question_to_blocks("run-1", "q-4", &q);
        let blocks_json: Value = serde_json::to_value(&blocks).unwrap();

        assert_eq!(blocks_json.as_array().unwrap().len(), 1);
        let text = blocks_json[0]["text"]["text"].as_str().unwrap();
        assert!(text.contains("What's the repo URL?"));
        assert!(text.contains("reply in thread"));
        assert!(text.contains("mention me"));
    }

    #[test]
    fn action_values_include_run_id_and_question_id() {
        let q = Question::new("Approve?", QuestionType::YesNo);
        let blocks = question_to_blocks("run-7", "q-7", &q);
        let blocks_json: Value = serde_json::to_value(&blocks).unwrap();

        let actions = &blocks_json[1];
        let elements = actions["elements"].as_array().unwrap();
        assert_eq!(
            elements[0]["action_id"],
            indexed_action_id(ANSWER_ACTION_ID, 0)
        );
        let value = elements[0]["value"].as_str().unwrap();
        assert!(value.contains("\"run_id\":\"run-7\""));
        assert!(value.contains("\"qid\":\"q-7\""));
    }

    #[test]
    fn answered_blocks_show_question_and_answer() {
        let blocks = answered_blocks("Do you approve?", "Yes");
        let json: Value = serde_json::to_value(&blocks).unwrap();

        assert_eq!(json.as_array().unwrap().len(), 1);
        let text = json[0]["text"]["text"].as_str().unwrap();
        assert!(text.contains("Do you approve?"));
        assert!(text.contains("Yes"));
    }

    #[test]
    fn answered_blocks_have_no_actions() {
        let blocks = answered_blocks("Pick one:", "Rust");
        let json: Value = serde_json::to_value(&blocks).unwrap();

        let has_actions = json
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["type"] == "actions");
        assert!(!has_actions);
    }

    #[test]
    fn multi_select_produces_checkboxes_and_submit_button() {
        let mut q = Question::new("Select features:", QuestionType::MultiSelect);
        q.options = vec![
            InterviewOption {
                key: "a".to_string(),
                label: "Auth".to_string(),
            },
            InterviewOption {
                key: "b".to_string(),
                label: "Billing".to_string(),
            },
        ];
        let blocks = question_to_blocks("run-1", "q-5", &q);
        let blocks_json: Value = serde_json::to_value(&blocks).unwrap();

        // Checkboxes in their own block with a block_id
        let checkbox_block = &blocks_json[1];
        assert_eq!(checkbox_block["type"], "actions");
        assert_eq!(checkbox_block["block_id"], MULTI_SELECT_BLOCK_ID);
        let cb_elements = checkbox_block["elements"].as_array().unwrap();
        assert_eq!(cb_elements[0]["type"], "checkboxes");
        assert_eq!(cb_elements[0]["action_id"], MULTI_SELECT_ACTION_ID);

        // Submit button in a separate actions block
        let submit_block = &blocks_json[2];
        assert_eq!(submit_block["type"], "actions");
        let submit_elements = submit_block["elements"].as_array().unwrap();
        assert_eq!(submit_elements[0]["type"], "button");
        assert_eq!(submit_elements[0]["text"]["text"], "Submit");
        assert_eq!(
            submit_elements[0]["action_id"],
            MULTI_SELECT_SUBMIT_ACTION_ID
        );
        assert!(
            submit_elements[0]["value"]
                .as_str()
                .unwrap()
                .contains("\"qid\":\"q-5\"")
        );
    }
}
