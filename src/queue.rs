use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Error, format_err, Result};
use base58::{FromBase58, ToBase58};
use uuid::Uuid;
use std::time::SystemTime;
use std::marker::PhantomData;
use std::fs::DirEntry;

#[derive(Debug, Clone)]
pub struct ID(Uuid);

impl FromStr for ID {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        let id = id.from_base58().map_err(|_| format_err!("Invalid base65 input"))?;
        let id = Uuid::from_slice(&id)?;
        return Ok(ID(id));
    }
}

impl ID {
    pub fn random() -> Self {
        return Self(Uuid::new_v4());
    }

    pub fn as_string(&self) -> String {
        return self.0.as_bytes().to_base58();
    }
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.as_string());
    }
}

#[derive(Debug)]
pub struct Queue {
    path: PathBuf,

    path_tmp: PathBuf,
    path_new: PathBuf,
    path_err: PathBuf,
}

pub trait Staged {}

#[derive(Debug)]
pub struct Stage<'q> {
    queue: &'q Queue,

    id: ID,
    path: Option<PathBuf>,
}

pub trait State {
    fn path(queue: &Queue) -> &Path;
}

#[derive(Debug)]
pub struct NewState {}

#[derive(Debug)]
pub struct ErrState {}

#[derive(Debug)]
pub struct Job<'q, S: State> {
    queue: &'q Queue,

    id: ID,

    since: SystemTime,

    _state: PhantomData<S>,
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub queued: usize,
    pub failed: usize,
}

impl Queue {
    pub fn at(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let queue = Self {
            path_tmp: path.join("tmp"),
            path_new: path.join("new"),
            path_err: path.join("err"),
            path,
        };

        // Create all needed dirs
        fs::create_dir_all(&queue.path_tmp)?;
        fs::create_dir_all(&queue.path_new)?;
        fs::create_dir_all(&queue.path_err)?;

        return Ok(queue);
    }

    pub fn push(&self) -> Result<Stage> {
        let id = ID::random();
        let path = self.path_tmp.join(id.as_string());

        fs::create_dir(&path)?;

        return Ok(Stage {
            queue: self,
            id,
            path: Some(path),
        });
    }

    pub fn poll(&self) -> Result<Option<Job<NewState>>> {
        for entry in fs::read_dir(&self.path_new)? {
            return Ok(Some(Job::from_entry(self, entry?)?));
        }

        return Ok(None);
    }

    pub fn stats(&self) -> Result<Stats> {
        return Ok(Stats {
            queued: fs::read_dir(&self.path_new)?.count(),
            failed: fs::read_dir(&self.path_err)?.count(),
        });
    }

    pub fn failed(&self) -> Result<Vec<Job<ErrState>>> {
        return Ok(fs::read_dir(&self.path_err)?
            .map(|entry| Job::from_entry(self, entry?))
            .collect::<Result<Vec<_>>>()?);
    }
}

impl<'q> Stage<'q> {
    pub fn id(&self) -> &ID {
        return &self.id;
    }

    pub fn path(&self) -> &Path {
        return self.path.as_ref().expect("no path");
    }

    pub fn persist(mut self) -> Result<()> {
        return Ok(fs::rename(
            self.path.take().expect("no path"),
            self.queue.path_new.join(self.id.to_string())
        )?);
    }

    pub fn dismiss(mut self) -> Result<()> {
        return self.remove();
    }

    fn remove(&mut self) -> Result<()> {
        // FIXME: Use something like named tmpfile for this?

        if let Some(path) = self.path.take() {
            fs::remove_dir_all(path)?;
        }

        return Ok(());
    }
}

impl<'q> Drop for Stage<'q> {
    fn drop(&mut self) {
        self.remove().expect("dismiss failed");
    }
}

impl State for NewState {
    fn path(queue: &Queue) -> &Path {
        return &queue.path_new;
    }
}

impl State for ErrState {
    fn path(queue: &Queue) -> &Path {
        return &queue.path_err;
    }
}

impl<'q, S: State> Job<'q, S> {
    fn from_entry(queue: &'q Queue, entry: DirEntry) -> Result<Self> {
        let id = ID::from_str(entry.file_name().to_string_lossy().as_ref())?;

        return Ok(Job {
            queue,
            id,
            since: entry.metadata()?.modified()?,
            _state: PhantomData,
        });
    }

    pub fn complete(self) -> Result<()> {
        fs::remove_dir_all(&self.path())?;
        return Ok(());
    }

    pub fn error(self) -> Result<()> {
        fs::rename(self.path(), &self.queue.path_err.join(self.id.as_string()))?;
        return Ok(());
    }

    pub fn path(&self) -> PathBuf {
        return S::path(self.queue).join(self.id.as_string());
    }

    pub fn id(&self) -> &ID {
        return &self.id;
    }

    pub fn since(&self) -> &SystemTime {
        return &self.since;
    }
}

impl <'q> Job<'q, ErrState> {
    pub fn retry(self) -> Result<()> {
        fs::rename(self.path(), &self.queue.path_new.join(self.id.as_string()))?;
        return Ok(());
    }
}
