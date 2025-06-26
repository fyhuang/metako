use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct KeyedItem<T> {
    item: T,
    group_id: String,
    key: f64,
}

struct ReservoirStats {
    min_item_idx: usize,
    min_item_key: f64,

    min_group_item_idx: Option<usize>,
    min_group_item_key: Option<f64>,
}

#[derive(Debug)]
struct Reservoir<T> {
    items: Vec<KeyedItem<T>>,
    group_counts: HashMap<String, usize>,
}

impl<T: Clone> Reservoir<T> {
    fn add(&mut self, keyed_item: KeyedItem<T>) {
        if let Some(count) = self.group_counts.get_mut(&keyed_item.group_id) {
            *count += 1;
        } else {
            self.group_counts.insert(keyed_item.group_id.clone(), 1);
        }
        self.items.push(keyed_item);
    }

    fn replace(&mut self, idx: usize, keyed_item: KeyedItem<T>) {
        let old_group_id = self.items[idx].group_id.clone();
        *self
            .group_counts
            .get_mut(&old_group_id)
            .expect("old group should exist") -= 1;

        if let Some(count) = self.group_counts.get_mut(&keyed_item.group_id) {
            *count += 1;
        } else {
            self.group_counts.insert(keyed_item.group_id.clone(), 1);
        }
        self.items[idx] = keyed_item;
    }

    fn stats_for_group(&self, group_id: &str) -> ReservoirStats {
        assert!(
            self.items.len() > 0,
            "Reservoir must have at least one item"
        );

        // Make one pass through the reservoir to gather needed stats
        let mut stats = ReservoirStats {
            min_item_idx: 0,
            min_item_key: f64::INFINITY,

            min_group_item_idx: None,
            min_group_item_key: None,
        };

        for (idx, existing) in self.items.iter().enumerate() {
            // Check if this is the min key
            if existing.key < stats.min_item_key {
                stats.min_item_key = existing.key;
                stats.min_item_idx = idx;
            }

            if existing.group_id == group_id {
                // Check if this is the min key for the group
                if existing.key < stats.min_group_item_key.unwrap_or(f64::INFINITY) {
                    stats.min_group_item_key = Some(existing.key);
                    stats.min_group_item_idx = Some(idx);
                }
            }
        }

        stats
    }
}

#[derive(Debug)]
pub struct WeightedGroupSampler<T> {
    sample_size: usize,
    reservoir: Reservoir<T>,
    rng: ThreadRng,
}

impl<T: Clone> WeightedGroupSampler<T> {
    pub fn new(sample_size: usize) -> Self {
        Self {
            sample_size,
            reservoir: Reservoir {
                items: Vec::with_capacity(sample_size),
                group_counts: HashMap::new(),
            },
            rng: rand::rng(),
        }
    }

    pub fn add(&mut self, item: T, weight: f64, group_id: String) {
        assert!(weight > 0.0, "Weight must be positive");
        //let key = self.rng.gen::<f64>().powf(1.0 / weight);
        //let key = (-rand::random::<f64>().ln()) / weight;
        let key = (self.rng.random::<f64>().ln()) / weight;
        self.add_with_key(item, group_id, key);
    }

    fn add_with_key(&mut self, item: T, group_id: String, key: f64) {
        let keyed_item = KeyedItem {
            item,
            group_id: group_id.clone(),
            key,
        };

        // If reservoir isn't full yet, we always add the item
        if self.reservoir.items.len() < self.sample_size {
            self.reservoir.add(keyed_item);
            return;
        }

        // Make one pass through the reservoir to gather needed stats
        let stats = self.reservoir.stats_for_group(&group_id);

        // Is this a new group, or a group that's already in the reservoir?
        let same_group_count = self.reservoir.group_counts.get(&group_id).unwrap_or(&0);
        if *same_group_count > 0 {
            // Existing group. Use a simple (but theoretically biased & incorrect) replacement strategy:
            // If the min item is the only one of its group, consider only same-group min for replacement. Otherwise, use the standard strategy.
            let min_group_count = self
                .reservoir
                .group_counts
                .get(&self.reservoir.items[stats.min_item_idx].group_id)
                .unwrap();
            if *min_group_count > 1 {
                // Standard replacement strategy
                if stats.min_item_key < key {
                    self.reservoir.replace(stats.min_item_idx, keyed_item);
                }
            } else {
                // Only item of its group: use same-group replacement strategy
                if stats.min_group_item_key.unwrap() < key {
                    self.reservoir
                        .replace(stats.min_group_item_idx.unwrap(), keyed_item);
                }
                return;
            }
        } else {
            // New group: use the standard replacement strategy, ignoring group ID
            if stats.min_item_key < key {
                self.reservoir.replace(stats.min_item_idx, keyed_item);
            }
        }
    }

