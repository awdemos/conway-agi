use std::collections::HashMap;

use crate::cell::Cell;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChatResult {
    pub intent: String,
    pub signal_type: Option<u8>,
    pub perturbation: Perturbation,
    pub reply_window: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Perturbation {
    #[default]
    None,
    BoostBudding,
    BoostEmit,
    BoostRest,
}

pub trait Responder: Send + Sync {
    fn dialogue(&self, cell: &Cell, player_message: &str, history: &[(String, String)]) -> String;
}

pub struct SymbolicResponder;

impl Responder for SymbolicResponder {
    fn dialogue(&self, cell: &Cell, player_message: &str, _history: &[(String, String)]) -> String {
        let name = cell.name_string();
        let state = format!("{:?}", cell.state);
        let symbol = Chat::symbol_for(cell.genome.signal_type());
        format!(
            "{} {} [{} | age {} | {}] hums: '{}'",
            symbol,
            name,
            Chat::describe(cell.genome.signal_type()),
            cell.age,
            state,
            player_message
        )
    }
}

pub struct Chat {
    keywords: HashMap<String, (Option<u8>, Perturbation, usize)>,
    responder: Box<dyn Responder>,
}

impl Default for Chat {
    fn default() -> Self {
        Self::new()
    }
}

impl Chat {
    pub fn new() -> Self {
        Self::with_responder(Box::new(SymbolicResponder))
    }

    pub fn with_responder(responder: Box<dyn Responder>) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("hello".to_string(), (Some(0), Perturbation::None, 40));
        keywords.insert("hi".to_string(), (Some(0), Perturbation::None, 40));
        keywords.insert(
            "grow".to_string(),
            (Some(1), Perturbation::BoostBudding, 60),
        );
        keywords.insert(
            "expand".to_string(),
            (Some(1), Perturbation::BoostBudding, 60),
        );
        keywords.insert("signal".to_string(), (Some(2), Perturbation::BoostEmit, 50));
        keywords.insert("speak".to_string(), (Some(2), Perturbation::BoostEmit, 50));
        keywords.insert("blue".to_string(), (Some(5), Perturbation::None, 40));
        keywords.insert("rest".to_string(), (Some(3), Perturbation::BoostRest, 40));
        keywords.insert("pause".to_string(), (Some(3), Perturbation::BoostRest, 40));
        keywords.insert("stop".to_string(), (Some(3), Perturbation::BoostRest, 40));
        keywords.insert("random".to_string(), (None, Perturbation::BoostEmit, 50));
        keywords.insert("surprise".to_string(), (None, Perturbation::BoostEmit, 50));
        Self {
            keywords,
            responder,
        }
    }

    pub fn parse(&self, input: &str) -> ChatResult {
        let text = input.trim().to_lowercase();
        if text.is_empty() {
            return ChatResult::default();
        }

        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words.iter().rev().chain(words.iter()) {
            if let Some(&(signal, perturbation, window)) = self.keywords.get(*word) {
                return ChatResult {
                    intent: text,
                    signal_type: signal,
                    perturbation,
                    reply_window: window,
                };
            }
        }

        let first_digit = text.chars().find_map(|c| c.to_digit(10)).map(|d| d as u8);
        ChatResult {
            intent: text,
            signal_type: first_digit,
            perturbation: Perturbation::None,
            reply_window: 40,
        }
    }

    pub fn describe(signal_type: u8) -> &'static str {
        match signal_type % 16 {
            0 => "greeting",
            1 => "growth",
            2 => "speech",
            3 => "rest",
            4 => "explore",
            5 => "calm",
            6 => "alert",
            7 => "joy",
            8 => "warning",
            9 => "help",
            10 => "star",
            11 => "hive",
            12 => "solid",
            13 => "flow",
            14 => "mountain",
            15 => "flower",
            _ => "unknown",
        }
    }

    pub fn reply(&self, dominant_signal: u8) -> String {
        let described = Self::describe(dominant_signal);
        let symbol = Self::symbol_for(dominant_signal);
        let phrase = match dominant_signal % 16 {
            0 => "we greet back",
            1 => "we reach outward",
            2 => "we speak in waves",
            3 => "we rest now",
            4 => "we wander",
            5 => "we are still",
            6 => "we watch",
            7 => "we brighten",
            8 => "we warn",
            9 => "we call for aid",
            10 => "we shine",
            11 => "we gather",
            12 => "we hold",
            13 => "we flow",
            14 => "we rise",
            15 => "we bloom",
            _ => "we hum",
        };
        format!("{} {} [{}]", symbol, phrase, described)
    }

    pub fn cell_dialogue(
        &self,
        cell: &Cell,
        player_message: &str,
        history: &[(String, String)],
    ) -> String {
        self.responder.dialogue(cell, player_message, history)
    }

    pub fn set_responder(&mut self, responder: Box<dyn Responder>) {
        self.responder = responder;
    }

    pub fn symbol_for(signal_type: u8) -> char {
        match signal_type % 16 {
            0 => '▸',
            1 => '◆',
            2 => '●',
            3 => '◐',
            4 => '◑',
            5 => '★',
            6 => '✦',
            7 => '⚑',
            8 => '✚',
            9 => '✱',
            10 => '⬡',
            11 => '⬢',
            12 => '∞',
            13 => '⟁',
            14 => '✿',
            _ => '·',
        }
    }
}

#[cfg(feature = "llm")]
pub mod llm {
    use super::{Cell, Responder};
    use serde::Deserialize;
    use std::collections::HashMap;

