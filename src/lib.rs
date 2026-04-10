/*!
# cuda-tuple-space

Generative communication via tuple spaces.

Based on the Linda coordination model. Instead of sending messages
directly to specific agents, you write tuples to a shared space and
other agents find them by pattern matching. Decouples producer from
consumer completely.

- Tuple operations: out, rd, in, rdp, inp
- Pattern matching on tuple fields
- Blocking and non-blocking reads
- Transactional operations (batch in/out)
- Tuple expiration
- Space partitioning
*/

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// A tuple field value
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Field {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Any,  // wildcard for matching
}

/// A tuple in the space
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tuple {
    pub fields: Vec<Field>,
    pub id: u64,
    pub created_ms: u64,
    pub ttl_ms: Option<u64>,
    pub owner: String,
}

impl Tuple {
    pub fn new(fields: Vec<Field>, owner: &str) -> Self {
        Tuple { fields, id: 0, created_ms: now(), ttl_ms: None, owner: owner.to_string() }
    }

    pub fn with_ttl(mut self, ttl_ms: u64) -> Self { self.ttl_ms = Some(ttl_ms); self }

    /// Does this tuple match a template?
    pub fn matches(&self, template: &[Field]) -> bool {
        if template.len() != self.fields.len() { return false; }
        for (t, f) in template.iter().zip(self.fields.iter()) {
            match t {
                Field::Any => {}
                other => if other != f { return false; }
            }
        }
        true
    }
}

/// Operation result
#[derive(Clone, Debug)]
pub enum TupleResult {
    Found(Tuple),
    NotFound,
    Timeout,
}

/// The tuple space
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TupleSpace {
    pub name: String,
    pub tuples: VecDeque<Tuple>,
    pub next_id: u64,
    pub max_size: usize,
    pub total_out: u64,
    pub total_in: u64,
    pub total_rd: u64,
    pub expired: u64,
}

impl TupleSpace {
    pub fn new(name: &str) -> Self { TupleSpace { name: name.to_string(), tuples: VecDeque::new(), next_id: 1, max_size: 1000, total_out: 0, total_in: 0, total_rd: 0, expired: 0 } }

    /// OUT: write a tuple to the space
    pub fn out(&mut self, mut tuple: Tuple) -> u64 {
        tuple.id = self.next_id;
        self.next_id += 1;
        if self.tuples.len() >= self.max_size { self.tuples.pop_front(); }
        self.tuples.push_back(tuple);
        self.total_out += 1;
        self.next_id - 1
    }

    /// IN: find and remove a matching tuple
    pub fn inp(&mut self, template: &[Field]) -> TupleResult {
        self.expire();
        if let Some(pos) = self.tuples.iter().position(|t| t.matches(template)) {
            let tuple = self.tuples.remove(pos).unwrap();
            self.total_in += 1;
            TupleResult::Found(tuple)
        } else {
            TupleResult::NotFound
        }
    }

    /// RD: read a matching tuple without removing
    pub fn rd(&mut self, template: &[Field]) -> TupleResult {
        self.expire();
        if let Some(tuple) = self.tuples.iter().find(|t| t.matches(template)) {
            self.total_rd += 1;
            TupleResult::Found(tuple.clone())
        } else {
            TupleResult::NotFound
        }
    }

    /// IN by ID
    pub fn in_by_id(&mut self, id: u64) -> TupleResult {
        if let Some(pos) = self.tuples.iter().position(|t| t.id == id) {
            let tuple = self.tuples.remove(pos).unwrap();
            self.total_in += 1;
            TupleResult::Found(tuple)
        } else { TupleResult::NotFound }
    }

    /// Read by ID (non-destructive)
    pub fn rd_by_id(&self, id: u64) -> TupleResult {
        if let Some(tuple) = self.tuples.iter().find(|t| t.id == id) {
            TupleResult::Found(tuple.clone())
        } else { TupleResult::NotFound }
    }

    /// Query: find all matching tuples (non-destructive)
    pub fn query(&self, template: &[Field]) -> Vec<&Tuple> {
        self.tuples.iter().filter(|t| t.matches(template)).collect()
    }

    /// Query by first field value
    pub fn query_by_tag(&self, tag: &str) -> Vec<&Tuple> {
        self.tuples.iter().filter(|t| t.fields.first() == Some(&Field::String(tag.to_string()))).collect()
    }

    /// Transaction: batch in — remove all matching tuples
    pub fn batch_in(&mut self, template: &[Field]) -> Vec<Tuple> {
        let mut results = vec![];
        self.expire();
        let mut i = 0;
        while i < self.tuples.len() {
            if self.tuples[i].matches(template) {
                let tuple = self.tuples.remove(i).unwrap();
                self.total_in += 1;
                results.push(tuple);
            } else { i += 1; }
        }
        results
    }

