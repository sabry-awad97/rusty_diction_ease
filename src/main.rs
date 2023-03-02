use std::collections::HashMap;
use std::io::{self, BufRead};

use difflib::get_close_matches;

const JSON_DATA: &str = include_str!("data/english_english.json");

type Definitions = Vec<String>;

#[derive(Debug)]
enum DictionaryError {
    NotFound,
    IncorrectWord(String),
    UnknownInput,
}

impl std::fmt::Display for DictionaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictionaryError::NotFound => write!(f, "The word doesn't exist."),
            DictionaryError::IncorrectWord(correct_word) => {
                write!(f, "Did you mean {} instead?", correct_word)
            }
            DictionaryError::UnknownInput => write!(f, "We didn't understand your input."),
        }
    }
}

#[derive(Debug)]
enum UserResponse {
    Yes,
    No,
    Unknown,
}

impl UserResponse {
    fn from_str(input: &str) -> Self {
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => UserResponse::Yes,
            "n" | "no" => UserResponse::No,
            _ => UserResponse::Unknown,
        }
    }
}

#[derive(Debug)]
struct Dictionary {
    data: HashMap<String, Definitions>,
}

impl Dictionary {
    fn from_json(json_data: &str) -> Result<Self, serde_json::Error> {
        let data: HashMap<String, Definitions> = serde_json::from_str(json_data)?;
        Ok(Dictionary { data })
    }

    fn lookup(&self, word: &str) -> Result<Definitions, DictionaryError> {
        let word = word.trim().to_lowercase();
        match self.data.get(&word) {
            Some(defns) => Ok(defns.clone()),
            None => {
                let choices = self.data.keys().map(|key| key.as_str()).collect();
                match get_close_matches(&word, choices, 1, 0.8).first() {
                    Some(close_word) => match self.confirm_word(close_word)? {
                        UserResponse::Yes => self.lookup(close_word),
                        UserResponse::No => Err(DictionaryError::NotFound),
                        UserResponse::Unknown => Err(DictionaryError::UnknownInput),
                    },
                    None => Err(DictionaryError::NotFound),
                }
            }
        }
    }

    fn confirm_word(&self, word: &str) -> Result<UserResponse, DictionaryError> {
        let mut input = String::new();
        println!("Did you mean {}? (Y/N)", word);
        io::stdin().lock().read_line(&mut input).unwrap();

        match UserResponse::from_str(&input) {
            UserResponse::Yes | UserResponse::No => Ok(UserResponse::from_str(&input)),
            UserResponse::Unknown => Err(DictionaryError::UnknownInput),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dictionary = Dictionary::from_json(JSON_DATA)?;

    loop {
        let mut input = String::new();
        println!("Enter a word to look up (or 'exit' to quit):");
        io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input == "exit" {
            break;
        }

        match dictionary.lookup(&input) {
            Ok(defns) => {
                println!("Definitions:");
                for defn in defns {
                    println!("- {}", defn);
                }
            }
            Err(DictionaryError::NotFound) => {
                println!(
                    "Sorry, the word '{}' was not found in the dictionary.",
                    input
                );
            }
            Err(DictionaryError::IncorrectWord(correct_word)) => {
                println!(
                    "{}",
                    DictionaryError::IncorrectWord(correct_word.to_owned())
                );
                match dictionary.lookup(&correct_word) {
                    Ok(defns) => {
                        println!("Definitions:");
                        for defn in defns {
                            println!("- {}", defn);
                        }
                    }
                    Err(_) => (),
                }
            }
            Err(DictionaryError::UnknownInput) => {
                println!("{}", DictionaryError::UnknownInput);
            }
        }
    }
    Ok(())
}