    pub fn get_sample(&self) -> Vec<T> {
        self.reservoir
            .items
            .iter()
            .map(|item| item.item.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashSet;

    #[test]
    fn test_reservoir_add() {
        let mut reservoir = Reservoir {
            items: Vec::new(),
            group_counts: HashMap::new(),
        };

        reservoir.add(KeyedItem {
            item: 1,
            group_id: "A".to_string(),
            key: 0.5,
        });
        reservoir.add(KeyedItem {
            item: 2,
            group_id: "B".to_string(),
            key: 0.6,
        });
        reservoir.add(KeyedItem {
            item: 3,
            group_id: "B".to_string(),
            key: 0.3,
        });

        assert_eq!(reservoir.items.len(), 3);
        assert_eq!(reservoir.group_counts.get("A"), Some(&1));
        assert_eq!(reservoir.group_counts.get("B"), Some(&2));
    }

    #[test]
    fn test_reservoir_replace() {
        let mut reservoir = Reservoir {
            items: Vec::new(),
            group_counts: HashMap::new(),
        };

        reservoir.add(KeyedItem {
            item: 1,
            group_id: "A".to_string(),
            key: 0.5,
        });
        reservoir.add(KeyedItem {
            item: 2,
            group_id: "B".to_string(),
            key: 0.6,
        });
        reservoir.replace(
            0,
            KeyedItem {
                item: 3,
                group_id: "B".to_string(),
                key: 0.7,
            },
        );

        assert_eq!(reservoir.items.len(), 2);
        assert_eq!(reservoir.items[0].item, 3);
        assert_eq!(reservoir.group_counts.get("A"), Some(&0));
        assert_eq!(reservoir.group_counts.get("B"), Some(&2));
    }

    #[test]
    fn test_reservoir_stats_for_group_exists() {
        let mut reservoir = Reservoir {
            items: Vec::new(),
            group_counts: HashMap::new(),
        };

        reservoir.add(KeyedItem {
            item: 1,
            group_id: "A".to_string(),
            key: 1.0,
        });
        reservoir.add(KeyedItem {
            item: 2,
            group_id: "B".to_string(),
            key: 0.0,
        });
        reservoir.add(KeyedItem {
            item: 3,
            group_id: "A".to_string(),
            key: 0.5,
        });

        let stats = reservoir.stats_for_group("A");

        assert_eq!(stats.min_item_idx, 1);
        assert_eq!(stats.min_item_key, 0.0);
        assert_eq!(stats.min_group_item_idx, Some(2));
        assert_eq!(stats.min_group_item_key, Some(0.5));
    }

    #[test]
    fn test_reservoir_stats_for_group_not_exists() {
        let mut reservoir = Reservoir {
            items: Vec::new(),
            group_counts: HashMap::new(),
        };

        reservoir.add(KeyedItem {
            item: 1,
            group_id: "A".to_string(),
            key: 1.0,
        });
        reservoir.add(KeyedItem {
            item: 2,
            group_id: "B".to_string(),
            key: 0.0,
        });

        let stats = reservoir.stats_for_group("C");

        assert_eq!(stats.min_item_idx, 1);
        assert_eq!(stats.min_item_key, 0.0);
        assert_eq!(stats.min_group_item_idx, None);
        assert_eq!(stats.min_group_item_key, None);
    }

    #[test]
    fn test_basic_sampling() {
        let mut sampler = WeightedGroupSampler::new(2);
        sampler.add_with_key(1, "A".to_string(), 0.5);
        sampler.add_with_key(2, "B".to_string(), 0.6);
        sampler.add_with_key(3, "C".to_string(), 0.7);

        let sample = sampler.get_sample();
        assert_eq!(sample.len(), 2);
        assert!(sample.iter().collect::<HashSet<_>>().len() == 2);
    }

    #[test]
    fn test_weight_influence() {
        let mut sampler = WeightedGroupSampler::new(10);

        // Add many items with low weights
        for i in 0..1000 {
            sampler.add(0, 0.1, i.to_string());
        }

        // Add many items with high weights
        for i in 0..1000 {
            sampler.add(1, 100.0, (i + 1000).to_string());
        }

        let sample = sampler.get_sample();
        // The same should have very few low weight samples
        assert!(sample.iter().filter(|val| **val == 0).count() < 2);
    }

    #[test]
    fn test_same_group_replacement_min_singleton() {
        let mut sampler = WeightedGroupSampler::new(3);

        // Add three items with different groups
        sampler.add_with_key(1, "A".to_string(), 0.1); // Lowest key
        sampler.add_with_key(2, "B".to_string(), 0.5);
        sampler.add_with_key(3, "C".to_string(), 0.5);

        // Add new item from group B with higher key
        sampler.add_with_key(4, "B".to_string(), 1.0);

        let sample = sampler.get_sample();
        assert_eq!(sample.len(), 3);
        assert!(sample.contains(&4));
        assert_eq!(false, sample.contains(&2));
    }

    #[test]
    fn test_same_group_replacement_min_not_singleton() {
        let mut sampler = WeightedGroupSampler::new(3);

        // Add three items with different groups
        sampler.add_with_key(1, "A".to_string(), 0.1); // Lowest key
        sampler.add_with_key(2, "A".to_string(), 0.5);
        sampler.add_with_key(3, "B".to_string(), 0.5);

        // Add new item from group B with higher key
        sampler.add_with_key(4, "B".to_string(), 0.6);

        let sample = sampler.get_sample();
        assert_eq!(sample.len(), 3);

        // Because the min item's group (A) has more than one item, we should
        // replace the min item, so there should be two items from group B in
        // the sample.
        assert!(sample.contains(&4));
        assert!(sample.contains(&3));
        assert_eq!(false, sample.contains(&1));
    }
}
