use std::path::PathBuf;
use chrono::NaiveDate;
use dashmap::DashMap;
#[derive(Debug, Default)]
pub struct DateIndex {
    dates: DashMap<NaiveDate, Vec<PathBuf>>,
}

impl DateIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, date: NaiveDate, path: PathBuf) {
        self.dates.entry(date)
            .or_insert_with(Vec::new)
            .push(path);
    }

    pub fn get(&self, date: NaiveDate) -> Vec<PathBuf> {
        self.dates.get(&date)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    pub fn all_dates(&self) -> Vec<(NaiveDate, Vec<PathBuf>)> {
        self.dates.iter()
            .map(|e| (*e.key(), e.value().clone()))
            .collect()
    }

    pub fn clear(&self) {
        self.dates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_date_add_and_get() {
        let idx = DateIndex::new();
        let d1 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();

        idx.add(d1, PathBuf::from("a.md"));
        idx.add(d1, PathBuf::from("b.md"));
        idx.add(d2, PathBuf::from("c.md"));

        assert_eq!(idx.get(d1).len(), 2);
        assert_eq!(idx.get(d2).len(), 1);
        assert!(idx.get(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()).is_empty());
    }

    #[test]
    fn test_date_all_dates() {
        let idx = DateIndex::new();
        let d1 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        idx.add(d1, PathBuf::from("a.md"));

        let all = idx.all_dates();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].0, d1);
    }

    #[test]
    fn test_date_clear() {
        let idx = DateIndex::new();
        idx.add(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), PathBuf::from("a.md"));
        idx.clear();
        assert!(idx.all_dates().is_empty());
    }
}