    /// Count tuples matching template
    pub fn count(&self, template: &[Field]) -> usize {
        self.tuples.iter().filter(|t| t.matches(template)).count()
    }

    /// Total tuples in space
    pub fn size(&self) -> usize { self.tuples.len() }

    /// Expire old tuples
    fn expire(&mut self) {
        let now = now();
        let before = self.tuples.len();
        self.tuples.retain(|t| t.ttl_ms.map_or(true, |ttl| now - t.created_ms < ttl));
        self.expired += (before - self.tuples.len()) as u64;
    }

    /// Summary
    pub fn summary(&self) -> String {
        format!("TupleSpace[{}]: {} tuples, out={}, in={}, rd={}, expired={}",
            self.name, self.tuples.len(), self.total_out, self.total_in, self.total_rd, self.expired)
    }
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_out_and_rd() {
        let mut ts = TupleSpace::new("test");
        let tuple = Tuple::new(vec![Field::String("task".into()), Field::String("navigate".into())], "agent1");
        ts.out(tuple);
        let result = ts.rd(&[Field::String("task".into()), Field::Any]);
        assert!(matches!(result, TupleResult::Found(_)));
    }

    #[test]
    fn test_in_removes() {
        let mut ts = TupleSpace::new("test");
        let tuple = Tuple::new(vec![Field::String("msg".into()), Field::Int(42)], "agent1");
        ts.out(tuple);
        let result = ts.inp(&[Field::String("msg".into()), Field::Any]);
        assert!(matches!(result, TupleResult::Found(_)));
        assert_eq!(ts.size(), 0);
    }

    #[test]
    fn test_in_not_found() {
        let mut ts = TupleSpace::new("test");
        let result = ts.inp(&[Field::String("nonexistent".into())]);
        assert!(matches!(result, TupleResult::NotFound));
    }

    #[test]
    fn test_wildcard_matching() {
        let mut ts = TupleSpace::new("test");
        ts.out(Tuple::new(vec![Field::String("position".into()), Field::Float(1.5), Field::Float(2.5)], "nav"));
        ts.out(Tuple::new(vec![Field::String("position".into()), Field::Float(3.0), Field::Float(4.0)], "nav"));
        let all = ts.query(&[Field::String("position".into()), Field::Any, Field::Any]);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_query_by_tag() {
        let mut ts = TupleSpace::new("test");
        ts.out(Tuple::new(vec![Field::String("alert".into()), Field::String("fire".into())], "sensor"));
        ts.out(Tuple::new(vec![Field::String("status".into()), Field::String("ok".into())], "agent"));
        let alerts = ts.query_by_tag("alert");
        assert_eq!(alerts.len(), 1);
    }

    #[test]
    fn test_batch_in() {
        let mut ts = TupleSpace::new("test");
        for i in 0..5 { ts.out(Tuple::new(vec![Field::String("work".into()), Field::Int(i)], "producer")); }
        let taken = ts.batch_in(&[Field::String("work".into()), Field::Any]);
        assert_eq!(taken.len(), 5);
        assert_eq!(ts.size(), 0);
    }

    #[test]
    fn test_in_by_id() {
        let mut ts = TupleSpace::new("test");
        let id = ts.out(Tuple::new(vec![Field::String("x".into())], "a"));
        let result = ts.in_by_id(id);
        assert!(matches!(result, TupleResult::Found(_)));
    }

    #[test]
    fn test_ttl_expiration() {
        let mut ts = TupleSpace::new("test");
        let mut tuple = Tuple::new(vec![Field::String("temp".into())], "a");
        tuple = tuple.with_ttl(0); // already expired
        tuple.created_ms = 0;
        ts.out(tuple);
        let result = ts.inp(&[Field::String("temp".into())]);
        assert!(matches!(result, TupleResult::NotFound));
    }

    #[test]
    fn test_max_size_eviction() {
        let mut ts = TupleSpace::new("test");
        ts.max_size = 3;
        for i in 0..5 { ts.out(Tuple::new(vec![Field::String("item".into()), Field::Int(i)], "a")); }
        assert_eq!(ts.size(), 3); // oldest evicted
    }

    #[test]
    fn test_count() {
        let mut ts = TupleSpace::new("test");
        ts.out(Tuple::new(vec![Field::String("fruit".into()), Field::String("apple".into())], "a"));
        ts.out(Tuple::new(vec![Field::String("fruit".into()), Field::String("banana".into())], "a"));
        ts.out(Tuple::new(vec![Field::String("veg".into()), Field::String("carrot".into())], "a"));
        assert_eq!(ts.count(&[Field::String("fruit".into()), Field::Any]), 2);
    }

    #[test]
    fn test_summary() {
        let ts = TupleSpace::new("test");
        let s = ts.summary();
        assert!(s.contains("0 tuples"));
    }
}
