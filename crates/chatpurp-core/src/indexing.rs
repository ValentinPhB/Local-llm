//! Préparation administrative déterministe ; aucun transport ni écriture ici.
use crate::{
    application::{Reader, document_body},
    policy::{Policy, valid_id},
};
use std::collections::BTreeSet;
pub const MAX_CHUNK: usize = 700;
pub const MAX_OVERLAP: usize = 160;
pub const MAX_PASSAGES: usize = 512;
pub const MAX_DIMENSIONS: usize = 4096;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndexError;
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub resource_id: String,
    pub classification: String,
    pub chunk_id: usize,
    pub text: String,
}
#[derive(Debug, Clone)]
pub struct IndexedPassage {
    pub chunk: Chunk,
    pub vector: Vec<f64>,
}

pub fn chunks(text: &str) -> Result<Vec<String>, IndexError> {
    if text.trim().is_empty() {
        return Err(IndexError);
    }
    // Paragraphes puis frontières de phrase ; les titres Markdown restent des données.
    let mut parts = Vec::new();
    let mut sentence = String::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            if !sentence.trim().is_empty() {
                parts.push(sentence.trim().to_owned());
                sentence.clear();
            }
            continue;
        }
        for word in line.split_whitespace() {
            if !sentence.is_empty() {
                sentence.push(' ');
            }
            sentence.push_str(word);
            if word.ends_with(['.', '!', '?']) {
                parts.push(std::mem::take(&mut sentence));
            }
        }
    }
    if !sentence.is_empty() {
        parts.push(sentence);
    }
    let mut units = Vec::new();
    for part in parts {
        let mut chars: Vec<char> = part.chars().collect();
        while chars.len() > MAX_CHUNK {
            let cut = chars[..=MAX_CHUNK]
                .iter()
                .rposition(|c| *c == ' ')
                .filter(|p| *p > 0)
                .unwrap_or(MAX_CHUNK);
            units.push(chars[..cut].iter().collect::<String>().trim().to_owned());
            chars = chars[cut..]
                .iter()
                .copied()
                .skip_while(|c| c.is_whitespace())
                .collect();
        }
        if !chars.is_empty() {
            units.push(chars.iter().collect());
        }
    }
    let mut output = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for unit in units {
        let candidate = if current.is_empty() {
            unit.chars().count()
        } else {
            current.join(" ").chars().count() + 1 + unit.chars().count()
        };
        if !current.is_empty() && candidate > MAX_CHUNK {
            output.push(current.join(" "));
            let last = current.pop().unwrap();
            current.clear();
            // Le recouvrement ne doit pas faire dépasser une unité déjà de 700 caractères.
            if last.chars().count() <= MAX_OVERLAP
                && last.chars().count() + 1 + unit.chars().count() <= MAX_CHUNK
            {
                current.push(last);
            }
        }
        current.push(unit);
    }
    if !current.is_empty() {
        output.push(current.join(" "));
    }
    if output.is_empty()
        || output
            .iter()
            .any(|s| s.is_empty() || s.chars().count() > MAX_CHUNK)
    {
        return Err(IndexError);
    }
    Ok(output)
}
pub fn prepare(
    policy: &Policy,
    reader: &dyn Reader,
    ids: &[String],
) -> Result<Vec<Chunk>, IndexError> {
    let mut unique = BTreeSet::new();
    if ids.is_empty()
        || ids.len() > policy.resources.len()
        || ids
            .iter()
            .any(|id| !valid_id(id) || !unique.insert(id) || policy.resource(id).is_none())
    {
        return Err(IndexError);
    }
    let mut result = Vec::new();
    for id in ids {
        let resource = policy.resource(id).ok_or(IndexError)?;
        let doc = reader.read(resource).map_err(|_| IndexError)?;
        for (chunk_id, text) in chunks(&document_body(&doc.content))?
            .into_iter()
            .enumerate()
        {
            if result.len() == MAX_PASSAGES {
                return Err(IndexError);
            }
            result.push(Chunk {
                resource_id: id.clone(),
                classification: resource.classification.clone(),
                chunk_id,
                text,
            });
        }
    }
    Ok(result)
}
pub fn validate_vector(vector: &[f64]) -> Result<(), IndexError> {
    if vector.is_empty() || vector.len() > MAX_DIMENSIONS || vector.iter().any(|v| !v.is_finite()) {
        Err(IndexError)
    } else {
        Ok(())
    }
}
pub fn validate_batch(passages: &[IndexedPassage]) -> Result<usize, IndexError> {
    if passages.is_empty() || passages.len() > MAX_PASSAGES {
        return Err(IndexError);
    }
    let dimensions = passages[0].vector.len();
    let mut seen = BTreeSet::new();
    for p in passages {
        validate_vector(&p.vector)?;
        if p.vector.len() != dimensions
            || !valid_id(&p.chunk.resource_id)
            || !["PUBLIC", "RH", "IT"].contains(&p.chunk.classification.as_str())
            || p.chunk.text.trim().is_empty()
            || p.chunk.text.chars().count() > MAX_CHUNK
            || !seen.insert((&p.chunk.resource_id, p.chunk.chunk_id))
        {
            return Err(IndexError);
        }
    }
    Ok(dimensions)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preparation_validates_all_ids_before_reading_and_preserves_metadata() {
        use crate::application::{Document, Failure};
        use crate::policy::Resource;
        use std::sync::Mutex;
        struct Spy(Mutex<Vec<String>>);
        impl Reader for Spy {
            fn read(&self, resource: &Resource) -> Result<Document, Failure> {
                self.0.lock().unwrap().push(resource.id.clone());
                Ok(Document {
                    resource_id: resource.id.clone(),
                    classification: resource.classification.clone(),
                    content:
                        "---\nid: public-welcome\nclassification: PUBLIC\n---\nBienvenue fictive."
                            .into(),
                })
            }
        }
        let policy: Policy = serde_json::from_str(include_str!(
            "../../../config/access-control/demo-policy.json"
        ))
        .unwrap();
        let reader = Spy(Mutex::new(vec![]));
        for ids in [
            vec![],
            vec!["public-welcome", "../../AGENTS.md"],
            vec!["public-welcome", "unknown"],
            vec!["public-welcome", "public-welcome"],
        ] {
            assert!(
                prepare(
                    &policy,
                    &reader,
                    &ids.into_iter().map(String::from).collect::<Vec<_>>()
                )
                .is_err()
            );
            assert!(reader.0.lock().unwrap().is_empty());
        }
        let result = prepare(&policy, &reader, &["public-welcome".into()]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resource_id, "public-welcome");
        assert_eq!(result[0].classification, "PUBLIC");
        assert_eq!(result[0].text, "Bienvenue fictive.");
        assert_eq!(*reader.0.lock().unwrap(), ["public-welcome"]);
    }
    #[test]
    fn sentence_boundaries_and_overlap_remain_bounded() {
        let text = format!("{}. Fin. {}", "a".repeat(680), "b".repeat(690));
        let output = chunks(&text).unwrap();
        assert_eq!(output.len(), 2);
        assert!(output[1].starts_with("Fin."));
        assert!(output.iter().all(|c| c.chars().count() <= 700));
        let text = format!("{}. Fin. {}", "a".repeat(680), "é".repeat(700));
        let output = chunks(&text).unwrap();
        assert_eq!(output.len(), 2);
        assert_eq!(output[1].chars().count(), 700);
        assert!(!output[1].starts_with("Fin."));
    }
    #[test]
    fn pathological_unicode_and_long_words_are_split_without_loss() {
        let word = "é".repeat(1600);
        let output = chunks(&word).unwrap();
        assert_eq!(output.concat(), word);
        assert_eq!(
            output.iter().map(|c| c.chars().count()).collect::<Vec<_>>(),
            [700, 700, 200]
        );
        assert!(chunks(" \n ").is_err());
    }
    #[test]
    fn vector_and_batch_bounds_fail_closed() {
        assert!(validate_vector(&[]).is_err());
        assert!(validate_vector(&[f64::NAN]).is_err());
        assert!(validate_vector(&vec![1.; 4097]).is_err());
        let p = IndexedPassage {
            chunk: Chunk {
                resource_id: "public-welcome".into(),
                classification: "PUBLIC".into(),
                chunk_id: 0,
                text: "Fictif".into(),
            },
            vector: vec![1., 2.],
        };
        assert_eq!(validate_batch(std::slice::from_ref(&p)), Ok(2));
        assert!(validate_batch(&[p.clone(), p.clone()]).is_err());
        let mut q = p.clone();
        q.chunk.chunk_id = 1;
        q.vector.push(3.);
        assert!(validate_batch(&[p, q]).is_err());
    }
}
