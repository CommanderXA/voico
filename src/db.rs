use chrono::{DateTime, Utc};
use color_eyre::Result;
use rusqlite::{params, types::Type, Connection};

use crate::{audio_clip::AudioClip, config::Config};

pub struct Db(Connection);

pub struct ClipMeta {
    pub id: usize,
    pub name: String,
    pub date: DateTime<Utc>,
}

fn encode(samples: &[f32]) -> Vec<u8> {
    let mut data = Vec::with_capacity(samples.len() * 4);
    for sample in samples {
        data.extend_from_slice(&sample.to_be_bytes());
    }
    data
}

fn decode(bytes: &[u8]) -> Vec<f32> {
    let mut samples = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks(4) {
        samples.push(f32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    samples
}

impl Db {
    pub fn open(config: &Config) -> Result<Db> {
        let connection = Connection::open((&config.db["name"]).to_owned() + ".sqlite")?;
        connection.pragma_update(None, "page_size", &config.db["page_size"])?;
        connection.pragma_update(None, "user_version", &config.db["user_version"])?;

        println!("{:?}", config.db["on_create"]);

        connection.execute(&config.db["on_create"], [])?;

        Ok(Db(connection))
    }

    pub fn save(&self, clip: &mut AudioClip) -> Result<()> {
        self.0.execute(
            "
                INSERT OR REPLACE INTO clips(id, name, date, sample_rate, samples)
                VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                clip.id,
                clip.name,
                clip.date.to_string(),
                clip.sample_rate,
                encode(&clip.samples),
            ],
        )?;

        if clip.id.is_none() {
            clip.id = Some(self.0.last_insert_rowid().try_into()?);
        }

        Ok(())
    }

    pub fn load(&self, name: &str) -> Result<Option<AudioClip>> {
        let mut stmt = self.0.prepare("SELECT * FROM clips WHERE name = ?1")?;

        let mut clip_iter = stmt.query_map([name], |row| {
            let date: String = row.get(2)?;
            let samples: Vec<u8> = row.get(4)?;

            Ok(AudioClip {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                date: date.parse().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(2, "date".to_string(), Type::Text)
                })?,
                sample_rate: row.get(3)?,
                samples: decode(&samples),
            })
        })?;

        Ok(if let Some(clip) = clip_iter.next() {
            Some(clip?)
        } else {
            None
        })
    }

    pub fn list(&self) -> Result<Vec<ClipMeta>> {
        let mut stmt = self
            .0
            .prepare("SELECT id, name, date FROM clips ORDER BY date")?;

        let clip_iter = stmt.query_map([], |row| {
            let date: String = row.get(2)?;

            Ok(ClipMeta {
                id: row.get(0)?,
                name: row.get(1)?,
                date: date.parse().map_err(|_| {
                    rusqlite::Error::InvalidColumnType(2, "date".to_string(), Type::Text)
                })?,
            })
        })?;

        Ok(clip_iter.collect::<Result<_, rusqlite::Error>>()?)
    }

    pub fn delete(&self, name: &str) -> Result<()> {
        self.0
            .execute("DELETE FROM clips WHERE name = ?1", params![name])?;

        Ok(())
    }
}
