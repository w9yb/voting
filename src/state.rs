use std::collections::{BTreeMap, BTreeSet};

use rand::seq::SliceRandom as _;
use tokio::sync::Mutex;

pub struct ApplicationState {
    key: String,
    templates: tera::Tera,
    synchronized: Mutex<SynchronizedState>,
}

#[derive(Default)]
struct SynchronizedState {
    candidates: BTreeSet<String>,
    ballots: BTreeMap<String, Vec<String>>,
}

impl ApplicationState {
    pub fn new(key: String) -> Self {
        Self {
            key,
            templates: tera::Tera::new("assets/**/*").unwrap(),
            synchronized: Default::default(),
        }
    }

    pub fn templates(&self) -> &tera::Tera {
        &self.templates
    }

    pub fn check_key(&self, key: &str) -> bool {
        key == self.key
    }

    pub async fn list_candidates(&self) -> BTreeSet<String> {
        self.synchronized.lock().await.candidates.clone()
    }

    pub async fn list_ballots(&self) -> BTreeSet<String> {
        self.synchronized
            .lock()
            .await
            .ballots
            .keys()
            .cloned()
            .collect()
    }

    pub async fn add_ballot(
        &self,
        callsign: String,
        ranking: Vec<String>,
    ) -> Result<(), &'static str> {
        let mut state = self.synchronized.lock().await;
        if !ranking.iter().all(|c| state.candidates.contains(c)) {
            return Err("invalid_candidate");
        }
        match state.ballots.entry(callsign) {
            std::collections::btree_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(ranking);
                Ok(())
            }
            std::collections::btree_map::Entry::Occupied(_) => Err("duplicate_ballot"),
        }
    }

    pub async fn set_candidates(&self, new_candidates: BTreeSet<String>) {
        let mut state = self.synchronized.lock().await;
        state.candidates = new_candidates;
        state.ballots.clear();
    }

    pub async fn take_data(&self) -> Data {
        let mut state = self.synchronized.lock().await;
        let candidates = std::mem::take(&mut state.candidates);
        let ballots = std::mem::take(&mut state.ballots);
        drop(state);

        let (people, mut ballots): (_, Vec<_>) = ballots.into_iter().unzip();

        // prevent ballots from being in the same order as people
        ballots.shuffle(&mut rand::rng());

        Data {
            candidates,
            people,
            ballots,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct Data {
    pub candidates: BTreeSet<String>,
    pub people: Vec<String>,
    pub ballots: Vec<Vec<String>>,
}
