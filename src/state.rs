use std::{
    collections::{BTreeMap, BTreeSet},
    ops::DerefMut as _,
};

use tokio::sync::{Mutex, RwLock};

pub struct ApplicationState {
    key: String,
    templates: tera::Tera,
    candidates: RwLock<BTreeSet<String>>,
    ballots: Mutex<BTreeMap<String, Vec<String>>>,
}

impl ApplicationState {
    pub fn new(key: String) -> Self {
        Self {
            key,
            templates: tera::Tera::new("assets/**/*").unwrap(),
            candidates: RwLock::default(),
            ballots: Mutex::default(),
        }
    }

    pub fn templates(&self) -> &tera::Tera {
        &self.templates
    }

    pub fn check_key(&self, key: &str) -> bool {
        key == self.key
    }

    pub async fn list_candidates(&self) -> BTreeSet<String> {
        self.candidates.read().await.clone()
    }

    pub async fn list_ballots(&self) -> BTreeSet<String> {
        self.ballots.lock().await.keys().cloned().collect()
    }

    pub async fn add_ballot(&self, callsign: String, ranking: Vec<String>) -> Result<(), ()> {
        let candidates = self.candidates.read().await;
        if !ranking.iter().all(|c| candidates.contains(c)) {
            return Err(());
        }
        self.ballots
            .lock()
            .await
            .try_insert(callsign, ranking)
            .map(|_| ())
            .map_err(|_| ())
    }

    pub async fn set_candidates(&self, new_candidates: BTreeSet<String>) {
        let mut candidates = self.candidates.write().await;
        let mut ballots = self.ballots.lock().await;
        *candidates = new_candidates;
        ballots.clear();
    }

    pub async fn take_data(&self) -> (BTreeSet<String>, BTreeMap<String, Vec<String>>) {
        let mut candidates = self.candidates.write().await;
        let mut ballots = self.ballots.lock().await;
        (
            std::mem::take(candidates.deref_mut()),
            std::mem::take(ballots.deref_mut()),
        )
    }
}
