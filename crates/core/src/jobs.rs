use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ids, CoreError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub kind: String,
    pub payload: Value,
    pub status: String,
    pub attempts: i64,
    pub error: Option<String>,
    pub created_at: String,
    pub finished_at: Option<String>,
}

pub fn enqueue(conn: &Connection, kind: &str, payload: &Value) -> Result<Job> {
    let job = Job {
        id: ids::new_id(),
        kind: kind.to_string(),
        payload: payload.clone(),
        status: "queued".into(),
        attempts: 0,
        error: None,
        created_at: ids::now_iso(),
        finished_at: None,
    };
    conn.execute(
        "INSERT INTO jobs (id, kind, payload, status, attempts, created_at)
         VALUES (?1, ?2, ?3, 'queued', 0, ?4)",
        rusqlite::params![job.id, job.kind, job.payload.to_string(), job.created_at],
    )?;
    Ok(job)
}

/// Claim the oldest queued job (marks it running, bumps attempts).
pub fn claim_next(conn: &Connection) -> Result<Option<Job>> {
    let claimed: Option<String> = conn
        .query_row(
            "UPDATE jobs SET status = 'running', attempts = attempts + 1
             WHERE id = (SELECT id FROM jobs WHERE status = 'queued'
                         ORDER BY created_at LIMIT 1)
             RETURNING id",
            [],
            |r| r.get(0),
        )
        .optional()?;
    match claimed {
        Some(id) => Ok(Some(get(conn, &id)?)),
        None => Ok(None),
    }
}

pub fn complete(conn: &Connection, id: &str) -> Result<()> {
    finish(conn, id, "done", None)
}

pub fn fail(conn: &Connection, id: &str, error: &str) -> Result<()> {
    finish(conn, id, "failed", Some(error))
}

/// Re-queue a failed job (the Problems feed's retry button).
pub fn retry(conn: &Connection, id: &str) -> Result<()> {
    let changed = conn.execute(
        "UPDATE jobs SET status = 'queued', error = NULL, finished_at = NULL
         WHERE id = ?1 AND status = 'failed'",
        [id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("failed job {id}")));
    }
    Ok(())
}

pub fn list_recent(conn: &Connection, limit: i64) -> Result<Vec<Job>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, payload, status, attempts, error, created_at, finished_at
         FROM jobs ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit], row_to_job)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn count_failed(conn: &Connection) -> Result<i64> {
    Ok(conn.query_row("SELECT COUNT(*) FROM jobs WHERE status = 'failed'", [], |r| r.get(0))?)
}

fn get(conn: &Connection, id: &str) -> Result<Job> {
    Ok(conn.query_row(
        "SELECT id, kind, payload, status, attempts, error, created_at, finished_at
         FROM jobs WHERE id = ?1",
        [id],
        row_to_job,
    )?)
}

fn finish(conn: &Connection, id: &str, status: &str, error: Option<&str>) -> Result<()> {
    let changed = conn.execute(
        "UPDATE jobs SET status = ?1, error = ?2, finished_at = ?3 WHERE id = ?4",
        rusqlite::params![status, error, ids::now_iso(), id],
    )?;
    if changed == 0 {
        return Err(CoreError::NotFound(format!("job {id}")));
    }
    Ok(())
}

fn row_to_job(r: &rusqlite::Row<'_>) -> rusqlite::Result<Job> {
    let payload: String = r.get(2)?;
    Ok(Job {
        id: r.get(0)?,
        kind: r.get(1)?,
        payload: serde_json::from_str(&payload).unwrap_or(Value::Null),
        status: r.get(3)?,
        attempts: r.get(4)?,
        error: r.get(5)?,
        created_at: r.get(6)?,
        finished_at: r.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;
    use serde_json::json;

    #[test]
    fn lifecycle_queued_running_done() {
        let db = Db::open_in_memory().unwrap();
        enqueue(&db.conn, "index_folder", &json!({ "path": "00_INBOX" })).unwrap();

        let job = claim_next(&db.conn).unwrap().unwrap();
        assert_eq!(job.status, "running");
        assert_eq!(job.attempts, 1);
        assert!(claim_next(&db.conn).unwrap().is_none()); // nothing else queued

        complete(&db.conn, &job.id).unwrap();
        let recent = list_recent(&db.conn, 10).unwrap();
        assert_eq!(recent[0].status, "done");
    }

    #[test]
    fn failed_jobs_count_and_retry() {
        let db = Db::open_in_memory().unwrap();
        enqueue(&db.conn, "metricool_push", &json!({})).unwrap();
        let job = claim_next(&db.conn).unwrap().unwrap();
        fail(&db.conn, &job.id, "429 rate limited").unwrap();
        assert_eq!(count_failed(&db.conn).unwrap(), 1);

        retry(&db.conn, &job.id).unwrap();
        assert_eq!(count_failed(&db.conn).unwrap(), 0);
        let again = claim_next(&db.conn).unwrap().unwrap();
        assert_eq!(again.attempts, 2);
    }
}
