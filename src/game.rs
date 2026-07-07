use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::LazyLock;
use std::time::{Duration, Instant};
use indexmap::IndexMap;

static WORDS: LazyLock<WordCatalogue> = LazyLock::new(|| {
    #[derive(Deserialize)]
    struct Category {
        description: String,
        colors: Colors,
        words: HashMap<String, WordEntry>,
    }

    #[derive(Deserialize)]
    struct WordEntry {
        difficulty: u8,
        exclusions: Vec<String>,
    }

    let categories = toml::from_slice::<IndexMap<String, Category>>(include_bytes!("words.toml"))
        .expect("static word catalogue loading failure");

    let mut word_idx = 0;
    WordCatalogue {
        words: categories
            .into_values()
            .map(|category| {
                (
                    category.description,
                    category.colors,
                    category
                        .words
                        .into_iter()
                        .map(|(word, entry)| Word {
                            number: {
                                word_idx += 1;
                                word_idx
                            },
                            word,
                            difficulty: entry.difficulty,
                            exclusions: entry.exclusions,
                        })
                        .collect(),
                )
            })
            .collect(),
    }
});

#[derive(Clone, Debug)]
pub struct WordCatalogue {
    words: VecDeque<(String, Colors, Vec<Word>)>,
}

impl WordCatalogue {
    fn random() -> Self {
        let mut catalogue = WORDS.clone();
        let mut rng = rand::rng();
        catalogue.words.make_contiguous().shuffle(&mut rng);
        for category in catalogue.words.iter_mut().map(|(_, _, words)| words) {
            category.shuffle(&mut rng);
        }

        catalogue
    }
}

impl Iterator for WordCatalogue {
    type Item = (String, Colors, Word);

    fn next(&mut self) -> Option<Self::Item> {
        let next = (
            self.words[0].0.clone(),
            self.words[0].1.clone(),
            self.words[0].2.pop().expect("empty word list"),
        );
        if self.words[0].2.is_empty() {
            assert!(self.words.pop_front().is_some());
        }

        if self.words.is_empty() {
            *self = WordCatalogue::random();
        } else {
            self.words.rotate_left(1);
        }

        Some(next)
    }
}

#[derive(Clone, Debug)]
pub struct Word {
    pub number: u32,
    pub word: String,
    pub difficulty: u8,
    pub exclusions: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Colors {
    background: String,
    border: String,
    word: String,
    header: String,
    exclusion: String,
    dot: String,
}

#[derive(Debug)]
pub struct Game {
    pub last_update: Instant,
    pub word_catalogue: WordCatalogue,
}

impl Game {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            word_catalogue: WordCatalogue::random(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.last_update.elapsed() > Duration::from_hours(6)
    }
}
