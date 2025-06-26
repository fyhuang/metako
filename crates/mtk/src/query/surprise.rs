use crate::catalog::Catalog;
use chrono::{DateTime, Utc};

use crate::Entry;
use crate::{FileTree, RepoPathBuf};
use crate::userdata::HistoryDb;

use super::sampler::WeightedGroupSampler;

#[derive(Clone)]
struct WeightInputs {
    days_since_viewed: Option<i64>,
    rating: Option<i64>,
    seconds_since_mod: i64,

    descendant_weight: f64,
}

impl Default for WeightInputs {
    fn default() -> Self {
        Self {
            days_since_viewed: None,
            rating: None,
            seconds_since_mod: 0,

            descendant_weight: 1.0,
        }
    }
}

trait WeightInputsExtractor {
    // TODO: repo_path is redundant with entry.*.repo_path
    fn inputs(&self, repo_path: &RepoPathBuf, entry: &Entry) -> WeightInputs;
}

struct StandardInputs<'a> {
    history_db: &'a HistoryDb,
}

impl WeightInputsExtractor for StandardInputs<'_> {
    fn inputs(&self, _: &RepoPathBuf, entry: &Entry) -> WeightInputs {
        let days = self
            .history_db
            .get(entry.db.id)
            .unwrap()
            .last_viewed_date
            .map(|last_viewed_str| {
                let last_viewed = chrono::DateTime::parse_from_rfc3339(&last_viewed_str).unwrap();
                let last_viewed_utc: DateTime<Utc> = DateTime::from(last_viewed);
                let now = Utc::now();
                now.signed_duration_since(last_viewed_utc).num_days()
            });

        let mod_seconds = {
            let now = Utc::now();
            now.signed_duration_since(entry.fs.mod_time)
                .num_seconds()
        };

        WeightInputs {
            days_since_viewed: days,
            rating: entry.db.rating(),
            seconds_since_mod: mod_seconds,
            descendant_weight: 1.0,
        }
    }
}

type WeightComputer = fn(&WeightInputs) -> f64;

fn surprise_weight(inputs: &WeightInputs) -> f64 {
    let hist_score = if let Some(days_ago) = inputs.days_since_viewed {
        let x = (days_ago as f64) / (720.0 / 2.95);
        let sig = 1.0 / (1.0 + (-x).exp());
        (sig - 0.5) * 2.0
    } else {
        // Never viewed
        1.0
    };

    let rating = inputs.rating.unwrap_or(0);
    let rating_score = 1.0 / (1.0 + std::f64::consts::E.powi((-rating).try_into().expect("rating to i32")));

    (hist_score + rating_score) * inputs.descendant_weight
}

fn recent_mod_weight(inputs: &WeightInputs) -> f64 {
    // Assigns a high weight to recently modified entries, then combines with low weight from surprise_weight
    let x = (inputs.seconds_since_mod as f64) / ((30.0 * 86400.0) / 1.1);
    let sig = 1.0 / (1.0 + (-x).exp());
    let recency_score = ((1.0 - sig) * 2.0) * 50.0;

    (recency_score * inputs.descendant_weight) + surprise_weight(inputs)
}

struct EntrySampler<'a> {
    inputs: &'a dyn WeightInputsExtractor,
    weight_computer: Box<WeightComputer>,
    weighted_sampler: WeightedGroupSampler<RepoPathBuf>,
}

impl EntrySampler<'_> {
    fn new<'a>(
        inputs: &'a dyn WeightInputsExtractor,
        max_entries: usize,
        recency_weight: bool,
    ) -> EntrySampler<'a> {
        let computer = if recency_weight {
            recent_mod_weight as WeightComputer
        } else {
            surprise_weight as WeightComputer
        };

        EntrySampler {
            inputs: inputs,
            weight_computer: Box::new(computer),
            weighted_sampler: WeightedGroupSampler::new(max_entries),
        }
    }

    fn process_entry(&mut self, repo_path: &RepoPathBuf, entry: &Entry) {
        let inputs = self.inputs.inputs(repo_path, entry);
        let weight = (self.weight_computer)(&inputs);
        let parent_path = repo_path.parent().unwrap_or(RepoPathBuf::from("")).to_string();

        self.weighted_sampler.add(
            repo_path.clone(),
            weight,
            parent_path,
        );
    }

    fn get_samples(&mut self) -> Vec<RepoPathBuf> {
        self.weighted_sampler.get_sample()
    }
}