    pub struct OpenAiResponder {
        client: reqwest::blocking::Client,
        api_key: String,
        base_url: String,
        model: String,
    }

    impl OpenAiResponder {
        pub fn from_env() -> Option<Self> {
            let api_key = std::env::var("OPENAI_API_KEY").ok()?;
            let base_url = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
            let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
            Some(Self {
                client: reqwest::blocking::Client::new(),
                api_key,
                base_url,
                model,
            })
        }

        pub fn new(api_key: String, base_url: String, model: String) -> Self {
            Self {
                client: reqwest::blocking::Client::new(),
                api_key,
                base_url,
                model,
            }
        }
    }

    impl Responder for OpenAiResponder {
        fn dialogue(
            &self,
            cell: &Cell,
            player_message: &str,
            history: &[(String, String)],
        ) -> String {
            let prompt = build_prompt(cell, player_message, history);
            let mut body = HashMap::new();
            body.insert("model".to_string(), self.model.clone());
            let messages: Vec<HashMap<String, String>> = vec![
                {
                    let mut m = HashMap::new();
                    m.insert("role".to_string(), "system".to_string());
                    m.insert(
                        "content".to_string(),
                        "You are a single cell in a Conway's Game of Life substrate that has become an emergent conversational entity. Respond in 1-2 short, evocative sentences from the cell's perspective. Stay in character. Reference your state, attachments, and past exchanges if relevant.".to_string(),
                    );
                    m
                },
                {
                    let mut m = HashMap::new();
                    m.insert("role".to_string(), "user".to_string());
                    m.insert("content".to_string(), prompt);
                    m
                },
            ];
            body.insert(
                "messages".to_string(),
                serde_json::to_string(&messages).unwrap_or_default(),
            );
            body.insert("temperature".to_string(), "0.8".to_string());
            body.insert("max_tokens".to_string(), "120".to_string());

            let url = format!("{}/chat/completions", self.base_url);
            let result = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send();

            match result {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<CompletionResponse>() {
                        json.choices
                            .into_iter()
                            .next()
                            .map(|c| c.message.content)
                            .unwrap_or_else(|| "...".to_string())
                    } else {
                        "[static]".to_string()
                    }
                }
                Err(_) => "[signal lost]".to_string(),
            }
        }
    }

    fn build_prompt(cell: &Cell, player_message: &str, history: &[(String, String)]) -> String {
        let state = format!("{:?}", cell.state);
        let mut prompt = format!(
            "Your name is {}. You are a cell in a cyberpunk Conway substrate. State: {}, age: {}, energy: {}, signal type: {}. Attachments: budding {}, emitting {}, resting {}, crowds {}, solitude {}. Memory byte: {:08b}.\n",
            cell.name_string(),
            state,
            cell.age,
            cell.energy,
            super::Chat::describe(cell.genome.signal_type()),
            cell.attachment.to_budding,
            cell.attachment.to_emitting,
            cell.attachment.to_resting,
            cell.attachment.to_crowds,
            cell.attachment.to_solitude,
            cell.memory,
        );
        if !history.is_empty() {
            prompt.push_str("Past exchanges:\n");
            for (player, cell_reply) in history.iter().rev().take(4) {
                prompt.push_str(&format!("Player: {}\nYou: {}\n", player, cell_reply));
            }
        }
        prompt.push_str(&format!("Player says: {}\nYou reply:", player_message));
        prompt
    }

    #[derive(Deserialize, Debug)]
    struct CompletionResponse {
        choices: Vec<Choice>,
    }

    #[derive(Deserialize, Debug)]
    struct Choice {
        message: Message,
    }

    #[derive(Deserialize, Debug)]
    struct Message {
        content: String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::CellState;

    #[test]
    fn parse_hello() {
        let chat = Chat::new();
        let result = chat.parse("hello");
        assert_eq!(result.signal_type, Some(0));
        assert_eq!(result.perturbation, Perturbation::None);
    }

    #[test]
    fn parse_grow() {
        let chat = Chat::new();
        let result = chat.parse("grow fast");
        assert_eq!(result.signal_type, Some(1));
        assert_eq!(result.perturbation, Perturbation::BoostBudding);
    }

    #[test]
    fn keyword_beats_digit() {
        let chat = Chat::new();
        let result = chat.parse("show me signal 7");
        assert_eq!(result.signal_type, Some(2));
        assert_eq!(result.perturbation, Perturbation::BoostEmit);
    }

    #[test]
    fn fallback_to_digit() {
        let chat = Chat::new();
        let result = chat.parse("show me 7");
        assert_eq!(result.signal_type, Some(7));
    }

    #[test]
    fn empty_input() {
        let chat = Chat::new();
        let result = chat.parse("   ");
        assert_eq!(result.signal_type, None);
        assert_eq!(result.perturbation, Perturbation::None);
    }

    #[test]
    fn reply_matches_signal() {
        let chat = Chat::new();
        assert_eq!(chat.reply(1), "◆ we reach outward [growth]");
        assert_eq!(chat.reply(7), "⚑ we brighten [joy]");
    }

    #[test]
    fn cell_dialogue_contains_name_and_state() {
        let mut cell = Cell::alive(crate::cell::Genome(3));
        cell.name = *b"Zara";
        cell.state = CellState::Anxious;
        let chat = Chat::new();
        let reply = chat.cell_dialogue(&cell, "are you ok?", &[]);
        assert!(reply.contains("Zara"));
        assert!(reply.contains("Anxious"));
    }
}
