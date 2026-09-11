use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn unique_nonempty(values: &[String]) -> bool {
    !values.is_empty()
        && values.iter().all(|v| !v.is_empty())
        && values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
#[derive(Debug, Clone, Deserialize)]
pub struct IdentityDefinition {
    pub id: String,
    pub display_name: String,
    pub groups: Vec<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Directory {
    pub issuer: String,
    pub audience: String,
    pub token_ttl_seconds: u64,
    pub identities: Vec<IdentityDefinition>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Resource {
    pub id: String,
    pub classification: String,
    pub path: String,
    pub allowed_roles: Vec<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Role {
    pub id: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GroupRole {
    pub group: String,
    pub role: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PolicyIdentity {
    pub id: String,
    pub display_name: String,
    pub roles: Vec<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Policy {
    pub default_decision: String,
    pub roles: Vec<Role>,
    pub group_role_mappings: Vec<GroupRole>,
    pub identities: Vec<PolicyIdentity>,
    pub resources: Vec<Resource>,
}

/// Cannot be deserialized from an HTTP request. Only a trusted session adapter
/// calls `validate_claims` AFTER its cryptographic and temporal verification.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedIdentity {
    id: String,
    groups: Vec<String>,
}
impl VerifiedIdentity {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn groups(&self) -> &[String] {
        &self.groups
    }
}

impl Directory {
    pub fn identity(&self, id: &str) -> Option<&IdentityDefinition> {
        self.identities.iter().find(|i| i.id == id)
    }
    pub fn validate_claims(&self, id: &str, groups: &[String]) -> Option<VerifiedIdentity> {
        let entry = self.identity(id)?;
        if entry.groups != groups {
            return None;
        }
        Some(VerifiedIdentity {
            id: id.into(),
            groups: groups.to_vec(),
        })
    }
}

impl Policy {
    pub fn validate(&self, directory: &Directory) -> Result<(), &'static str> {
        if self.default_decision != "deny"
            || directory.issuer.is_empty()
            || directory.audience.is_empty()
            || directory.token_ttl_seconds != 900
            || directory.identities.is_empty()
            || self.resources.is_empty()
        {
            return Err("invalid policy/directory defaults");
        }
        let roles: BTreeSet<_> = self.roles.iter().map(|r| r.id.as_str()).collect();
        if roles.len() != self.roles.len() || roles.is_empty() || roles.iter().any(|r| r.is_empty())
        {
            return Err("invalid roles");
        }
        let mut groups = BTreeMap::new();
        for mapping in &self.group_role_mappings {
            if mapping.group.is_empty()
                || !roles.contains(mapping.role.as_str())
                || groups
                    .insert(mapping.group.as_str(), mapping.role.as_str())
                    .is_some()
            {
                return Err("invalid group mapping");
            }
        }
        let mut identities = BTreeSet::new();
        for entry in &directory.identities {
            if !valid_id(&entry.id)
                || entry.display_name.is_empty()
                || !unique_nonempty(&entry.groups)
                || !identities.insert(&entry.id)
                || entry
                    .groups
                    .iter()
                    .any(|g| !groups.contains_key(g.as_str()))
            {
                return Err("invalid directory identity");
            }
            let matching: Vec<_> = self
                .identities
                .iter()
                .filter(|p| p.id == entry.id)
                .collect();
            if matching.len() != 1
                || matching[0].display_name != entry.display_name
                || !unique_nonempty(&matching[0].roles)
            {
                return Err("policy and directory identity mismatch");
            }
            let expected: BTreeSet<_> = entry.groups.iter().map(|g| groups[g.as_str()]).collect();
            let declared: BTreeSet<_> = matching[0].roles.iter().map(String::as_str).collect();
            if declared != expected {
                return Err("policy and directory roles mismatch");
            }
        }
        if self.identities.len() != directory.identities.len() {
            return Err("identity count mismatch");
        }
        let mut resources = BTreeSet::new();
        for r in &self.resources {
            if !valid_id(&r.id)
                || !resources.insert(&r.id)
                || !["PUBLIC", "RH", "IT"].contains(&r.classification.as_str())
                || !unique_nonempty(&r.allowed_roles)
                || r.allowed_roles
                    .iter()
                    .any(|role| !roles.contains(role.as_str()))
                || !r.path.starts_with("demo-documents/")
                || !r.path.ends_with(".md")
                || r.path.split('/').any(|p| {
                    p.is_empty()
                        || p == "."
                        || p == ".."
                        || !p
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                })
            {
                return Err("invalid resource");
            }
        }
        Ok(())
    }
    pub fn resource(&self, id: &str) -> Option<&Resource> {
        self.resources.iter().find(|r| r.id == id)
    }
    pub fn permits(&self, identity: &VerifiedIdentity, resource: &Resource) -> bool {
        identity.groups.iter().any(|group| {
            self.group_role_mappings.iter().any(|mapping| {
                &mapping.group == group && resource.allowed_roles.contains(&mapping.role)
            })
        })
    }
    pub fn authorized<'a>(&'a self, identity: &VerifiedIdentity) -> Vec<&'a Resource> {
        self.resources
            .iter()
            .filter(|r| self.permits(identity, r))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Policy, Directory) {
        (
            serde_json::from_str(include_str!(
                "../../../config/access-control/demo-policy.json"
            ))
            .unwrap(),
            serde_json::from_str(include_str!("../../../config/demo-idp/directory.json")).unwrap(),
        )
    }
    #[test]
    fn complete_four_by_fifteen_matrix() {
        let (policy, directory) = fixture();
        policy.validate(&directory).unwrap();
        assert_eq!(policy.resources.len(), 15);
        for entry in &directory.identities {
            let identity = directory.validate_claims(&entry.id, &entry.groups).unwrap();
            for resource in &policy.resources {
                let expected = match entry.id.as_str() {
                    "alice" => resource.classification != "IT",
                    "bob" => resource.classification != "RH",
                    "charlie" => resource.classification == "PUBLIC",
                    "oscar" => resource.id == "public-welcome",
                    _ => panic!("unknown fixture identity"),
                };
                assert_eq!(
                    policy.permits(&identity, resource),
                    expected,
                    "{} {}",
                    entry.id,
                    resource.id
                );
            }
        }
    }
    #[test]
    fn unknown_subject_and_group_spoofing_fail() {
        let (_, directory) = fixture();
        assert!(directory.validate_claims("unknown", &[]).is_none());
        assert!(
            directory
                .validate_claims("oscar", &["LAB_READERS".into()])
                .is_none()
        );
    }
    #[test]
    fn invalid_configs_fail_closed() {
        let (original, directory) = fixture();
        let mut p = original.clone();
        p.default_decision = "allow".into();
        assert!(p.validate(&directory).is_err());
        let mut p = original.clone();
        p.resources.push(p.resources[0].clone());
        assert!(p.validate(&directory).is_err());
        let mut p = original.clone();
        p.resources[0].path = "demo-documents/../AGENTS.md".into();
        assert!(p.validate(&directory).is_err());
        let mut p = original.clone();
        p.identities[3].roles.push("lab_reader".into());
        assert!(p.validate(&directory).is_err());
        let mut p = original;
        p.resources[0].allowed_roles.push("unknown".into());
        assert!(p.validate(&directory).is_err());
    }
    #[test]
    fn identifiers_are_bounded_ascii_and_never_paths() {
        for id in ["", "../x", "a/b", "%2f", "A", "é", "x_y"] {
            assert!(!valid_id(id));
        }
        assert!(valid_id(&"a".repeat(80)));
        assert!(!valid_id(&"a".repeat(81)));
    }
}