pub fn surprise_entries(
    base_path: &RepoPathBuf,
    file_tree: &FileTree,
    catalog: &mut Catalog,
    history_db: &HistoryDb,
    num_entries: usize,
    recency_weight: bool,
) -> Vec<Entry> {
    let standard_inputs = StandardInputs {
        history_db: history_db,
    };

    let mut sampler = EntrySampler::new(&standard_inputs, num_entries, recency_weight);

    let start = std::time::Instant::now();
    for child in file_tree.list_recursive(base_path).expect("list_recursive") {
        if child.is_metadata_file {
            continue;
        }

        if child.file_type.is_dir {
            continue;
        }

        let db_entry = catalog.get_or_create(&child);
        let entry = Entry {
            fs: child,
            db: db_entry,
        };

        sampler.process_entry(&entry.fs.repo_path, &entry);
    }

    println!("surprise process_entry took {:?}", start.elapsed());

    sampler
        .get_samples()
        .iter()
        .map(|path| {
            let fs_entry = file_tree.get_fs_entry(&path).unwrap();
            let db_entry = catalog.get_or_create(&fs_entry);
            Entry { fs: fs_entry, db: db_entry }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use crate::testing::fake_entry;

    struct FixedInputs(HashMap<&'static str, WeightInputs>);
    impl WeightInputsExtractor for FixedInputs {
        fn inputs(&self, repo_path: &RepoPathBuf, _: &Entry) -> WeightInputs {
            self.0.get(repo_path.0.as_str()).unwrap().clone()
        }
    }

    #[test]
    fn test_surprise_weight_rating() {
        let w_none = surprise_weight(&WeightInputs {
            rating: None,
            ..Default::default()
        });
        let w_low = surprise_weight(&WeightInputs {
            rating: Some(1),
            ..Default::default()
        });
        let w_neg = surprise_weight(&WeightInputs {
            rating: Some(-1),
            ..Default::default()
        });
        assert!(w_low > w_none);
        assert!(w_none > w_neg);
    }

    #[test]
    fn test_surprise_weight_date() {
        let w_never = surprise_weight(&WeightInputs {
            days_since_viewed: None,
            ..Default::default()
        });
        let w_recent = surprise_weight(&WeightInputs {
            days_since_viewed: Some(30),
            ..Default::default()
        });
        assert!(w_never > w_recent);
    }

    #[test]
    fn test_recent_mod_weight_recency_more_important() {
        let w_recent = recent_mod_weight(&WeightInputs {
            seconds_since_mod: 0,
            ..Default::default()
        });
        let w_high_rating_old = recent_mod_weight(&WeightInputs {
            rating: Some(10),
            seconds_since_mod: 30 * 86400,
            ..Default::default()
        });
        assert!(w_recent > w_high_rating_old);
    }

    #[test]
    fn test_sampler_favors_high_scores() {
        let inputs = FixedInputs(
            [
                (
                    "1.mp4",
                    WeightInputs {
                        days_since_viewed: Some(0),
                        rating: Some(-10),
                        ..Default::default()
                    },
                ),
                (
                    "2.mp4",
                    WeightInputs {
                        days_since_viewed: Some(120),
                        rating: Some(10),
                        ..Default::default()
                    },
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        );

        let mut count_1 = 0;
        let mut count_2 = 0;

        for _ in 0..10 {
            let mut sampler = EntrySampler::new(&inputs, 1, false);
            sampler.process_entry(&RepoPathBuf::from("1.mp4"), &fake_entry("1.mp4"));
            sampler.process_entry(&RepoPathBuf::from("2.mp4"), &fake_entry("2.mp4"));

            let result = sampler.get_samples();
            if &result[0] == &RepoPathBuf::from("1.mp4") {
                count_1 += 1;
            } else {
                count_2 += 1;
            }
        }

        assert!(count_1 < count_2);
    }
}
